use crate::error::GatewayError;
use nx_http::spin_sdk::http::{Method, Request};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Route {
    SpinUnstableExample,
    SpinExample,
    WasiExample,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum _RouteAccess {
    Public,
    Protected,
    Internal,
}

#[derive(Debug)]
pub(crate) struct RouteMetadata {
    pub config_key: &'static str,
    pub default_url: Option<&'static str>,
    pub prefix: &'static str,
    pub methods: &'static [Method],
    //    pub access: RouteAccess,
}

impl Route {
    pub(crate) fn from_request(request: &Request) -> Result<Self, GatewayError> {
        let path = request.path();
        let method = request.method();
        let segments = path.split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>();

        let route = match segments.as_slice() {
            ["examples", "spin", "unstable", ..] => Self::SpinUnstableExample,
            ["examples", "spin", ..] => Self::SpinExample,
            ["examples", "wasi", ..] => Self::WasiExample,
            _ => {
                return Err(
                    GatewayError::not_found().with_details_fn(|| format!("Invalid route: {path}"))
                );
            },
        };

        if !route.metadata().methods.contains(method) {
            return Err(GatewayError::method_not_allowed()
                .with_details_fn(|| format!("Method `{method}` not allowed for route: {path}")));
        }

        Ok(route)
    }

    pub(crate) const fn metadata(&self) -> RouteMetadata {
        match self {
            Self::SpinUnstableExample { .. } => RouteMetadata {
                config_key: "nx_spin_unstable_component_url",
                default_url: Some("http://nx-spin-unstable-component.spin.internal"),
                prefix: "/examples/spin/unstable",
                methods: &[Method::GET],
            },
            Self::SpinExample { .. } => RouteMetadata {
                config_key: "nx_spin_component_url",
                default_url: Some("http://nx-spin-component.spin.internal"),
                prefix: "/examples/spin",
                methods: &[Method::GET, Method::HEAD],
            },
            Self::WasiExample { .. } => RouteMetadata {
                config_key: "nx_wasi_component_url",
                default_url: Some("http://nx-wasi-component.spin.internal"),
                prefix: "/examples/wasi",
                methods: &[Method::GET],
            },
        }
    }
}
