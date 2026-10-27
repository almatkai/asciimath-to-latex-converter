use super::common::{CoreContext, CoreResult};
use serde_json::Value;
use std::sync::{Arc, Mutex};
#[path = "../../../../../../fixtures/beta/src-tauri/src/native_core/extensions/native.rs"]
mod native;
#[path = "../../../../../../fixtures/beta/src-tauri/src/native_core/extensions/registry.rs"]
mod registry;
#[path = "../../../../../../adapter/v2_asciimath.rs"]
mod v2_asciimath;
#[path = "../../../../../../integration/host-files/src-tauri/src/native_core/extensions/v2_catalog.rs"]
mod v2_catalog;
#[path = "../../../../../../integration/host-files/src-tauri/src/native_core/extensions/v2_runtime.rs"]
mod v2_runtime;
#[path = "../../../../../../fixtures/beta/src-tauri/src/native_core/extensions/v2_sdk.rs"]
pub mod v2_sdk;
// Only unrelated compiled handlers are stubbed. The AsciiMath registration,
// registry, catalog, NativeSessions, SDK and common module execute real source.
macro_rules! unrelated_adapter {
    ($name:ident) => {
        mod $name {
            use super::*;
            pub fn commands() -> &'static [&'static str] {
                &[]
            }
            pub async fn handle(
                _: &CoreContext,
                _: &str,
                _: &str,
                _: &Value,
                _: &mut Value,
            ) -> CoreResult {
                Err("Unrelated adapter stub".into())
            }
        }
    };
}
unrelated_adapter!(v2_amphetamine);
unrelated_adapter!(v2_audio_device);
unrelated_adapter!(v2_cleanshotx);
unrelated_adapter!(v2_color_picker);
unrelated_adapter!(v2_gif_search);
unrelated_adapter!(v2_google_calendar);
unrelated_adapter!(v2_google_search);
unrelated_adapter!(v2_port_manager);
unrelated_adapter!(v2_speedtest);
unrelated_adapter!(v2_timers);
unrelated_adapter!(v2_perplexity);
#[derive(Default, Clone)]
pub struct Harness {
    sessions: native::NativeSessions,
    files: Arc<Mutex<()>>,
}
impl Harness {
    pub async fn call(&self, ctx: &CoreContext, op: &str, p: Value) -> Value {
        self.sessions
            .handle(ctx, &self.files, op, &[p])
            .await
            .expect("native route")
            .expect("handled error value")
    }
    pub async fn clear(&self) {
        self.sessions.clear().await;
    }
    pub async fn clear_slug(&self) {
        self.sessions
            .clear_slug("asciimath-to-latex-converter")
            .await;
    }
}
pub async fn adapter(ctx: &CoreContext, op: &str, p: &Value, s: &mut Value) -> CoreResult {
    v2_asciimath::handle(ctx, "convert-asciimath-to-latex", op, p, s).await
}
pub fn ack(v: &CoreResult) -> bool {
    v2_asciimath::acknowledged(v)
}

pub fn install_native(ctx: &CoreContext) -> CoreResult {
    v2_catalog::cli_install(ctx, "com.tezbar.native.asciimath-to-latex-converter")
}
pub fn catalog(ctx: &CoreContext) -> CoreResult {
    v2_catalog::cli_list(ctx)
}
pub fn registry_call(ctx: &CoreContext, channel: &str, args: &[Value]) -> CoreResult {
    registry::handle(ctx, channel, args)
}
