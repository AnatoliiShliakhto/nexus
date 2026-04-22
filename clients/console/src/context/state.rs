#![allow(dead_code)]

use crate::shared::utils::path::app_data_dir;
use dioxus::prelude::*;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

static STATE: GlobalSignal<AppState> = Signal::global(AppState::default);

#[derive(Debug)]
pub(crate) struct AppState {
    pub data_dir: PathBuf,
    pub is_authorized: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self { data_dir: app_data_dir(), is_authorized: false }
    }
}

pub(crate) fn state() -> impl Deref<Target = AppState> {
    STATE.read()
}

pub(crate) fn mut_state() -> impl DerefMut<Target = AppState> {
    STATE.write()
}
