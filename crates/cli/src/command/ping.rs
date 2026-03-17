use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;

pub struct PingCommand;

impl Command for PingCommand {
    fn name(&self) -> &str { "ping" }
    fn description(&self) -> &str { "Test RPC connectivity to the game" }
    fn usage(&self) -> &str { "ping" }

    fn execute(&self, _cmd: &ParsedCommand, ctx: &mut AppContext) {
        if !ctx.connected() {
            ctx.log_error("Not connected to any game client.");
            return;
        }

        let start = std::time::Instant::now();
        match ctx.game.ping() {
            Ok(true) => {
                let elapsed = start.elapsed();
                ctx.log_success(format!("Pong! Round-trip: {:.1}ms", elapsed.as_secs_f64() * 1000.0));
            }
            Ok(false) => {
                ctx.log_warn("Ping returned false - server may be unresponsive.");
            }
            Err(e) => {
                ctx.log_error(format!("Ping failed: {}", e));
            }
        }
    }
}
