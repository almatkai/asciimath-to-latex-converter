use beta_session_runtime::{ack, adapter, CoreContext, CorePaths, CoreResult, Harness};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
const CMD: &str = "convert-asciimath-to-latex";
const ID: &str = "com.tezbar.native.asciimath-to-latex-converter";
fn context(d: &std::path::Path) -> CoreContext {
    CoreContext::new(
        CorePaths {
            data_dir: d.join("data"),
            config_dir: d.join("config"),
            resource_dir: d.join("res"),
            temp_dir: d.join("tmp"),
            home_dir: d.into(),
        },
        None,
    )
}
fn action(v: &Value, contains: &str) -> String {
    v["actions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["title"].as_str().unwrap().contains(contains))
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .into()
}
fn field(v: &Value, index: usize) -> String {
    v["root"]["children"][index]["props"]["actionId"]
        .as_str()
        .unwrap()
        .into()
}
fn map(input: &str, wrap: &str) -> Value {
    json!({"input":input,"wrapStyle":wrap})
}
async fn launch(ctx: &CoreContext, wrap: &str, s: &mut Value) -> Value {
    adapter(
        ctx,
        "extension:run-command",
        &json!({"preferences":{"defaultWrapLatex":wrap}}),
        s,
    )
    .await
    .unwrap()
}
async fn change(ctx: &CoreContext, v: &Value, index: usize, value: &str, s: &mut Value) -> Value {
    adapter(
        ctx,
        "extension:invoke-action",
        &json!({"actionId":field(v,index),"formValues":{"value":value}}),
        s,
    )
    .await
    .unwrap()
}
fn install(ctx: &CoreContext) {
    beta_session_runtime::install_native(ctx).unwrap();
}
#[tokio::test]
async fn defaults_empty_actions_source_order_and_supported_routes() {
    for wrap in ["none", "single", "double"] {
        let d = tempfile::tempdir().unwrap();
        let ctx = context(d.path());
        let calls = Arc::new(Mutex::new(vec![]));
        let log = calls.clone();
        ctx.set_router(Arc::new(move |c, a| {
            log.lock().unwrap().push((c, a));
            Box::pin(async { Ok(json!({"ok":true})) })
        }))
        .unwrap();
        let mut s = json!({});
        let v = launch(&ctx, wrap, &mut s).await;
        assert_eq!(
            v["actions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a["title"].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["Copy LaTeX Code", "Paste LaTeX Code"]
        );
        assert_eq!(v["root"]["children"][1]["props"]["defaultValue"], wrap);
        assert!(v["root"]["children"][0]["props"].get("value").is_none());
        let copy = action(&v, "Copy");
        let result = adapter(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":copy,"formValues":map("",wrap)}),
            &mut s,
        )
        .await
        .unwrap();
        assert!(result["message"].as_str().unwrap().contains("acknowledged"));
        let paste = action(&result, "Paste");
        adapter(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":paste,"formValues":map("",wrap)}),
            &mut s,
        )
        .await
        .unwrap();
        let text = match wrap {
            "single" => "$$",
            "double" => "$$$$",
            _ => "",
        };
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                ("clipboard:write".into(), vec![json!(text)]),
                ("core:platform:paste-text".into(), vec![json!(text)])
            ]
        );
    }
}
#[tokio::test]
async fn callbacks_wrap_preview_back_errors_and_drafts() {
    let d = tempfile::tempdir().unwrap();
    let ctx = context(d.path());
    let mut s = json!({});
    let initial = launch(&ctx, "single", &mut s).await;
    let input = change(&ctx, &initial, 0, "x^2 + y^2", &mut s).await;
    assert_eq!(input["actions"][0]["title"], "Show Rendered Preview");
    let changed = change(&ctx, &input, 1, "double", &mut s).await;
    let latex = s["latex"].as_str().unwrap().to_owned();
    assert_eq!(
        changed["root"]["children"][2]["props"]["text"],
        format!("$${latex}$$")
    );
    for i in [0, 1] {
        assert!(changed["root"]["children"][i]["props"]
            .get("value")
            .is_none());
    }
    // Stable callbacks intentionally accept queued latest input; effect capabilities rotate.
    assert_eq!(field(&initial, 0), field(&changed, 0));
    let preview = adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":action(&changed,"Preview"),"formValues":map("x^2 + y^2","double")}),
        &mut s,
    )
    .await
    .unwrap();
    assert_eq!(
        preview["actions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["title"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "Copy Latex Code",
            "Edit Asciimath Expression",
            "Paste Latex Code"
        ]
    );
    assert_eq!(
        preview["root"]["props"]["markdown"],
        format!("\n$$\\huge {latex}$$\n")
    );
    assert_eq!(preview["root"]["props"]["nativeAsciiMath"]["latex"], latex);
    let back = preview["root"]["props"]["backActionId"].clone();
    let returned = adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":back}),
        &mut s,
    )
    .await
    .unwrap();
    assert_eq!(
        returned["root"]["children"][0]["props"]["defaultValue"],
        "x^2 + y^2"
    );
    assert_eq!(
        returned["root"]["children"][1]["props"]["defaultValue"],
        "double"
    );
    let error = change(&ctx, &returned, 0, "(x_1^)", &mut s).await;
    assert_eq!(s["latex"], "Error parsing expression");
    assert_eq!(s["error"], Value::Null);
    assert!(error["root"]["children"][2]["props"]["text"]
        .as_str()
        .unwrap()
        .contains("Error parsing expression"));
    let bounded = change(&ctx, &error, 0, &format!("{}x", "(".repeat(150)), &mut s).await;
    assert_eq!(s["latex"], "");
    assert!(bounded["root"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["props"]["title"] == "Conversion Error"));
}
#[tokio::test]
async fn capabilities_stale_cross_session_forged_clones_and_refresh() {
    let d = tempfile::tempdir().unwrap();
    let ctx = context(d.path());
    let (mut a, mut b) = (json!({}), json!({}));
    let va = launch(&ctx, "single", &mut a).await;
    let vb = launch(&ctx, "single", &mut b).await;
    for bad in [action(&vb, "Copy"), "copy".into(), "__nav_pop__".into()] {
        assert!(adapter(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":bad,"formValues":map("","single")}),
            &mut a
        )
        .await
        .is_err());
    }
    let old = action(&va, "Copy");
    let current = change(&ctx, &va, 0, "x", &mut a).await;
    assert!(adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":old,"formValues":map("x","single")}),
        &mut a
    )
    .await
    .is_err());
    let before = a.clone();
    for _ in 0..5 {
        assert_eq!(
            adapter(&ctx, "extension:refresh-session", &json!({}), &mut a)
                .await
                .unwrap(),
            current
        );
        assert_eq!(a, before);
    }
    let mut discarded = a.clone();
    let staged = change(&ctx, &current, 0, "discarded", &mut discarded).await;
    let committed = change(&ctx, &current, 0, "committed", &mut a).await;
    assert_ne!(action(&staged, "Copy"), action(&committed, "Copy"));
    assert!(adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":action(&staged,"Copy"),"formValues":map("committed","single")}),
        &mut a
    )
    .await
    .is_err());
    assert!(adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":action(&committed,"Copy"),"formValues":map("newer draft","single")}),
        &mut a
    )
    .await
    .is_err());
}
#[test]
fn acknowledgement_contract() {
    for v in [
        json!({"ok":true}),
        json!({"ok":true,"error":null}),
        json!({"ok":true,"success":true}),
    ] {
        assert!(ack(&Ok(v)));
    }
    for v in [
        Value::Null,
        json!(true),
        json!(false),
        json!([]),
        json!({}),
        json!({"success":true}),
        json!({"ok":"true"}),
        json!({"ok":false}),
        json!({"ok":true,"error":"private"}),
        json!({"ok":true,"error":false}),
        json!({"ok":true,"success":false}),
        json!({"ok":true,"success":null}),
    ] {
        assert!(!ack(&Ok(v)));
    }
    assert!(!ack(&Err("secret path".into())));
}
#[tokio::test]
async fn uncertain_effect_checkpoint_cancel_retry_and_ack_survives_later_error() {
    let d = tempfile::tempdir().unwrap();
    let ctx = context(d.path());
    let calls = Arc::new(Mutex::new(0));
    let log = calls.clone();
    ctx.set_router(Arc::new(move |_, _| {
        let mut n = log.lock().unwrap();
        *n += 1;
        let n = *n;
        Box::pin(async move {
            if n == 1 {
                std::future::pending::<CoreResult>().await
            } else {
                Ok(json!({"ok":true,"error":null}))
            }
        })
    }))
    .unwrap();
    let mut s = json!({});
    let v = launch(&ctx, "single", &mut s).await;
    let old = action(&v, "Paste");
    let p = json!({"actionId":old,"formValues":map("","single")});
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(10),
        adapter(&ctx, "extension:invoke-action", &p, &mut s)
    )
    .await
    .is_err());
    assert_eq!(s["intent"]["stage"], "pending");
    let refresh = adapter(&ctx, "extension:refresh-session", &json!({}), &mut s)
        .await
        .unwrap();
    assert_eq!(*calls.lock().unwrap(), 1);
    assert!(refresh["message"].as_str().unwrap().contains("uncertain"));
    assert!(adapter(&ctx, "extension:invoke-action", &p, &mut s)
        .await
        .is_err());
    assert_eq!(*calls.lock().unwrap(), 1);
    let retry = action(&refresh, "Retry Paste");
    let done = adapter(
        &ctx,
        "extension:invoke-action",
        &json!({"actionId":retry,"formValues":map("","single")}),
        &mut s,
    )
    .await
    .unwrap();
    assert_eq!(s["intent"]["stage"], "acknowledged");
    assert_eq!(*calls.lock().unwrap(), 2);
    assert!(done["message"].as_str().unwrap().contains("acknowledged"));
    let checkpoint = s.clone();
    assert!(adapter(&ctx, "bad-operation", &json!({}), &mut s)
        .await
        .is_err());
    assert_eq!(s, checkpoint);
}
#[tokio::test]
async fn private_router_errors_and_contradictory_ack_are_sanitized() {
    for response in [
        Err("/private/user/secret-token".into()),
        Ok(Value::Null),
        Ok(json!({"ok":true,"success":false})),
    ] {
        let d = tempfile::tempdir().unwrap();
        let ctx = context(d.path());
        ctx.set_router(Arc::new(move |_, _| {
            let r = response.clone();
            Box::pin(async move { r })
        }))
        .unwrap();
        let mut s = json!({});
        let v = launch(&ctx, "none", &mut s).await;
        let v = adapter(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":action(&v,"Copy"),"formValues":map("","none")}),
            &mut s,
        )
        .await
        .unwrap();
        assert!(!v.to_string().contains("private"));
        assert_eq!(s["intent"]["stage"], "uncertain");
    }
}
#[tokio::test]
async fn native_sessions_real_lifecycle_install_alias_refresh_dispose() {
    let d = tempfile::tempdir().unwrap();
    let ctx = context(d.path());
    let h = Harness::default();
    let launch = json!({"extensionId":ID,"commandName":CMD,"argumentValues":{}});
    assert_eq!(
        h.call(&ctx, "extension:run-command", launch.clone()).await["ok"],
        false
    );
    install(&ctx);
    let v = h.call(&ctx, "extension:run-command", launch).await;
    assert_eq!(v["ok"], true);
    let sid = v["sessionId"].clone();
    let v = h
        .call(
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":sid,"actionId":field(&v,0),"formValues":{"value":"sqrt(x)"}}),
        )
        .await;
    let refreshed = h
        .call(&ctx, "extension:refresh-session", json!({"sessionId":sid}))
        .await;
    assert_eq!(refreshed, v);
    for op in ["extension:search-text-changed", "extension:load-more"] {
        assert_eq!(
            h.call(&ctx, op, json!({"sessionId":sid,"searchText":"ignored"}))
                .await["mode"],
            "unchanged"
        );
    }
    assert_eq!(
        h.call(&ctx, "extension:dispose-session", json!({"sessionId":sid}))
            .await,
        true
    );
    assert_eq!(
        h.call(
            &ctx,
            "extension:invoke-action",
            json!({"sessionId":sid,"actionId":action(&v,"Copy")})
        )
        .await["ok"],
        false
    );
    let alias = h
        .call(
            &ctx,
            "extension:run-command",
            json!({"extensionId":"raycast.asciimath-to-latex-converter","commandName":CMD}),
        )
        .await;
    assert_eq!(alias["ok"], true);
    h.clear_slug().await;
    assert_eq!(
        h.call(
            &ctx,
            "extension:refresh-session",
            json!({"sessionId":alias["sessionId"]})
        )
        .await["ok"],
        false
    );
}
#[tokio::test]
async fn native_sessions_cancel_pending_effect_without_replay() {
    let d = tempfile::tempdir().unwrap();
    let ctx = context(d.path());
    install(&ctx);
    let h = Harness::default();
    let calls = Arc::new(Mutex::new(0));
    let log = calls.clone();
    let started = Arc::new(tokio::sync::Notify::new());
    let notify = started.clone();
    ctx.set_router(Arc::new(move |_, _| {
        *log.lock().unwrap() += 1;
        notify.notify_one();
        Box::pin(std::future::pending())
    }))
    .unwrap();
    let v = h
        .call(
            &ctx,
            "extension:run-command",
            json!({"extensionId":ID,"commandName":CMD}),
        )
        .await;
    let sid = v["sessionId"].clone();
    let hc = h.clone();
    let cc = ctx.clone();
    let body = json!({"sessionId":sid,"actionId":action(&v,"Paste"),"formValues":map("","single")});
    let pending = tokio::spawn(async move { hc.call(&cc, "extension:invoke-action", body).await });
    started.notified().await;
    assert_eq!(
        h.call(&ctx, "extension:refresh-session", json!({"sessionId":sid}))
            .await["mode"],
        "unchanged"
    );
    h.clear().await;
    let result = pending.await.unwrap();
    assert_eq!(result["ok"], false);
    assert_eq!(*calls.lock().unwrap(), 1);
}
#[tokio::test]
async fn actual_registered_runtime_catalog_and_registry_preserve_v1_and_command_preferences() {
    use beta_session_runtime::{catalog, registry_call};
    let d = tempfile::tempdir().unwrap();
    let ctx = context(d.path());
    assert!(
        std::env::var_os("SUPERCMD_EXTENSION_PATHS").is_none(),
        "Harness requires external extension paths unset"
    );
    let old = ctx
        .paths
        .data_dir
        .join("extensions/asciimath-to-latex-converter");
    std::fs::create_dir_all(&old).unwrap();
    let legacy = include_str!("../../../fixtures/original/package.json");
    std::fs::write(old.join("package.json"), legacy).unwrap();
    std::fs::write(old.join("preferences.json"),r#"{"unknown":"keep","commands":{"convert-asciimath-to-latex":{"defaultWrapLatex":"double"}}}"#).unwrap();
    let before = std::fs::read(old.join("preferences.json")).unwrap();
    install(&ctx);
    let items = catalog(&ctx).unwrap();
    assert_eq!(items.as_array().unwrap().len(), 12);
    let native = items
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == ID)
        .unwrap();
    assert_eq!(native["commands"][0]["runtime"], "rust");
    assert_eq!(native["installed"], true);
    assert_eq!(native["verification"][0]["state"], "not-run");
    let h = Harness::default();
    let v=h.call(&ctx,"extension:run-command",json!({"extensionId":"raycast.asciimath-to-latex-converter","commandName":CMD,"argumentValues":{"input":"ignored"}})).await;
    assert_eq!(v["root"]["children"][0]["props"]["defaultValue"], "");
    assert_eq!(v["root"]["children"][1]["props"]["defaultValue"], "double");
    assert_eq!(std::fs::read(old.join("preferences.json")).unwrap(), before);
    assert_eq!(
        std::fs::read_to_string(old.join("package.json")).unwrap(),
        legacy
    );
    let commands = registry_call(&ctx, "core:extensions:commands", &[]).unwrap();
    assert!(commands
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["extensionId"] == ID && c["cmdName"] == CMD));
    h.clear().await;
}
