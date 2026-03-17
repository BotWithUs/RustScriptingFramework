use crate::gui::context::AppContext;
use crate::gui::panels::GuiPanel;
use crate::gui::theme;
use imgui::Ui;

/// Settings panel for configuration.
/// Equivalent to Java's SettingsPanel.
pub struct SettingsPanel;

impl SettingsPanel {
    pub fn new() -> Self {
        Self
    }
}

impl GuiPanel for SettingsPanel {
    fn title(&self) -> &str {
        "Settings"
    }

    fn render(&mut self, ui: &Ui, ctx: &mut AppContext) {
        ui.text("Connection");
        ui.separator();
        ui.spacing();

        ui.text_colored(theme::DIM_TEXT, "Pipe Name:");
        ui.same_line();
        ui.text(&ctx.pipe_name);

        ui.text_colored(theme::DIM_TEXT, "Status:");
        ui.same_line();
        if ctx.connected() {
            ui.text_colored(theme::GREEN, "Connected");
        } else {
            ui.text_colored(theme::RED, "Disconnected (offline mode)");
        }

        ui.spacing();
        ui.spacing();
        ui.text("Scripts");
        ui.separator();
        ui.spacing();

        ui.text_colored(theme::DIM_TEXT, "Scripts Directory:");
        ui.same_line();
        ui.text(ctx.scripts_dir.display().to_string());

        ui.text_colored(theme::DIM_TEXT, "Loaded:");
        ui.same_line();
        ui.text(format!("{}", ctx.total_script_count()));

        ui.text_colored(theme::DIM_TEXT, "Running:");
        ui.same_line();
        ui.text(format!("{}", ctx.running_script_count()));
    }
}
