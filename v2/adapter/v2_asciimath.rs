//! Compiled-in beta adapter. Sibling SDK; source-backed session state only.
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
use tezbar_asciimath_to_latex_converter::{convert, wrapped, MAX_INPUT_BYTES};

const COMMAND: &str = "convert-asciimath-to-latex";
const TITLE: &str = "AsciiMath to LaTeX Converter";
pub(super) fn commands() -> &'static [&'static str] {
    &[COMMAND]
}
fn capability() -> String {
    static NONCE: OnceLock<String> = OnceLock::new();
    static SEQ: AtomicU64 = AtomicU64::new(1);
    // Allocated outside cloned JSON: discarded staged states never reuse IDs.
    let serial = SEQ
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| x.checked_add(1))
        .expect("capability counter exhausted");
    format!(
        "am:{}:{serial}",
        NONCE.get_or_init(|| uuid::Uuid::new_v4().to_string())
    )
}
#[derive(Clone, Serialize, Deserialize)]
struct Intent {
    operation: String,
    text: String,
    stage: String,
    attempt: String,
}
#[derive(Clone, Serialize, Deserialize)]
struct Session {
    input: String,
    wrap: String,
    latex: String,
    error: Option<String>,
    preview: bool,
    input_action: String,
    wrap_action: String,
    actions: BTreeMap<String, String>,
    intent: Option<Intent>,
    revision: String,
}
fn valid_wrap(style: &str) -> bool {
    matches!(style, "none" | "single" | "double")
}
impl Session {
    fn new(wrap: &str) -> Self {
        let mut s = Self {
            input: String::new(),
            wrap: wrap.into(),
            latex: String::new(),
            error: None,
            preview: false,
            input_action: capability(),
            wrap_action: capability(),
            actions: BTreeMap::new(),
            intent: None,
            revision: String::new(),
        };
        s.rotate();
        s
    }
    fn rotate(&mut self) {
        self.revision = capability();
        self.actions.clear();
        for kind in if self.preview {
            vec!["copy", "edit", "paste"]
        } else if self.latex.is_empty() {
            vec!["copy", "paste"]
        } else {
            vec!["preview", "copy", "paste"]
        } {
            self.actions.insert(capability(), kind.into());
        }
    }
    fn id(&self, kind: &str) -> String {
        self.actions
            .iter()
            .find(|(_, v)| v.as_str() == kind)
            .expect("rendered capability")
            .0
            .clone()
    }
    fn update(&mut self, input: String, wrap: String) {
        self.input = input;
        self.wrap = wrap;
        match convert(&self.input) {
            Ok(v) => {
                self.latex = v;
                self.error = None;
            }
            Err(e) => {
                self.latex.clear();
                self.error = Some(e.into());
            }
        }
        self.rotate();
    }
    fn retry(&self, operation: &str) -> bool {
        self.intent
            .as_ref()
            .is_some_and(|i| i.operation == operation && i.stage != "acknowledged")
    }
    fn feedback(&self) -> Option<String> {
        self.intent.as_ref().map(|i| {
            if i.stage=="acknowledged" {format!("{} acknowledged.",if i.operation=="copy" {"Copy"} else {"Paste"})}
            else {format!("{} acknowledgement is uncertain. Retry deliberately; it may already have happened.",if i.operation=="copy" {"Copy"} else {"Paste"})}
        })
    }
    fn render(&self) -> Value {
        let mut actions = Vec::new();
        let make = |kind: &str, title: String, submit: bool| json!({"id":self.id(kind),"title":title,"kind":if submit {"submit-form"} else {"action"}});
        let copy = if self.preview {
            "Copy Latex Code"
        } else {
            "Copy LaTeX Code"
        };
        let paste = if self.preview {
            "Paste Latex Code"
        } else {
            "Paste LaTeX Code"
        };
        if self.preview {
            actions.push(make(
                "copy",
                format!("{}{copy}", if self.retry("copy") { "Retry " } else { "" }),
                false,
            ));
            actions.push(make("edit", "Edit Asciimath Expression".into(), false));
            actions.push(make(
                "paste",
                format!("{}{paste}", if self.retry("paste") { "Retry " } else { "" }),
                false,
            ));
        } else {
            if !self.latex.is_empty() {
                actions.push(make("preview", "Show Rendered Preview".into(), true));
            }
            actions.push(make(
                "copy",
                format!("{}{copy}", if self.retry("copy") { "Retry " } else { "" }),
                self.latex.is_empty(),
            ));
            actions.push(make(
                "paste",
                format!("{}{paste}", if self.retry("paste") { "Retry " } else { "" }),
                false,
            ));
        }
        let root = if self.preview {
            // Focused renderer prerequisite consumes this typed field with KaTeX.
            // Markdown is retained as source evidence; never claimed to render math alone.
            json!({"type":"Detail","props":{"backActionId":self.id("edit"),"markdown":format!("\n$$\\huge {}$$\n",self.latex),"nativeAsciiMath":{"latex":self.latex,"displayMode":true,"size":"huge"}}})
        } else {
            let mut children = vec![
                json!({"type":"Form.TextField","props":{"id":"input","title":"AsciiMath Expression","placeholder":"Enter math expression, e.g., x^2 + y^2 = z^2","defaultValue":self.input,"actionId":self.input_action}}),
                json!({"type":"Form.Dropdown","props":{"id":"wrapStyle","title":"LaTeX Wrapping Style","defaultValue":self.wrap,"actionId":self.wrap_action},"children":[
                    {"type":"Form.Dropdown.Item","props":{"value":"none","title":"None"}},
                    {"type":"Form.Dropdown.Item","props":{"value":"single","title":"Single ($...$)"}},
                    {"type":"Form.Dropdown.Item","props":{"value":"double","title":"Double ($$...$$)"}}]}),
                json!({"type":"Form.Description","props":{"title":"LaTeX Output","text":if self.latex.is_empty() {"No LaTeX output available.".into()} else {wrapped(&self.latex,&self.wrap)}}}),
            ];
            if let Some(e) = &self.error {
                children.push(json!({"type":"Form.Description","props":{"title":"Conversion Error","text":format!("An error occurred: {e}")}}));
            }
            if let Some(message) = self.feedback() {
                children.push(json!({"type":"Form.Description","props":{"title":"Action status","text":message}}));
            }
            json!({"type":"Form","props":{"nativeAsciiMath":true},"children":children})
        };
        let mut root = root;
        root["props"]["nativeAsciiMathRevision"] = json!(self.revision);
        let mut view = sdk::view(
            if self.preview { "LaTeX Preview" } else { TITLE },
            root,
            actions,
        );
        if let Some(message) = self.feedback() {
            view["message"] = json!(message);
        }
        view
    }
}
fn save(state: &mut Value, s: &Session) -> Result<(), String> {
    *state = serde_json::to_value(s).map_err(|_| "Invalid converter state")?;
    Ok(())
}
fn read(state: &Value) -> Result<Session, String> {
    serde_json::from_value(state.clone())
        .map_err(|_| "Converter session is not initialized; reopen it".into())
}
/// Genuine, positive, noncontradictory object acknowledgement only.
pub fn acknowledged(result: &CoreResult) -> bool {
    result
        .as_ref()
        .ok()
        .and_then(Value::as_object)
        .is_some_and(|v| {
            v.get("ok") == Some(&json!(true))
                && v.get("error").is_none_or(Value::is_null)
                && v.get("success").is_none_or(|v| v == &json!(true))
        })
}
fn complete_form(payload: &Value, s: &Session) -> Result<(), String> {
    let map = payload["formValues"]
        .as_object()
        .ok_or("Expression is not synchronized; wait for conversion and choose the action again")?;
    if map.get("input").and_then(Value::as_str) != Some(&s.input)
        || map.get("wrapStyle").and_then(Value::as_str) != Some(&s.wrap)
    {
        return Err("Expression changed; wait for conversion and choose the action again".into());
    }
    Ok(())
}
pub(super) async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command != COMMAND {
        return Err("Unknown AsciiMath command".into());
    }
    match operation {
        "extension:run-command" => {
            let pref = payload["preferences"].get("defaultWrapLatex");
            let wrap = match pref {
                None => "single",
                Some(Value::String(s)) if valid_wrap(s) => s,
                _ => return Err("Default LaTeX Wrapping must be none, single, or double".into()),
            };
            let s = Session::new(wrap);
            let view = s.render();
            save(state, &s)?;
            Ok(view)
        }
        "extension:dispose-session" => {
            *state = Value::Null;
            Ok(sdk::unchanged())
        }
        "extension:refresh-session" => Ok(read(state)?.render()),
        "extension:load-more" | "extension:search-text-changed" => {
            read(state)?;
            Ok(sdk::unchanged())
        }
        "extension:invoke-action" => {
            let mut s = read(state)?;
            let id = payload["actionId"]
                .as_str()
                .ok_or("Missing action capability")?;
            if !s.preview && (id == s.input_action || id == s.wrap_action) {
                let v = payload["formValues"]["value"]
                    .as_str()
                    .ok_or("Invalid form callback value")?;
                if id == s.input_action {
                    // Oversized input is retained only within this bounded state budget.
                    if v.len() > MAX_INPUT_BYTES {
                        return Err("Expression exceeds 8192 UTF-8 bytes".into());
                    }
                    s.update(v.into(), s.wrap.clone());
                } else {
                    if !valid_wrap(v) {
                        return Err("Invalid wrapping style".into());
                    }
                    s.update(s.input.clone(), v.into());
                }
                let view = s.render();
                save(state, &s)?;
                return Ok(view);
            }
            let kind = s
                .actions
                .get(id)
                .cloned()
                .ok_or("Action expired; use the current view")?;
            if !s.preview {
                complete_form(payload, &s)?;
            }
            match kind.as_str() {
                "preview" => {
                    if s.latex.is_empty() {
                        return Err("No LaTeX output available".into());
                    }
                    s.preview = true;
                    s.rotate();
                    let view = s.render();
                    save(state, &s)?;
                    Ok(view)
                }
                "edit" => {
                    s.preview = false;
                    s.rotate();
                    let view = s.render();
                    save(state, &s)?;
                    Ok(view)
                }
                "copy" | "paste" => {
                    let text = wrapped(&s.latex, &s.wrap);
                    s.intent = Some(Intent {
                        operation: kind.clone(),
                        text: text.clone(),
                        stage: "pending".into(),
                        attempt: capability(),
                    });
                    s.rotate();
                    save(state, &s)?; // checkpoint BEFORE the cancellable OS boundary
                    let result = if kind == "copy" {
                        sdk::copy(ctx, &text).await
                    } else {
                        ctx.request("core:platform:paste-text", vec![json!(text)])
                            .await
                    };
                    if let Some(intent) = &mut s.intent {
                        intent.stage = if acknowledged(&result) {
                            "acknowledged"
                        } else {
                            "uncertain"
                        }
                        .into();
                    }
                    s.rotate();
                    save(state, &s)?; // acknowledgement survives render/later failures
                    Ok(s.render())
                }
                _ => Err("Unknown action".into()),
            }
        }
        _ => Err("Unsupported AsciiMath operation".into()),
    }
}
