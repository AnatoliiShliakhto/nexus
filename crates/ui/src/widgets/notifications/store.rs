use dioxus::prelude::*;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NOTIFY_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: Rc<str>,
    pub title: Rc<str>,
    pub description: Option<Rc<str>>,
    pub duration: u64,
    pub n_type: NotificationType,
    pub is_read: bool,
    pub show_toast: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct NotificationStore {
    pub items: Signal<Vec<Notification>>,
    pub is_open: Signal<bool>,
}

impl NotificationStore {
    pub fn new(&self, title: impl Into<String>) -> Notify {
        Notify::new(*self, title)
    }

    pub fn push(&mut self, notif: Notification) {
        self.items.write().push(notif);
    }

    pub fn dismiss_toast(&mut self, id: &str) {
        if let Some(n) = self.items.write().iter_mut().find(|n| n.id.as_ref() == id) {
            n.show_toast = false;
        }
    }

    pub fn mark_all_read(&mut self) {
        for n in self.items.write().iter_mut() {
            n.is_read = true;
        }
    }

    pub fn clear_all(&mut self) {
        self.items.write().clear();
    }

    #[must_use]
    pub fn unread_count(&self) -> usize {
        self.items.read().iter().filter(|n| !n.is_read).count()
    }

    pub fn open(&mut self) {
        self.is_open.set(true);
        self.mark_all_read();
    }

    pub fn close(&mut self) {
        self.is_open.set(false);
    }
}

#[must_use]
pub fn use_init_notifications() -> NotificationStore {
    use_context_provider(|| NotificationStore {
        items: Signal::new(Vec::new()),
        is_open: Signal::new(false),
    })
}

#[must_use]
pub fn use_notifications() -> NotificationStore {
    use_context::<NotificationStore>()
}

#[derive(Debug)]
pub struct Notify {
    store: NotificationStore,
    title: Rc<str>,
    description: Option<Rc<str>>,
    duration: u64,
    n_type: NotificationType,
    show_toast: bool,
}

impl Notify {
    pub fn new(store: NotificationStore, title: impl Into<String>) -> Self {
        Self {
            store,
            title: Rc::from(title.into()),
            description: None,
            duration: 5,
            n_type: NotificationType::Info,
            show_toast: true,
        }
    }

    #[must_use]
    pub fn desc(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(Rc::from(desc.into()));
        self
    }

    #[must_use]
    pub const fn duration(mut self, duration: u64) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub const fn success(mut self) -> Self {
        self.n_type = NotificationType::Success;
        self
    }

    #[must_use]
    pub const fn error(mut self) -> Self {
        self.n_type = NotificationType::Error;
        self
    }

    #[must_use]
    pub const fn warning(mut self) -> Self {
        self.n_type = NotificationType::Warning;
        self
    }

    #[must_use]
    pub const fn silent(mut self) -> Self {
        self.show_toast = false;
        self
    }

    pub fn send(mut self) {
        let id = NOTIFY_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.store.push(Notification {
            id: Rc::from(format!("ntf-{id}")),
            title: self.title,
            description: self.description,
            duration: self.duration,
            n_type: self.n_type,
            is_read: false,
            show_toast: self.show_toast,
        });
    }
}

#[must_use]
pub fn use_toast() -> ToastApi {
    let store = use_notifications();
    ToastApi { store }
}

#[derive(Debug)]
pub struct ToastApi {
    store: NotificationStore,
}

impl ToastApi {
    pub fn info(&self, title: impl Into<String>) -> Notify {
        Notify::new(self.store, title)
    }
    pub fn success(&self, title: impl Into<String>) -> Notify {
        Notify::new(self.store, title).success()
    }
    pub fn warning(&self, title: impl Into<String>) -> Notify {
        Notify::new(self.store, title).warning()
    }
    pub fn error(&self, title: impl Into<String>) -> Notify {
        Notify::new(self.store, title).error()
    }
}
