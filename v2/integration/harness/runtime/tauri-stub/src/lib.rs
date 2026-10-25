//! Presentation/OS stub ONLY: does not create a window, emit or touch the clipboard.
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit<S: serde::Serialize + Clone>(&self, event: &str, payload: S) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit<S: serde::Serialize + Clone>(&self, _: &str, _: S) -> Result<(), String> {
        Ok(())
    }
}
