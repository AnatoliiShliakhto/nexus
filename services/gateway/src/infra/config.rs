// use crate::error::GatewayError;
// use serde::Deserialize;
// use std::marker::PhantomData;
// use std::net::{IpAddr, Ipv4Addr};
// use std::ops::{Deref, DerefMut};
// use std::sync::Arc;
// use nx_config::Config;
//
// #[derive(Debug)]
// pub struct NotInitialized;
// #[derive(Debug)]
// pub struct Initialized;
//
// /// Top-level configuration.
// #[derive(Default, Debug, Clone, Deserialize)]
// #[serde(default)]
// pub struct SettingsInner {
//     pub server: ServerConfig,
//     // pub security: SecurityConfig,
//     // pub database: DatabaseConfig,
//     // pub storage: StorageConfig,
// }
//
// /// Thin Arc-wrapped config for inexpensive cloning into subsystems.
// #[derive(Default, Debug, Clone, Deserialize)]
// pub struct Settings<S = NotInitialized> {
//     #[serde(flatten, default)]
//     inner: Arc<SettingsInner>,
//     _state: PhantomData<S>,
// }
//
// impl Deref for Settings {
//     type Target = SettingsInner;
//
//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }
//
// impl DerefMut for Settings {
//     fn deref_mut(&mut self) -> &mut SettingsInner {
//         Arc::make_mut(&mut self.inner)
//     }
// }
//
// /// HTTP server configuration.
// #[derive(Debug, Clone, Deserialize)]
// #[serde(default, rename_all = "snake_case")]
// pub struct ServerConfig {
//     pub address: IpAddr,
//     pub port: u16,
//     tls: bool,
// }
//
// // --- Default ---
//
// impl Default for ServerConfig {
//     fn default() -> Self {
//         Self { address: IpAddr::V4(Ipv4Addr::UNSPECIFIED), port: 8080, tls: false }
//     }
// }
//
// // --- Implementations ---
//
// impl Settings<Initialized> {
//     pub(crate) fn load() -> Result<Self, GatewayError> {
//         let config = Config::init::<()>();
//
//     }
// }
