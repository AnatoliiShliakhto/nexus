use crate::core::config::{DpopStorageProvider, GatewayConfig};
use crate::error::ErrorEmit;
use crate::features::identity::dpop::engine::DpopValidator;
use crate::features::identity::error::IdentityError;
use crate::features::identity::session::models::{
    Actions, AuthResponse, Session, SessionChanged, SessionStatus,
};
use crate::infra::database::{Database, DatabaseError};
use crate::server::extractors::identity::IdentityContext;
use futures_util::StreamExt;
use fxhash::FxHashMap;
use http::HeaderValue;
use moka::sync::Cache;
use postcard::{from_bytes, to_allocvec};
use redis::aio::ConnectionManager;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::Action;
use tracing::{error, info, instrument, warn};

#[derive(Debug)]
pub(crate) struct SessionManager {
    database: Database,
    dpop: DpopValidator,
    redis: ConnectionManager, // If RPS is high, we can use a pool of connections
    cache: Cache<SmolStr, Arc<Session>>,
    ttl_sec: u64,
}

impl SessionManager {
    pub(crate) fn new(database: Database, redis: ConnectionManager, config: GatewayConfig) -> Self {
        let cfg = config.get().security.identity.clone();
        let dpop = DpopValidator::new(
            cfg.dpop.time_window_sec,
            cfg.dpop.max_capacity,
            if cfg.dpop.provider == DpopStorageProvider::Redis {
                Some(redis.clone())
            } else {
                None
            },
        );

        let cache = Cache::builder()
            .max_capacity(cfg.session.cache_capacity)
            .time_to_live(Duration::from_secs(cfg.session.access_token_ttl_sec))
            .build();

        Self { database, redis, dpop, cache, ttl_sec: cfg.session.access_token_ttl_sec }
    }

    #[inline]
    fn redis_key(sid: &str) -> String {
        let mut key = String::with_capacity(8 + sid.len());
        key.push_str("session:");
        key.push_str(sid);
        key
    }

    pub(crate) async fn validate_session(
        &self,
        identity: &IdentityContext,
    ) -> Result<Arc<Session>, IdentityError> {
        let session_id =
            identity.session_id.as_deref().ok_or_else(IdentityError::authorization_missing)?;
        let proof = identity.dpop_proof.as_ref().ok_or_else(IdentityError::header_missing)?;

        let jkt_from_dpop =
            self.dpop.verify(&identity.htm, &identity.htu, proof, Some(session_id)).await?;

        let session = if let Some(cached_session) = self.cache.get(session_id) {
            cached_session
        } else {
            if let Some(s) = self.get_from_redis(session_id).await {
                let arc_session = Arc::new(s);
                self.cache.insert(SmolStr::new(session_id), arc_session.clone());
                arc_session
            } else {
                return Err(IdentityError::unauthorized());
            }
        };

        if session.status != SessionStatus::Active {
            return Err(IdentityError::unauthorized());
        }

        if session.ip != identity.ip {
            warn!(sid = %session_id, "Session IP mismatch");
            return Err(IdentityError::ip_restricted());
        }

        if session.jkt.as_str() != jkt_from_dpop.as_str() {
            warn!(sid = %session_id, "JKT mismatch detected");
            return Err(IdentityError::replay_detected());
        }

        Ok(session)
    }

    pub(crate) async fn authenticate_with_credentials(
        &self,
        username: String,
        password: String,
        identity: &IdentityContext,
    ) -> Result<(String, Arc<Session>), IdentityError> {
        let Some(proof) = &identity.dpop_proof else {
            return Err(IdentityError::header_missing());
        };

        let jkt = self.dpop.verify(&identity.htm, &identity.htu, proof, None).await?;

        let user_agent = identity.user_agent.as_ref()
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        let res = self
            .database
            .query("fn::auth_with_credentials($username, $password, $jkt, $ip, $user_agent);")
            .bind(("username", username))
            .bind(("password", password))
            .bind(("jkt", jkt.as_str().to_owned()))
            .bind(("ip", identity.ip.clone()))
            .bind(("user_agent", user_agent))
            .execute()
            .await?
            .at::<AuthResponse>(0)?;

        match res {
            AuthResponse::Ok { data } => {
                let database_token =
                    HeaderValue::try_from(self.database.generate_token(&data.account)?).map_err(
                        |e| DatabaseError::internal().with_details(format!("Invalid token: {e}")),
                    )?;
                let session = Arc::new(Session {
                    id: SmolStr::new(data.session_id),
                    database_token,
                    jkt,
                    permissions: map_permissions(data.permissions),
                    ip: identity.ip.clone(),
                    status: data.status.parse().unwrap_or_default(),
                });

                self.persist_session(session.clone()).await?;

                Ok((data.refresh_token, session))
            },
            AuthResponse::Err { code } => Err(match code.as_str() {
                "INVALID_CREDENTIALS" => IdentityError::invalid_credentials(),
                "IP_RESTRICTED" => IdentityError::ip_restricted(),
                _ => IdentityError::unauthorized(),
            }),
        }
    }

    pub(crate) async fn refresh_session(
        &self,
        refresh_token: String,
        identity: &IdentityContext,
    ) -> Result<(String, Arc<Session>), IdentityError> {
        let Some(session_id) = identity.session_id.as_deref() else {
            return Err(IdentityError::authorization_missing());
        };

        let Some(proof) = &identity.dpop_proof else {
            return Err(IdentityError::header_missing());
        };

        let jkt = self
            .dpop
            .verify(&identity.htm, &identity.htu, proof, identity.session_id.as_deref())
            .await?;

        let user_agent = identity.user_agent.as_ref()
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        let res = self
            .database
            .query("fn::auth_refresh($id, $token, $jkt, $ip, $ua);")
            .bind(("id", session_id.to_owned()))
            .bind(("token", refresh_token))
            .bind(("jkt", jkt.as_str().to_owned()))
            .bind(("ip", identity.ip.clone()))
            .bind(("ua", user_agent))
            .execute()
            .await?
            .at::<AuthResponse>(0)?;

        match res {
            AuthResponse::Ok { data } => {
                let database_token =
                    HeaderValue::try_from(self.database.generate_token(&data.account)?).map_err(
                        |e| DatabaseError::internal().with_details(format!("Invalid token: {e}")),
                    )?;
                let session = Arc::new(Session {
                    id: SmolStr::new(data.session_id),
                    database_token,
                    jkt,
                    permissions: map_permissions(data.permissions.clone()),
                    ip: identity.ip.clone(),
                    status: data.status.parse().unwrap_or_default(),
                });

                self.persist_session(session.clone()).await?;

                Ok((data.refresh_token, session))
            },
            AuthResponse::Err { code } => Err(match code.as_str() {
                "SECURITY_VIOLATION" => IdentityError::replay_detected(),
                "IP_RESTRICTED" => IdentityError::ip_restricted(),
                _ => IdentityError::unauthorized(),
            }),
        }
    }

    pub(crate) async fn revoke_session(
        &self,
        identity: &IdentityContext,
    ) -> Result<(), IdentityError> {
        let session_id =
            identity.session_id.as_deref().ok_or_else(IdentityError::authorization_missing)?;

        let _ = self
            .database
            .query("fn::auth_revoke($id, $ip);")
            .bind(("id", session_id.to_owned()))
            .bind(("ip", identity.ip.clone()))
            .execute()
            .await?;

        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), DatabaseError> {
        let key = Self::redis_key(session_id);
        let mut conn = self.redis.clone();

        if let Err(e) = redis::cmd("DEL").arg(key).query_async::<()>(&mut conn).await {
            error!(error = ?e, sid = %session_id, "Failed to remove session from Redis");
        }

        self.cache.invalidate(session_id);

        info!(sid = %session_id, "Session successfully revoked");
        Ok(())
    }

    async fn persist_session(&self, session: Arc<Session>) -> Result<(), IdentityError> {
        let sid = session.id.clone();
        let key = Self::redis_key(&sid);

        self.cache.insert(sid, session.clone());

        let encoded = to_allocvec(&session)?;
        let mut conn = self.redis.clone();

        if let Err(e) = redis::cmd("SET")
            .arg(key)
            .arg(encoded)
            .arg("EX")
            .arg(self.ttl_sec)
            .query_async::<()>(&mut conn)
            .await
        {
            warn!(error = ?e, "Failed to persist session in Redis");
        }

        Ok(())
    }

    async fn get_from_redis(&self, sid: &str) -> Option<Session> {
        let key = Self::redis_key(sid);
        let mut conn = self.redis.clone();

        let data: Vec<u8> = redis::cmd("GET").arg(key).query_async(&mut conn).await.ok()?;
        from_bytes(&data).ok()
    }

    pub(crate) fn watch(&'static self) {
        let observer = SessionObserver::new(self);
        observer.spawn();
    }
}

// --- Session Observer ---

#[derive(Debug, Clone)]
struct SessionObserver {
    manager: &'static SessionManager,
}

impl SessionObserver {
    fn new(manager: &'static SessionManager) -> Self {
        Self { manager }
    }

    fn spawn(self) {
        sessions_maintenance(self.manager.database.clone());

        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(2);
            let max_backoff = Duration::from_mins(1);

            loop {
                match self.watch_loop().await {
                    Ok(()) => {
                        backoff = Duration::from_secs(2);
                    },
                    Err(e) => {
                        e.emit();
                        tokio::time::sleep(backoff).await;
                        backoff = std::cmp::min(backoff * 2, max_backoff);
                    },
                }
            }
        });
    }

    #[instrument(skip(self), name = "session_sync_loop")]
    async fn watch_loop(&self) -> Result<(), DatabaseError> {
        info!("Establishing SessionObserver baseline sync...");

        let mut stream = self
            .manager
            .database
            .query(include_str!("queries/session_watch.surql"))
            .subscribe::<SessionChanged>()
            .await?;

        info!("LiveQuery established. Gateway is tracking session changes.");

        while let Some(res) = stream.next().await {
            match res {
                Ok(notification) => {
                    let sid = &notification.data.id;
                    match notification.action {
                        Action::Delete => {
                            let _ = self.manager.delete_session(sid).await;
                        },
                        Action::Update if notification.data.status == "REVOKED" => {
                            let _ = self.manager.delete_session(sid).await;
                        },
                        _ => continue,
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

// --- Helpers ---

fn map_permissions(raw: HashMap<String, Vec<String>>) -> FxHashMap<String, Actions> {
    let mut processed = FxHashMap::with_capacity_and_hasher(raw.len(), Default::default());
    for (component, actions_vec) in raw {
        processed.insert(component, Actions::from_vec(&actions_vec));
    }
    processed
}

fn sessions_maintenance(database: Database) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = database
                .query("DELETE _session WHERE status = 'REVOKED' OR expires_at < time::now();")
                .execute()
                .await
            {
                e.emit();
            }
        }
    });
}
