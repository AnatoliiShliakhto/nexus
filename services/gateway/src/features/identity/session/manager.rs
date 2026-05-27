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
use moka::future::Cache;
use postcard::{from_bytes, to_allocvec};
use redis::aio::ConnectionManager;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::types::Action;
use tracing::{error, info, instrument, warn};

#[derive(Debug, Clone)]
pub(crate) struct SessionManager {
    inner: Arc<SessionManagerInner>,
}

#[derive(Debug, Clone)]
struct SessionManagerInner {
    database: Database,
    dpop: DpopValidator,
    redis: ConnectionManager,
    cache: Cache<SmolStr, Arc<Session>>,
    config: GatewayConfig,
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

        Self { inner: Arc::new(SessionManagerInner { database, redis, dpop, cache, config }) }
    }

    pub(crate) async fn authenticate_with_credentials(
        &self,
        username: &str,
        password: &str,
        identity: &IdentityContext,
    ) -> Result<(String, Arc<Session>), IdentityError> {
        let Some(proof) = &identity.dpop_proof else {
            return Err(IdentityError::header_missing());
        };

        let jkt =
            self.inner.dpop.verify(&identity.htm, &identity.htu, proof, None).await?.to_string();

        let res = self
            .inner
            .database
            .query("fn::auth_with_credentials($username, $password, $jkt, $ip, $user_agent);")
            .bind(("username", username.to_owned()))
            .bind(("password", password.to_owned()))
            .bind(("jkt", jkt.clone()))
            .bind(("ip", identity.ip.clone()))
            .bind(("user_agent", identity.user_agent.as_ref().map(ToString::to_string)))
            .execute()
            .await?
            .at::<AuthResponse>(0)?;

        match res {
            AuthResponse::Ok { data } => {
                let database_token =
                    HeaderValue::try_from(self.inner.database.generate_token(&data.account)?)
                        .map_err(|e| {
                            DatabaseError::internal().with_details(format!("Invalid token: {e}"))
                        })?;
                let session = Arc::new(Session {
                    id: SmolStr::new(data.session_id),
                    database_token,
                    jkt: SmolStr::new(jkt),
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

    pub(crate) async fn validate_session(
        &self,
        identity: &IdentityContext,
    ) -> Result<Arc<Session>, IdentityError> {
        let Some(session_id) = &identity.session_id else {
            return Err(IdentityError::authorization_missing());
        };

        let Some(proof) = &identity.dpop_proof else {
            return Err(IdentityError::header_missing());
        };

        let jkt_from_dpop = self
            .inner
            .dpop
            .verify(&identity.htm, &identity.htu, proof, identity.session_id.as_deref())
            .await?;

        let session = self
            .inner
            .cache
            .try_get_with(session_id.clone(), async {
                if let Some(s) = self.get_from_redis(session_id).await {
                    Ok(Arc::new(s))
                } else {
                    Err(IdentityError::unauthorized())
                }
            })
            .await
            .map_err(|_| IdentityError::unauthorized())?;

        if session.status != SessionStatus::Active {
            return Err(IdentityError::unauthorized())?;
        }

        if session.ip != identity.ip {
            warn!(sid = %session_id, "Session IP mismatch");
            return Err(IdentityError::ip_restricted());
        }

        if session.jkt != *jkt_from_dpop {
            warn!(sid = %session_id, "JKT mismatch detected");
            return Err(IdentityError::replay_detected());
        }

        Ok(session)
    }

    pub(crate) async fn refresh_session(
        &self,
        refresh_token: &str,
        identity: &IdentityContext,
    ) -> Result<(String, Arc<Session>), IdentityError> {
        let Some(session_id) = identity.session_id.as_deref() else {
            return Err(IdentityError::authorization_missing());
        };

        let Some(proof) = &identity.dpop_proof else {
            return Err(IdentityError::header_missing());
        };

        let jkt = self
            .inner
            .dpop
            .verify(&identity.htm, &identity.htu, proof, identity.session_id.as_deref())
            .await?
            .to_string();

        let res = self
            .inner
            .database
            .query("fn::auth_refresh($id, $token, $jkt, $ip, $ua);")
            .bind(("id", session_id.to_owned()))
            .bind(("token", refresh_token.to_owned()))
            .bind(("jkt", jkt.clone()))
            .bind(("ip", identity.ip.clone()))
            .bind(("ua", identity.user_agent.as_ref().map(ToString::to_string)))
            .execute()
            .await?
            .at::<AuthResponse>(0)?;

        match res {
            AuthResponse::Ok { data } => {
                let database_token =
                    HeaderValue::try_from(self.inner.database.generate_token(&data.account)?)
                        .map_err(|e| {
                            DatabaseError::internal().with_details(format!("Invalid token: {e}"))
                        })?;
                let session = Arc::new(Session {
                    id: SmolStr::new(data.session_id),
                    database_token,
                    jkt: SmolStr::new(jkt),
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
        let Some(session_id) = &identity.session_id else {
            return Err(IdentityError::authorization_missing());
        };

        let _ = self
            .inner
            .database
            .query("fn::auth_revoke($id, $ip);")
            .bind(("id", session_id.to_string()))
            .bind(("ip", identity.ip.clone()))
            .execute()
            .await?;

        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), DatabaseError> {
        let mut key = String::with_capacity(8 + session_id.len());
        let _ = write!(key, "session:{session_id}");

        let mut conn = self.inner.redis.clone();
        if let Err(e) = redis::cmd("DEL").arg(key).query_async::<()>(&mut conn).await {
            error!(error = ?e, sid = %session_id, "Failed to remove session from Redis during revoke");
        }

        self.inner.cache.invalidate(session_id).await;

        info!(sid = %session_id, "Session successfully revoked");

        Ok(())
    }

    async fn persist_session(&self, session: Arc<Session>) -> Result<(), IdentityError> {
        let sid = session.id.clone();
        let mut key = String::with_capacity(8 + sid.len());
        let _ = write!(key, "session:{sid}");

        self.inner.cache.insert(sid, session.clone()).await;

        let mut conn = self.inner.redis.clone();

        let encoded = to_allocvec(&session)?;

        if let Err(e) = redis::cmd("SET")
            .arg(key)
            .arg(encoded)
            .arg("EX")
            .arg(self.inner.config.get().security.identity.session.access_token_ttl_sec)
            .query_async::<()>(&mut conn)
            .await
        {
            warn!(error = ?e, "Failed to persist session in Redis");
        }

        Ok(())
    }

    async fn get_from_redis(&self, sid: &str) -> Option<Session> {
        let mut key = String::with_capacity(8 + sid.len());
        let _ = write!(key, "session:{sid}");

        let mut conn = self.inner.redis.clone();

        let data: Vec<u8> = redis::cmd("GET").arg(key).query_async(&mut conn).await.ok()?;

        from_bytes(&data).ok()
    }

    pub(crate) fn watch(&self) {
        let observer = SessionObserver::new(self.inner.database.clone(), self.clone());
        observer.spawn();
    }

    pub(crate) async fn update_session_status(
        &self,
        session: Arc<Session>,
        status: SessionStatus,
    ) -> Result<Arc<Session>, IdentityError> {
        let sid = session.id.clone();

        self.inner
            .database
            .query("UPDATE type::record('_session', $id) SET status = $status")
            .bind(("id", sid.to_string()))
            .bind(("status", status.as_str().to_owned()))
            .execute()
            .await?;

        let updated_session = session.with_status(status);

        self.persist_session(updated_session.clone()).await?;

        Ok(updated_session)
    }
}

// --- Session Observer ---

#[derive(Debug, Clone)]
struct SessionObserver {
    database: Database,
    manager: SessionManager,
}

impl SessionObserver {
    fn new(database: Database, manager: SessionManager) -> Self {
        Self { database, manager }
    }

    fn spawn(self) {
        sessions_maintenance(self.database.clone());

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
            .database
            .query(include_str!("queries/session_watch.surql"))
            .subscribe::<SessionChanged>()
            .await?;

        info!("LiveQuery established. Gateway is autonomously tracking session changes.");

        while let Some(res) = stream.next().await {
            match res {
                Ok(notification) => {
                    let sid = &notification.data.id;

                    match notification.action {
                        Action::Delete => {
                            self.manager.delete_session(sid).await?;
                        },
                        Action::Update if notification.data.status == "REVOKED" => {
                            self.manager.delete_session(sid).await?;
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
            if let Err(e) = database
                .query("DELETE _session WHERE status = 'REVOKED' OR expires_at < time::now();")
                .execute()
                .await
            {
                e.emit();
            }
            interval.tick().await;
        }
    });
}
