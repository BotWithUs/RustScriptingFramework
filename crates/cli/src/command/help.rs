use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;
use crate::gui::theme;

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str { "help" }
    fn aliases(&self) -> &[&str] { &["h", "?"] }
    fn description(&self) -> &str { "Show available commands" }
    fn usage(&self) -> &str { "help [command]" }

    fn execute(&self, cmd: &ParsedCommand, ctx: &mut AppContext) {
        if let Some(cmd_name) = cmd.arg(0) {
            // Collect data first, then log (avoids borrow conflicts)
            let info = ctx.commands.resolve(cmd_name).map(|target| {
                let name = target.name().to_string();
                let desc = target.description().to_string();
                let usage = target.usage().to_string();
                let aliases: Vec<String> = target.aliases().iter().map(|a| a.to_string()).collect();
                (name, desc, usage, aliases)
            });

            match info {
                Some((name, desc, usage, aliases)) => {
                    ctx.log_info(format!("  {} - {}", name, desc));
                    ctx.log(format!("  Usage: {}", usage), theme::DIM_TEXT);
                    if !aliases.is_empty() {
                        ctx.log(format!("  Aliases: {}", aliases.join(", ")), theme::DIM_TEXT);
                    }
                }
                None => {
                    ctx.log_error(format!("Unknown command: '{}'", cmd_name));
                }
            }
            return;
        }

        // Collect all command info first
        let entries: Vec<(String, String, String)> = ctx.commands.all().iter().map(|command| {
            let name = command.name().to_string();
            let aliases = command.aliases();
            let alias_str = if aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", aliases.join(", "))
            };
            let desc = command.description().to_string();
            (name, alias_str, desc)
        }).collect();

        ctx.log_info("Available commands:");
        for (name, alias_str, desc) in entries {
            ctx.log(
                format!("  {:12}{} - {}", name, alias_str, desc),
                theme::TEXT,
            );
        }
    }
}
