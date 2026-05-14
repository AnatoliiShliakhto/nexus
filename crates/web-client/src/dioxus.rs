use crate::client::WebClient;
use crate::error::WebClientError;
use crate::session::Session;
use dioxus::prelude::*;
use reqwest::Url;
use std::borrow::Cow;
use std::sync::Arc;

/// Global storage for the initialized `WebClient`.
static WEB_CLIENT: GlobalSignal<Option<WebClient>> = Signal::global(|| None);

/// Global storage for the current `Session`.
/// Initialized with a default (Guest) session.
static SESSION: GlobalSignal<Arc<Session>> = Signal::global(|| Arc::new(Session::default()));

/// Initializes the global `WebClient` and starts the session synchronization loop.
///
/// This function should typically be called once at the root of your application
/// (e.g., in the `App` component) before any other components attempt to access
/// the client or session.
///
/// ### Synchronization
/// It spawns a background task that listens for changes in the `WebClient`'s
/// session channel. When the session state changes (e.g., via an authentication
/// handler), the global `SESSION` signal is updated, triggering UI updates.
///
/// # Errors
/// Returns [`WebClientError`] if the underlying client fails to initialize
/// (e.g., failure to access secure storage for tokens).
pub fn use_init_web_client(
    service_name: impl Into<Cow<'static, str>>,
    base_url: impl AsRef<str>,
) -> Result<(), WebClientError> {
    let url = base_url.as_ref().parse::<Url>().map_err(|e| {
        WebClientError::internal()
            .with_message("Base URL is not a valid URL")
            .with_details(e.to_string())
    })?;
    let client = WebClient::init(service_name, url)?;

    *WEB_CLIENT.write() = Some(client.clone());
    *SESSION.write() = client.session_rx().borrow().clone();

    let mut rx = client.session_rx();
    spawn(async move {
        while rx.changed().await.is_ok() {
            let new_session = rx.borrow().clone();
            if *SESSION.peek() != new_session {
                *SESSION.write() = new_session;
            }
        }
    });

    Ok(())
}

/// Returns a cloned instance of the initialized `WebClient`.
///
/// Components use this to perform network requests or authentication actions.
///
/// # Panics
/// If called before [`use_init_web_client`] has been successfully executed.
#[must_use]
pub fn use_web_client() -> WebClient {
    WEB_CLIENT.read().clone().expect("WebClient accessed before initialization")
}

/// Returns a reactive `Signal` containing the current `Session`.
///
/// Use this in your components to react to login state changes,
/// check permissions, or display user information.
///
/// # Example
/// ```rust,ignore
/// #[component]
/// fn UserProfile() -> Element {
///     let session = use_session();
///
///     match &session.read().status {
///         SessionStatus::LoggedIn => rsx! { "Welcome back!" },
///         _ => rsx! { "Please log in." }
///     }
/// }
#[must_use]
pub fn use_session() -> Signal<Arc<Session>> {
    SESSION.signal()
}
