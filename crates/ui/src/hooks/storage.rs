use dioxus::prelude::*;
use heck::ToSnakeCase;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::rc::Rc;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
struct Config<T> {
    #[serde(rename = "value")]
    inner: T,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Default> Default for Config<T> {
    fn default() -> Self {
        Self { inner: T::default() }
    }
}

/// A cross-platform hook for persisting state using `Signal`.
///
/// This hook provides a unified API for local storage, automatically switching between
/// `gloo-storage` (Web/Wasm) and `confy` (Desktop/Native).
///
/// ### Behavior
/// - **Desktop**: Data is stored in the user's config directory under the `nexus` application
///   subfolder. The `key` is converted to `snake_case` to ensure valid filenames on Windows.
///   Internally, the value is wrapped in a `config` key to satisfy TOML requirements.
/// - **Web**: Data is stored in `localStorage` using the provided key.
///
/// ### Type Requirements
/// The type `T` must implement `Serialize`, `DeserializeOwned`, `Clone`, and `Default`.
///
/// # Example
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
/// struct UserSettings {
///     username: String,
///     dark_mode: bool,
/// }
///
/// #[component]
/// fn Settings() -> Element {
///     // Persists as "user_settings.toml" on desktop or "user-settings" in localStorage
///     let mut settings = use_storage("UserSettings", || UserSettings {
///         username: "Guest".into(),
///         dark_mode: true,
///     });
///
///     rsx! {
///         input {
///             value: "{settings.read().username}",
///             oninput: move |e| settings.write().username = e.value()
///         }
///         button {
///             onclick: move |_| {
///                 let is_dark = settings.read().dark_mode;
///                 settings.write().dark_mode = !is_dark;
///             },
///             "Toggle Dark Mode: {settings.read().dark_mode}"
///         }
///     }
/// }
/// ```
pub fn use_storage<T>(key: impl AsRef<str>, init: impl FnOnce() -> T) -> Signal<T>
where
    T: Serialize + DeserializeOwned + Clone + Default + 'static,
{
    let key = Rc::<str>::from(key.as_ref().to_snake_case());
    let init_key = key.clone();

    let state = use_signal(move || {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            LocalStorage::get::<T>(&*init_key).unwrap_or_else(|_| init())
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            match confy::load::<Config<T>>(env!("WORKSPACE_NAME"), Some(&*init_key)) {
                Ok(wrapper) => wrapper.inner,
                Err(e) => {
                    warn!("Failed to load config `{init_key}`: {e:?}. Using default.");
                    init()
                },
            }
        }
    });

    use_effect(move || {
        let value = state.read();

        #[cfg(target_arch = "wasm32")]
        {
            use gloo_storage::{LocalStorage, Storage};
            let _ = LocalStorage::set(&*key, &*value);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let wrapper = Config { inner: (*value).clone() };
            if let Err(e) = confy::store("nexus", Some(&*key), &wrapper) {
                error!("Desktop persistence failed for `{key}`: {e:?}");
            }
        }
    });

    state
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct StorageProvider<T> {
    key: Rc<str>,
    data: Arc<RwLock<T>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> StorageProvider<T>
where
    T: Serialize + DeserializeOwned + Clone + Default + 'static,
{
    pub fn new(key: impl AsRef<str>, init: impl FnOnce() -> T) -> Self {
        let key = Rc::<str>::from(key.as_ref().to_snake_case());

        let initial: T = match confy::load::<Config<T>>(env!("WORKSPACE_NAME"), Some(&*key)) {
            Ok(wrapper) => wrapper.inner,
            Err(_) => init(),
        };

        Self { key, data: Arc::new(RwLock::new(initial)) }
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut lock = self.data.write();
        let result = f(&mut lock);

        let wrapper = Config { inner: lock.clone() };

        drop(lock);

        if let Err(e) = confy::store(env!("WORKSPACE_NAME"), Some(&*self.key), wrapper) {
            error!("Failed to store config '{}': {e}", self.key);
        }

        result
    }

    #[must_use]
    pub fn read(&self) -> T {
        self.data.read().clone()
    }
}
