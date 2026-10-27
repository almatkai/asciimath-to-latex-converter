// Compiles/lints the real production adapter directly in every test target.
// The separate runtime dependency executes unchanged NativeSessions without cfg(test).
#[path = "support/native_core/mod.rs"]
pub mod native_core;
