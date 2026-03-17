/// Trait for scripts that provide a custom UI.
/// Equivalent to Java's ScriptUI interface.
/// The render method is called every frame on the UI thread.
pub trait ScriptUi: Send {
    fn render(&self);
}
