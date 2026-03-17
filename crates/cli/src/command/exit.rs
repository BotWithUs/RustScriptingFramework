use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;

pub struct ExitCommand;

impl Command for ExitCommand {
    fn name(&self) -> &str { "exit" }
    fn aliases(&self) -> &[&str] { &["quit", "q"] }
    fn description(&self) -> &str { "Exit the application" }
    fn usage(&self) -> &str { "exit" }

    fn execute(&self, _cmd: &ParsedCommand, ctx: &mut AppContext) {
        ctx.log_info("Shutting down...");
        ctx.runtime.stop_all();
        if let Some(rpc) = ctx.rpc_client.take() {
            rpc.close();
        }
        ctx.exit_requested = true;
    }
}
