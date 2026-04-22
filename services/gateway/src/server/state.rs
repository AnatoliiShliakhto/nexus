use crate::core::config::GatewayConfig;
use crate::core::error::GatewayError;
use crate::features::identity::session::manager::SessionManager;
use crate::features::proxy::route::RouteRegistry;
use crate::infra::client::{GatewayClient, create_client};
use crate::infra::database::Database;
use redis::aio::ConnectionManager;

#[derive(Debug, Clone)]
pub(crate) struct GatewayState {
    pub config: GatewayConfig,
    pub client: GatewayClient,
    pub database: Database,
    pub redis: ConnectionManager,
    pub routes: RouteRegistry,
    pub sessions: SessionManager,
}

impl GatewayState {
    pub(crate) async fn init(config: GatewayConfig) -> Result<Self, GatewayError> {
        let settings = config.get();

        let client = create_client(&settings.http_client);

        let database = Database::init(config.clone()).await?;

        let redis =
            redis::Client::open(settings.redis.url.as_str())?.get_connection_manager().await?;

        let sessions = SessionManager::new(database.clone(), redis.clone(), config.clone());
        sessions.watch();

        let routes = RouteRegistry::new();
        routes.watch(database.clone());

        Ok(Self { config, client, database, redis, sessions, routes })
    }
}
