pub mod console;
pub mod scripts;
pub mod settings;

use crate::gui::context::AppContext;

/// Trait for tabbed GUI panels.
/// Equivalent to Java's GuiPanel interface.
pub trait GuiPanel {
    fn title(&self) -> &str;
    fn render(&mut self, ui: &imgui::Ui, ctx: &mut AppContext);
}
