use crate::gui::context::AppContext;
use crate::gui::panels::GuiPanel;
use crate::gui::theme;
use bot_core::loader::local::LocalScriptLoader;

/// Scripts management panel - list scripts, start/stop, reload.
/// Equivalent to Java's ScriptsPanel.
pub struct ScriptsPanel;

impl ScriptsPanel {
    pub fn new() -> Self {
        Self
    }
}

impl GuiPanel for ScriptsPanel {
    fn title(&self) -> &str {
        "Scripts"
    }

    fn render(&mut self, ui: &imgui::Ui, ctx: &mut AppContext) {
        // Top controls
        if ui.button("Reload Scripts") {
            reload_scripts(ctx);
        }

        ui.spacing();
        ui.separator();
        ui.spacing();

        if !ctx.connected {
            ui.text_colored(theme::DIM_TEXT, "Offline mode - using stub API.");
            ui.spacing();
        }

        let scripts = ctx.runtime.list_all();
        if scripts.is_empty() {
            ui.text_colored(
                theme::DIM_TEXT,
                "No scripts loaded. Click 'Reload Scripts' or place .dll files in the scripts directory.",
            );
            return;
        }

        // Scripts table
        if let Some(_table) = ui.begin_table_with_flags(
            "scriptsTable",
            6,
            imgui::TableFlags::BORDERS
                | imgui::TableFlags::ROW_BG
                | imgui::TableFlags::SIZING_STRETCH_PROP,
        ) {
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: "#",
                init_width_or_weight: 0.3,
                ..Default::default()
            });
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: "Name",
                init_width_or_weight: 1.5,
                ..Default::default()
            });
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: "Author",
                init_width_or_weight: 0.8,
                ..Default::default()
            });
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: "Version",
                init_width_or_weight: 0.5,
                ..Default::default()
            });
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: "Status",
                init_width_or_weight: 0.6,
                ..Default::default()
            });
            ui.table_setup_column_with(imgui::TableColumnSetup {
                name: "Actions",
                init_width_or_weight: 1.2,
                ..Default::default()
            });
            ui.table_headers_row();

            for (i, script) in scripts.iter().enumerate() {
                ui.table_next_row();

                ui.table_set_column_index(0);
                ui.text(format!("{}", i + 1));

                ui.table_set_column_index(1);
                ui.text(&script.name);

                ui.table_set_column_index(2);
                if script.author.is_empty() {
                    ui.text_colored(theme::DIM_TEXT, "-");
                } else {
                    ui.text(&script.author);
                }

                ui.table_set_column_index(3);
                ui.text(&script.version);

                ui.table_set_column_index(4);
                if script.running {
                    ui.text_colored(theme::GREEN, "RUNNING");
                } else {
                    ui.text_colored(theme::RED, "STOPPED");
                }

                ui.table_set_column_index(5);
                let _id = ui.push_id_usize(i);
                if script.running {
                    if ui.small_button("Stop") {
                        ctx.runtime.stop(&script.name);
                        ctx.log_warn(format!("Stopped script '{}'", script.name));
                    }
                } else {
                    if ui.small_button("Start") {
                        if ctx.runtime.start(&script.name) {
                            ctx.log_success(format!("Started script '{}'", script.name));
                        } else {
                            ctx.log_error(format!("Failed to start script '{}'", script.name));
                        }
                    }
                }
            }
        }
    }
}

fn reload_scripts(ctx: &mut AppContext) {
    if !ctx.scripts_dir.exists() {
        ctx.log_error(format!("Scripts directory {:?} does not exist", ctx.scripts_dir));
        return;
    }

    // Stop all running scripts first
    ctx.runtime.stop_all();

    let mut loader = LocalScriptLoader::new(&ctx.scripts_dir);
    match unsafe { loader.load_scripts() } {
        Ok(scripts) => {
            let count = scripts.len();
            for script in scripts {
                ctx.runtime.register(script);
            }
            ctx.log_success(format!("Reloaded {} script(s)", count));
        }
        Err(e) => {
            ctx.log_error(format!("Failed to load scripts: {}", e));
        }
    }
}
