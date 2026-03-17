use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;
use crate::gui::theme;

pub struct MetricsCommand;

impl Command for MetricsCommand {
    fn name(&self) -> &str { "metrics" }
    fn description(&self) -> &str { "Show RPC call metrics" }
    fn usage(&self) -> &str { "metrics [reset]" }

    fn execute(&self, cmd: &ParsedCommand, ctx: &mut AppContext) {
        if !ctx.connected {
            ctx.log_error("Not connected - no metrics available.");
            return;
        }

        if cmd.arg(0) == Some("reset") {
            if let Some(rpc) = &ctx.rpc_client {
                rpc.metrics().reset();
                ctx.log_success("RPC metrics reset.");
            }
            return;
        }

        if let Some(rpc) = &ctx.rpc_client {
            let snapshot = rpc.metrics().snapshot();
            if snapshot.is_empty() {
                ctx.log(
                    "No RPC calls recorded yet.",
                    theme::DIM_TEXT,
                );
                return;
            }

            ctx.log_info(format!("RPC Metrics ({} methods):", snapshot.len()));
            ctx.log(
                format!("  {:30} {:>8} {:>10} {:>8}", "Method", "Calls", "Avg (ms)", "Errors"),
                theme::DIM_TEXT,
            );

            let mut entries: Vec<_> = snapshot.into_iter().collect();
            entries.sort_by(|a, b| b.1.call_count.cmp(&a.1.call_count));

            for (method, stats) in &entries {
                let avg_ms = if stats.call_count > 0 {
                    stats.total_time.as_secs_f64() * 1000.0 / stats.call_count as f64
                } else {
                    0.0
                };
                let color = if stats.error_count > 0 { theme::RED } else { theme::TEXT };
                ctx.log(
                    format!(
                        "  {:30} {:>8} {:>9.1} {:>8}",
                        method, stats.call_count, avg_ms, stats.error_count
                    ),
                    color,
                );
            }
        }
    }
}
