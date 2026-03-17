use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;
use bot_core::loader::local::LocalScriptLoader;

pub struct ReloadCommand;

impl Command for ReloadCommand {
    fn name(&self) -> &str { "reload" }
    fn aliases(&self) -> &[&str] { &["rl"] }
    fn description(&self) -> &str { "Reload scripts from the scripts directory" }
    fn usage(&self) -> &str { "reload [--start]" }

    fn execute(&self, cmd: &ParsedCommand, ctx: &mut AppContext) {
        let auto_start = cmd.has_flag("start");

        if !ctx.scripts_dir.exists() {
            ctx.log_error(format!("Scripts directory {:?} does not exist.", ctx.scripts_dir));
            return;
        }

        // Stop all running scripts first
        ctx.runtime.stop_all();
        ctx.log_info("Stopped all running scripts.");

        let mut loader = LocalScriptLoader::new(&ctx.scripts_dir);
        match unsafe { loader.load_scripts() } {
            Ok(scripts) => {
                let count = scripts.len();
                for script in scripts {
                    ctx.runtime.register(script);
                }
                ctx.log_success(format!("Loaded {} script(s) from {:?}", count, ctx.scripts_dir));

                if auto_start && count > 0 {
                    let names: Vec<String> = ctx.runtime
                        .list_all()
                        .iter()
                        .map(|s| s.name.clone())
                        .collect();
                    for name in &names {
                        ctx.runtime.start(name);
                    }
                    ctx.log_success(format!("Auto-started {} script(s).", names.len()));
                }
            }
            Err(e) => {
                ctx.log_error(format!("Failed to load scripts: {}", e));
            }
        }
    }
}
