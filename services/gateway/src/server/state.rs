use crate::core::config::GatewayConfig;
use crate::core::error::GatewayError;
use crate::features::identity::session::manager::SessionManager;
use crate::features::proxy::route::RouteRegistry;
use crate::infra::client::{GatewayClient, create_client};
use crate::infra::database::Database;
use redis::aio::ConnectionManager;
use std::sync::OnceLock;

static STATE: OnceLock<&'static GatewayState> = OnceLock::new();

#[derive(Debug)]
#[repr(align(64))]
pub(crate) struct GatewayState {
    pub config: GatewayConfig,
    pub client: GatewayClient,
    pub database: Database,
    pub redis: ConnectionManager,
    pub routes: RouteRegistry,
    pub sessions: SessionManager,
}

impl GatewayState {
    pub(crate) async fn init(config: GatewayConfig) -> Result<&'static Self, GatewayError> {
        if let Some(state) = STATE.get() {
            return Ok(state);
        }

        let settings = config.get();

        let client = create_client(&settings.http_client);

        let database = Database::init(config.clone()).await?;

        let redis =
            redis::Client::open(settings.redis.url.as_str())?.get_connection_manager().await?;

        let sessions = SessionManager::new(database.clone(), redis.clone(), config.clone());
        let routes = RouteRegistry::new();

        let state = Self { config, client, database, redis, sessions, routes };

        // SAFETY: We're leaking the state to the static variable.
        let leaked_state = Box::leak(Box::new(state));

        match STATE.set(leaked_state) {
            Ok(()) => {
                leaked_state.sessions.watch();
                leaked_state.routes.watch(&leaked_state.database);

                Ok(leaked_state)
            },
            Err(_) => {
                // SAFETY: We've leaked the state, so we can safely drop it.
                unsafe {
                    let raw_ptr = leaked_state as *const GatewayState as *mut GatewayState;
                    let reconstructed = Box::from_raw(raw_ptr);
                    drop(reconstructed);
                }
                // SAFETY: It's safe to unwrap here, since we've already checked the lock.
                Ok(STATE.get().unwrap())
            },
        }
    }
}