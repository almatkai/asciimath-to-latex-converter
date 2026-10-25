#[path = "../../../../adapter/v2_asciimath.rs"]
mod v2_asciimath;
#[path = "../../../../fixtures/beta/src-tauri/src/native_core/extensions/v2_sdk.rs"]
pub mod v2_sdk;
#[tokio::test]
async fn compiled_production_adapter_uses_unchanged_sdk() {
    use crate::native_core::common::{CoreContext, CorePaths};
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().to_path_buf();
    let ctx = CoreContext::headless(CorePaths {
        data_dir: p.clone(),
        config_dir: p.clone(),
        resource_dir: p.clone(),
        temp_dir: p.clone(),
        home_dir: p,
    });
    assert!(v2_asciimath::commands().contains(&"convert-asciimath-to-latex"));
    let mut state = json!({});
    let view = v2_asciimath::handle(
        &ctx,
        "convert-asciimath-to-latex",
        "extension:run-command",
        &json!({}),
        &mut state,
    )
    .await
    .unwrap();
    assert_eq!(view["actions"][0]["title"], "Copy LaTeX Code");
    // Public common routes remain stubbed; no application/OS calls.
    assert!(ctx.request("clipboard:write", vec![]).await.is_err());
}
