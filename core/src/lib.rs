//! TMail core: Tuta Mail protocol, encrypted cache, sync and actions.
//!
//! This crate has no UI dependency. A CLI, a Tauri shell or a future mobile
//! frontend can drive the same service. All cryptography is delegated to the
//! official Tuta SDK; this crate never invents primitives.

pub mod auth;
pub mod error;
pub mod event_bus;
pub mod folders;
pub mod mail_ext;
pub mod mail_set_entry_id;
pub mod models;
pub mod net;
pub mod send;
pub mod service;
pub mod session;
pub mod store;

pub use error::{CoreError, Result};
pub use service::{default_data_dir, CoreService, LoginOutcome, Snapshot, SyncEvent};

/// Human-readable client version. Keep the product clearly unofficial.
pub const PRODUCT_NAME: &str = "TMail (unofficial)";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CLIENT_LABEL: &str = "TMail/0.1 (unofficial, not affiliated with Tuta)";
