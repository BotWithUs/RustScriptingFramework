use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;

pub struct ClearCommand;

impl Command for ClearCommand {
    fn name(&self) -> &str { "clear" }
    fn aliases(&self) -> &[&str] { &["cls"] }
    fn description(&self) -> &str { "Clear the console output" }
    fn usage(&self) -> &str { "clear" }

    fn execute(&self, _cmd: &ParsedCommand, ctx: &mut AppContext) {
        ctx.console_lines.clear();
    }
}
