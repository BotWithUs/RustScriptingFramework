use crate::gui::context::AppContext;
use crate::gui::panels::GuiPanel;
use crate::gui::theme;
use imgui::Ui;

const BANNER: &str = r"
       ____        _  __        ___ _   _     _   _
      | __ )  ___ | |_\ \      / (_) |_| |__ | | | |___
      |  _ \ / _ \| __|\ \ /\ / /| | __| '_ \| | | / __|
      | |_) | (_) | |_  \ V  V / | | |_| | | | |_| \__ \
      |____/ \___/ \__|  \_/\_/  |_|\__|_| |_|\___/|___/
              Rust Scripting Framework

         Type 'help' for available commands.
";

/// Console output panel with scrolling text and command input.
/// Equivalent to Java's ConsolePanel.
pub struct ConsolePanel {
    banner_shown: bool,
}

impl ConsolePanel {
    pub fn new() -> Self {
        Self {
            banner_shown: false,
        }
    }
}

impl GuiPanel for ConsolePanel {
    fn title(&self) -> &str {
        "Console"
    }

    fn render(&mut self, ui: &Ui, ctx: &mut AppContext) {
        if !self.banner_shown {
            ctx.log(BANNER.to_string(), theme::CYAN);
            self.banner_shown = true;
        }

        let input_bar_height = ui.frame_height_with_spacing() * 2.0 + 8.0;
        let output_height = ui.content_region_avail()[1] - input_bar_height;

        // === Output area ===
        if let Some(_child) = ui
            .child_window("##console_output")
            .size([0.0, output_height])
            .border(false)
            .begin()
        {
            for line in &ctx.console_lines {
                ui.text_colored(line.color, &line.text);
            }

            if ctx.console_scroll_to_bottom {
                ui.set_scroll_here_y_with_ratio(1.0);
                ctx.console_scroll_to_bottom = false;
            }
        }

        // === Input bar ===
        ui.separator();

        // Prompt
        if ctx.connected() {
            ui.text_colored(theme::GREEN, "\u{25CF}");
            ui.same_line_with_spacing(0.0, 4.0);
            ui.text("bot:");
            ui.same_line_with_spacing(0.0, 0.0);
            ui.text_colored(theme::CYAN, &ctx.pipe_name);
            ui.same_line_with_spacing(0.0, 0.0);
            ui.text("> ");
        } else {
            ui.text_colored(theme::RED, "\u{25CF}");
            ui.same_line_with_spacing(0.0, 4.0);
            ui.text("bot> ");
        }

        ui.same_line();

        // Input text field
        let input_width = ui.content_region_avail()[0];
        ui.set_next_item_width(input_width);

        if ctx.focus_input {
            ui.set_keyboard_focus_here();
            ctx.focus_input = false;
        }

        let enter_pressed = ui
            .input_text("##console_input", &mut ctx.input_buffer)
            .enter_returns_true(true)
            .build();

        if enter_pressed {
            let line = ctx.input_buffer.clone();
            let trimmed = line.trim().to_string();

            if !trimmed.is_empty() {
                ctx.input_history.push(trimmed.clone());
                ctx.history_index = None;
                ctx.execute_command(&trimmed);
            }

            ctx.input_buffer.clear();
            ctx.focus_input = true;
        }

        // History navigation with arrow keys when input is focused
        if ui.is_item_focused() && !ctx.input_history.is_empty() {
            if ui.is_key_pressed(imgui::Key::UpArrow) {
                let new_idx = match ctx.history_index {
                    None => ctx.input_history.len() - 1,
                    Some(i) if i > 0 => i - 1,
                    Some(i) => i,
                };
                ctx.history_index = Some(new_idx);
                ctx.input_buffer = ctx.input_history[new_idx].clone();
            }
            if ui.is_key_pressed(imgui::Key::DownArrow) {
                match ctx.history_index {
                    Some(i) if i + 1 < ctx.input_history.len() => {
                        ctx.history_index = Some(i + 1);
                        ctx.input_buffer = ctx.input_history[i + 1].clone();
                    }
                    Some(_) => {
                        ctx.history_index = None;
                        ctx.input_buffer.clear();
                    }
                    None => {}
                }
            }
        }
    }
}
