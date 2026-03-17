use crate::gui::context::AppContext;
use crate::gui::theme;
use imgui::Ui;

/// Fixed status bar at the bottom of the window.
/// Equivalent to Java's StatusBar.
pub struct StatusBar;

impl StatusBar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, ui: &Ui, ctx: &AppContext) {
        ui.separator();

        // Connection indicator dot
        if ctx.connected {
            ui.text_colored(theme::GREEN, "\u{25CF}");
        } else {
            ui.text_colored(theme::RED, "\u{25CF}");
        }

        ui.same_line_with_spacing(0.0, 6.0);

        // Connection name
        if ctx.connected {
            ui.text_colored(theme::CYAN, &ctx.pipe_name);
        } else {
            ui.text_colored(theme::DIM_TEXT, "disconnected");
        }

        ui.same_line_with_spacing(0.0, 12.0);
        ui.text_colored(theme::DIM_TEXT, "|");
        ui.same_line_with_spacing(0.0, 12.0);

        // Running scripts count
        let running = ctx.running_script_count();
        if running > 0 {
            let label = if running == 1 {
                "1 script running".to_string()
            } else {
                format!("{} scripts running", running)
            };
            ui.text_colored(theme::GREEN, &label);
        } else {
            ui.text_colored(theme::DIM_TEXT, "no scripts");
        }

        ui.same_line_with_spacing(0.0, 12.0);
        ui.text_colored(theme::DIM_TEXT, "|");
        ui.same_line_with_spacing(0.0, 12.0);

        // Total loaded
        let total = ctx.total_script_count();
        ui.text(format!("{} loaded", total));
    }
}
