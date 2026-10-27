//! Unchanged pinned beta common/SDK/NativeSessions, real registry/catalog/dispatch/adapter; explicit Tauri and unrelated-adapter stubs.
// Fixtures are loaded by relative path so extracted packages are self contained.
pub mod native_core;

pub use native_core::{
    common::*,
    extensions::{ack, adapter, Harness},
};

pub use native_core::extensions::{catalog, install_native, registry_call};
