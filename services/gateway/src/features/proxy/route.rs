use crate::error::ErrorEmit;
use crate::infra::database::{Database, DatabaseError};
use crossbeam_epoch::{self as epoch, Atomic, Guard, Owned};
use futures_util::StreamExt;
use smol_str::SmolStr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use surrealdb::types::SurrealValue;
use tracing::{error, info, instrument, warn};
use url::Url;

// --- Domain Models ---

#[derive(Debug, Clone)]
pub(crate) struct ServiceRoute {
    pub component: SmolStr,
    pub target: Arc<Url>,
    pub route: SmolStr,
    pub protected: bool,
}

#[derive(Debug, SurrealValue)]
struct ActiveComponent {
    component: String,
    target: String,
    route: String,
    protected: bool,
}

#[derive(Debug)]
pub(crate) struct RouteTable {
    routes: Vec<ServiceRoute>,
}

// --- Route Registry ---

#[derive(Debug)]
pub(crate) struct RouteRegistry {
    inner: Atomic<RouteTable>,
}

impl RouteRegistry {
    pub(crate) fn new() -> Self {
        Self { inner: Atomic::null() }
    }

    /// Replaces the current routing table with a new one.
    pub(crate) fn reload(&self, mut routes: Vec<ServiceRoute>) {
        for r in &mut routes {
            if r.route != "/" {
                r.route = SmolStr::new(r.route.trim_end_matches('/'));
            }
        }
        routes.sort_by(|a, b| b.route.len().cmp(&a.route.len()));

        let new_table = Owned::new(RouteTable { routes });
        let guard = &epoch::pin();

        let old_table = self.inner.swap(new_table, Ordering::AcqRel, guard);

        if !old_table.is_null() {
            unsafe {
                guard.defer_destroy(old_table);
            }
        }
    }

    /// Performs a prefix match on the routing table.
    #[inline]
    pub(crate) fn match_prefix<'g>(
        &self,
        path: &str,
        guard: &'g Guard,
    ) -> Option<&'g ServiceRoute> {
        let snapshot = self.inner.load(Ordering::Acquire, guard);
        let table = unsafe { snapshot.as_ref() }?;

        table.routes.iter().find(|r| {
            let prefix = r.route.as_str();
            let prefix_len = prefix.len();

            path.starts_with(prefix)
                && (prefix == "/"
                    || path.len() == prefix_len
                    || path.as_bytes().get(prefix_len) == Some(&b'/'))
        })
    }

    pub(crate) fn watch(&'static self, database: &'static Database) {
        let observer = RouteObserver::new(database, self);
        observer.spawn();
    }
}

// --- Route Observer ---

#[derive(Debug)]
struct RouteObserver {
    database: &'static Database,
    registry: &'static RouteRegistry,
}

impl RouteObserver {
    fn new(database: &'static Database, registry: &'static RouteRegistry) -> Self {
        Self { database, registry }
    }

    fn spawn(self) {
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

    #[instrument(skip(self), name = "route_sync_loop")]
    async fn watch_loop(&self) -> Result<(), DatabaseError> {
        info!("Establishing RouteObserver baseline sync...");
        self.perform_full_sync().await?;

        let mut stream = self
            .database
            .query(include_str!("queries/component_watch.surql"))
            .subscribe::<ActiveComponent>()
            .await?;

        info!("LiveQuery established. Gateway is autonomously tracking route changes.");

        while let Some(res) = stream.next().await {
            match res {
                Ok(_) => {
                    if let Err(e) = self.perform_full_sync().await {
                        warn!(error = %e, "Failed to apply route diff, state might be stale");
                    }
                },
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    async fn perform_full_sync(&self) -> Result<(), DatabaseError> {
        let components = self
            .database
            .query(include_str!("queries/component_list.surql"))
            .execute()
            .await?
            .many_at::<ActiveComponent>(0)?;

        let routes = components
            .into_iter()
            .filter_map(|c| match c.target.parse::<Url>() {
                Ok(parsed_target) => Some(ServiceRoute {
                    component: SmolStr::new(c.component),
                    target: Arc::new(parsed_target),
                    route: SmolStr::new(c.route),
                    protected: c.protected,
                }),
                Err(e) => {
                    error!(component = %c.component, target = %c.target, details = %e, "URL parse failed");
                    None
                }
            })
            .collect::<Vec<ServiceRoute>>();

        let count = routes.len();
        self.registry.reload(routes);
        info!(routes = count, "Route registry updated");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_route(component: &str, route: &str) -> ServiceRoute {
        ServiceRoute {
            component: SmolStr::new(component),
            target: Arc::new(Url::parse("http://localhost:8080").unwrap()),
            route: SmolStr::new(route),
            protected: false,
        }
    }

    #[test]
    fn test_zero_copy_matching() {
        let registry = RouteRegistry::new();
        let routes = vec![
            mock_route("api", "/api"),
            mock_route("auth", "/auth/v1"),
            mock_route("root", "/"),
        ];

        registry.reload(routes);

        let guard = epoch::pin();

        let match1 = registry.match_prefix("/api", &guard).expect("Should match /api");
        assert_eq!(match1.component, "api");

        let match2 = registry.match_prefix("/api/users", &guard).expect("Should match /api/users");
        assert_eq!(match2.component, "api");

        let match3 = registry.match_prefix("/random", &guard).expect("Should match /");
        assert_eq!(match3.component, "root");
    }

    #[test]
    fn test_atomic_reload_no_contention() {
        let registry = RouteRegistry::new();

        registry.reload(vec![mock_route("v1", "/service")]);

        {
            let guard = epoch::pin();
            let route = registry.match_prefix("/service", &guard).unwrap();
            assert_eq!(route.component, "v1");

            registry.reload(vec![mock_route("v2", "/service")]);

            assert_eq!(route.component, "v1");
        }

        let guard = epoch::pin();
        let route = registry.match_prefix("/service", &guard).unwrap();
        assert_eq!(route.component, "v2");
    }

    #[test]
    fn test_trailing_slashes() {
        let registry = RouteRegistry::new();
        registry.reload(vec![mock_route("service", "/app/")]);

        let guard = epoch::pin();
        assert!(registry.match_prefix("/app", &guard).is_some());
        assert!(registry.match_prefix("/app/", &guard).is_some());
        assert!(registry.match_prefix("/app/data", &guard).is_some());
    }
}
