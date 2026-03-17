use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;
use crate::gui::theme;

pub struct ScriptsCommand;

impl Command for ScriptsCommand {
    fn name(&self) -> &str { "scripts" }
    fn aliases(&self) -> &[&str] { &["s"] }
    fn description(&self) -> &str { "List, start, or stop scripts" }
    fn usage(&self) -> &str { "scripts [list|start <name>|stop <name>|restart <name>|info <name>]" }

    fn execute(&self, cmd: &ParsedCommand, ctx: &mut AppContext) {
        let subcommand = cmd.arg(0).unwrap_or("list");

        match subcommand {
            "list" | "ls" => list_scripts(ctx),
            "start" => {
                if let Some(name) = cmd.arg(1) {
                    start_script(ctx, name);
                } else {
                    ctx.log_error("Usage: scripts start <name>");
                }
            }
            "stop" => {
                if let Some(name) = cmd.arg(1) {
                    stop_script(ctx, name);
                } else {
                    ctx.log_error("Usage: scripts stop <name>");
                }
            }
            "restart" => {
                if let Some(name) = cmd.arg(1) {
                    stop_script(ctx, name);
                    start_script(ctx, name);
                } else {
                    ctx.log_error("Usage: scripts restart <name>");
                }
            }
            "info" => {
                if let Some(name) = cmd.arg(1) {
                    script_info(ctx, name);
                } else {
                    ctx.log_error("Usage: scripts info <name>");
                }
            }
            "status" => {
                let running = ctx.running_script_count();
                let total = ctx.total_script_count();
                ctx.log_info(format!("{} script(s) loaded, {} running.", total, running));
            }
            _ => {
                ctx.log_error(format!("Unknown subcommand: '{}'. Use: list, start, stop, restart, info, status", subcommand));
            }
        }
    }
}

fn list_scripts(ctx: &mut AppContext) {
    let scripts = ctx.runtime.list_all();
    if scripts.is_empty() {
        ctx.log(
            "No scripts loaded. Use 'reload' to discover scripts.",
            theme::DIM_TEXT,
        );
        return;
    }

    ctx.log_info(format!("{} script(s):", scripts.len()));
    for (i, s) in scripts.iter().enumerate() {
        let status = if s.running { "RUNNING" } else { "STOPPED" };
        let status_color = if s.running { theme::GREEN } else { theme::RED };
        let author = if s.author.is_empty() { "-" } else { &s.author };
        // Log each part
        ctx.log(
            format!(
                "  {:2}. {:20} v{:8} by {:12} [{}]",
                i + 1,
                s.name,
                s.version,
                author,
                status
            ),
            status_color,
        );
    }
}

fn start_script(ctx: &mut AppContext, name: &str) {
    if ctx.runtime.start(name) {
        ctx.log_success(format!("Started script '{}'", name));
    } else {
        ctx.log_error(format!("Failed to start '{}' - not found or already running.", name));
    }
}

fn stop_script(ctx: &mut AppContext, name: &str) {
    if ctx.runtime.stop(name) {
        ctx.log_success(format!("Stopped script '{}'", name));
    } else {
        ctx.log_error(format!("Failed to stop '{}' - not found or not running.", name));
    }
}

fn script_info(ctx: &mut AppContext, name: &str) {
    match ctx.runtime.get_info(name) {
        Some(info) => {
            ctx.log_info(format!("Script: {}", info.name));
            ctx.log(format!("  Version: {}", info.version), theme::DIM_TEXT);
            ctx.log(format!("  Author:  {}", if info.author.is_empty() { "-" } else { &info.author }), theme::DIM_TEXT);
            ctx.log(format!("  Description: {}", if info.description.is_empty() { "-" } else { &info.description }), theme::DIM_TEXT);
            let status = if info.running { "RUNNING" } else { "STOPPED" };
            ctx.log(format!("  Status:  {}", status), if info.running { theme::GREEN } else { theme::RED });
        }
        None => {
            ctx.log_error(format!("Script '{}' not found.", name));
        }
    }
}
