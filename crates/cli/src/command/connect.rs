use crate::command::{Command, ParsedCommand};
use crate::gui::context::AppContext;
use crate::gui::theme;
use bot_api::GameApi;
use bot_core::rpc::client::RpcClient;
use bot_core::rpc::game_api::RpcGameApi;
use bot_core::rpc::pipe::PipeClient;
use bot_core::rpc::retry::RetryPolicy;
use std::sync::Arc;

pub struct ConnectCommand;

impl Command for ConnectCommand {
    fn name(&self) -> &str { "connect" }
    fn aliases(&self) -> &[&str] { &["conn"] }
    fn description(&self) -> &str { "Connect/disconnect/scan game pipes" }
    fn usage(&self) -> &str { "connect [<pipe>|scan [filter]|disconnect|status]" }

    fn execute(&self, cmd: &ParsedCommand, ctx: &mut AppContext) {
        let subcommand = cmd.arg(0).unwrap_or("status");

        match subcommand {
            "status" => show_status(ctx),
            "disconnect" | "dc" => disconnect(ctx),
            "scan" => {
                let filter = cmd.arg(1).unwrap_or("BotWithUs");
                scan_pipes(ctx, filter);
            }
            pipe_name => {
                do_connect(ctx, pipe_name);
            }
        }
    }
}

fn show_status(ctx: &mut AppContext) {
    if ctx.connected {
        ctx.log_info(format!("Connected to: {}", ctx.pipe_name));

        // Try to get account info
        match ctx.game.get_account_info() {
            Ok(info) => {
                let name = if info.display_name.is_empty() {
                    "(unknown)".to_string()
                } else {
                    info.display_name.clone()
                };
                ctx.log(format!("  Account: {}", name), theme::CYAN);
                ctx.log(format!("  World:   {}", info.world), theme::DIM_TEXT);
                ctx.log(
                    format!("  Members: {}", if info.members { "Yes" } else { "No" }),
                    theme::DIM_TEXT,
                );
                ctx.log(
                    format!("  Login State: {}", info.login_state),
                    theme::DIM_TEXT,
                );
            }
            Err(e) => {
                ctx.log_warn(format!("  Could not fetch account info: {}", e));
            }
        }

        // Show current world info
        match ctx.game.get_current_world() {
            Ok(world) => {
                if world.id > 0 {
                    ctx.log(
                        format!(
                            "  World {}: {} (pop: {})",
                            world.id, world.activity, world.population
                        ),
                        theme::DIM_TEXT,
                    );
                }
            }
            Err(_) => {}
        }
    } else {
        ctx.log_warn("Not connected. Use 'connect <pipe_name>' or 'connect scan' to find pipes.");
    }
}

fn disconnect(ctx: &mut AppContext) {
    if !ctx.connected {
        ctx.log_warn("Not connected.");
        return;
    }

    // Stop all scripts
    ctx.runtime.stop_all();

    // Close RPC
    if let Some(rpc) = ctx.rpc_client.take() {
        rpc.close();
    }
    ctx.connected = false;
    ctx.log_success(format!("Disconnected from '{}'.", ctx.pipe_name));
}

fn scan_pipes(ctx: &mut AppContext, filter: &str) {
    ctx.log_info(format!("Scanning for pipes matching '{}'...", filter));
    let pipes = PipeClient::scan_pipes(filter);
    if pipes.is_empty() {
        ctx.log_warn("No matching pipes found.");
    } else {
        ctx.log_success(format!("Found {} pipe(s):", pipes.len()));
        for (i, pipe) in pipes.iter().enumerate() {
            ctx.log(format!("  {:2}. {}", i + 1, pipe), theme::TEXT);
        }
        ctx.log(
            "Use 'connect <pipe_name>' to connect.",
            theme::DIM_TEXT,
        );
    }
}

fn do_connect(ctx: &mut AppContext, pipe_name: &str) {
    // Disconnect existing connection first
    if ctx.connected {
        ctx.runtime.stop_all();
        if let Some(rpc) = ctx.rpc_client.take() {
            rpc.close();
        }
        ctx.log_info(format!("Disconnected from '{}'.", ctx.pipe_name));
    }

    ctx.log_info(format!("Connecting to '{}'...", pipe_name));

    let mut pipe = PipeClient::new(pipe_name);
    match pipe.connect() {
        Ok(()) => {}
        Err(e) => {
            ctx.log_error(format!("Failed to connect pipe: {}", e));
            ctx.connected = false;
            return;
        }
    }

    let mut rpc = RpcClient::new(pipe);
    rpc.set_timeout(std::time::Duration::from_secs(10));
    rpc.set_retry_policy(RetryPolicy::default_policy());

    let rpc = Arc::new(rpc);
    rpc.start();

    let api = RpcGameApi::new(rpc.clone());

    // Test connectivity with a ping
    match api.ping() {
        Ok(true) => {
            ctx.log_success(format!("Connected to '{}' - ping OK!", pipe_name));
        }
        Ok(false) => {
            ctx.log_warn(format!("Connected to '{}' but ping returned false.", pipe_name));
        }
        Err(e) => {
            ctx.log_warn(format!("Connected to '{}' but ping failed: {}", pipe_name, e));
        }
    }

    ctx.pipe_name = pipe_name.to_string();
    ctx.game = Arc::new(api);
    ctx.rpc_client = Some(rpc);
    ctx.connected = true;
}
