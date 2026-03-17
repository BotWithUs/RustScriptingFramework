pub mod clear;
pub mod connect;
pub mod exit;
pub mod help;
pub mod metrics;
pub mod ping;
pub mod reload;
pub mod scripts;

use crate::gui::context::AppContext;
use std::collections::HashMap;

/// Parsed command input with positional args and named flags.
pub struct ParsedCommand {
    pub name: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,
}

impl ParsedCommand {
    pub fn arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.contains_key(name)
    }
}

/// Parse a raw input line into a ParsedCommand.
pub fn parse_command(input: &str) -> ParsedCommand {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let name = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();
    let mut args = Vec::new();
    let mut flags = HashMap::new();

    for part in parts.iter().skip(1) {
        if let Some(flag) = part.strip_prefix("--") {
            if let Some((key, value)) = flag.split_once('=') {
                flags.insert(key.to_string(), value.to_string());
            } else {
                flags.insert(flag.to_string(), String::new());
            }
        } else {
            args.push(part.to_string());
        }
    }

    ParsedCommand { name, args, flags }
}

/// Command trait - all CLI commands implement this.
pub trait Command {
    fn name(&self) -> &str;
    fn aliases(&self) -> &[&str] { &[] }
    fn description(&self) -> &str;
    fn usage(&self) -> &str;
    fn execute(&self, cmd: &ParsedCommand, ctx: &mut AppContext);
}

/// Registry of all available commands.
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut reg = Self { commands: Vec::new() };
        reg.register(Box::new(help::HelpCommand));
        reg.register(Box::new(ping::PingCommand));
        reg.register(Box::new(scripts::ScriptsCommand));
        reg.register(Box::new(reload::ReloadCommand));
        reg.register(Box::new(connect::ConnectCommand));
        reg.register(Box::new(metrics::MetricsCommand));
        reg.register(Box::new(clear::ClearCommand));
        reg.register(Box::new(exit::ExitCommand));
        reg
    }

    fn register(&mut self, cmd: Box<dyn Command>) {
        self.commands.push(cmd);
    }

    pub fn resolve(&self, name: &str) -> Option<&dyn Command> {
        let lower = name.to_lowercase();
        self.commands.iter().find(|cmd| {
            cmd.name() == lower || cmd.aliases().iter().any(|a| *a == lower)
        }).map(|b| b.as_ref())
    }

    pub fn all(&self) -> &[Box<dyn Command>] {
        &self.commands
    }

    pub fn execute_line(&self, line: &str, ctx: &mut AppContext) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        // Echo the command
        ctx.log(format!("> {}", trimmed), crate::gui::theme::YELLOW);

        let parsed = parse_command(trimmed);
        match self.resolve(&parsed.name) {
            Some(cmd) => cmd.execute(&parsed, ctx),
            None => ctx.log_error(format!(
                "Unknown command: '{}'. Type 'help' for available commands.",
                parsed.name
            )),
        }
    }
}
