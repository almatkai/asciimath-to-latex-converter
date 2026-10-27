use beta_session_runtime::{adapter, CoreContext, CorePaths};
use serde_json::{json, Value};
#[tokio::main]
async fn main() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().to_path_buf();
    let ctx = CoreContext::new(
        CorePaths {
            data_dir: p.clone(),
            config_dir: p.clone(),
            resource_dir: p.clone(),
            temp_dir: p.clone(),
            home_dir: p,
        },
        None,
    );
    let mut s = json!({});
    let mut out = Vec::new();
    let mut view = adapter(
        &ctx,
        "extension:run-command",
        &json!({"preferences":{"defaultWrapLatex":"single"}}),
        &mut s,
    )
    .await
    .unwrap();
    out.push(view.clone());
    for (field, value) in [(0, "(x+1)/sqrt(y)"), (1, "double")] {
        view=adapter(&ctx,"extension:invoke-action",&json!({"actionId":view["root"]["children"][field]["props"]["actionId"],"formValues":{"value":value}}),&mut s).await.unwrap();
        out.push(view.clone());
    }
    view=adapter(&ctx,"extension:invoke-action",&json!({"actionId":view["actions"][0]["id"],"formValues":{"input":"(x+1)/sqrt(y)","wrapStyle":"double"}}),&mut s).await.unwrap();
    out.push(view.clone());
    view = adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":view["root"]["props"]["backActionId"]}),
        &mut s,
    )
    .await
    .unwrap();
    out.push(view);
    println!("{}", Value::Array(out));
}
