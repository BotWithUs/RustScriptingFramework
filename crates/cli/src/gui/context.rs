use bot_api::game::GameApi;
use bot_core::rpc::client::RpcClient;
use bot_core::runtime::runtime::ScriptRuntime;
use crate::command::CommandRegistry;
use std::collections::VecDeque;
use std::sync::Arc;

const MAX_CONSOLE_LINES: usize = 5000;

/// Shared state passed to all GUI panels during rendering.
/// Equivalent to Java's CliContext.
pub struct AppContext {
    pub runtime: ScriptRuntime,
    pub game: Arc<dyn GameApi>,
    pub rpc_client: Option<Arc<RpcClient>>,
    pub pipe_name: String,
    pub scripts_dir: std::path::PathBuf,
    pub console_lines: VecDeque<ConsoleLine>,
    pub console_scroll_to_bottom: bool,
    pub commands: CommandRegistry,
    pub exit_requested: bool,

    // Console input state
    pub input_buffer: String,
    pub input_history: Vec<String>,
    pub history_index: Option<usize>,
    pub focus_input: bool,
}

#[derive(Clone)]
pub struct ConsoleLine {
    pub text: String,
    pub color: [f32; 4],
}

impl AppContext {
    pub fn new(
        runtime: ScriptRuntime,
        game: Arc<dyn GameApi>,
        rpc_client: Option<Arc<RpcClient>>,
        pipe_name: String,
        scripts_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            runtime,
            game,
            rpc_client,
            pipe_name,
            scripts_dir,
            console_lines: VecDeque::new(),
            console_scroll_to_bottom: false,
            commands: CommandRegistry::new(),
            exit_requested: false,
            input_buffer: String::new(),
            input_history: Vec::new(),
            history_index: None,
            focus_input: true,
        }
    }

    pub fn connected(&self) -> bool {
        self.rpc_client.is_some()
    }

    pub fn disconnect(&mut self) {
        self.runtime.stop_all();
        if let Some(rpc) = self.rpc_client.take() {
            rpc.close();
        }
    }

    pub fn log(&mut self, text: impl Into<String>, color: [f32; 4]) {
        self.console_lines.push_back(ConsoleLine {
            text: text.into(),
            color,
        });
        if self.console_lines.len() > MAX_CONSOLE_LINES {
            self.console_lines.pop_front();
        }
        self.console_scroll_to_bottom = true;
    }

    pub fn log_info(&mut self, text: impl Into<String>) {
        self.log(text, super::theme::TEXT);
    }

    pub fn log_warn(&mut self, text: impl Into<String>) {
        self.log(text, super::theme::YELLOW);
    }

    pub fn log_error(&mut self, text: impl Into<String>) {
        self.log(text, super::theme::RED);
    }

    pub fn log_success(&mut self, text: impl Into<String>) {
        self.log(text, super::theme::GREEN);
    }

    pub fn running_script_count(&self) -> usize {
        self.runtime.running_count()
    }

    pub fn total_script_count(&self) -> usize {
        self.runtime.total_count()
    }

    /// Execute a command line. Uses an unsafe self-reference trick because
    /// commands need &CommandRegistry and &mut AppContext simultaneously.
    pub fn execute_command(&mut self, line: &str) {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        // Echo the command
        self.log(format!("> {}", trimmed), super::theme::YELLOW);

        let parsed = crate::command::parse_command(&trimmed);
        // We need to borrow self.commands immutably while passing &mut self to execute.
        // Resolve the command index first, then execute.
        let cmd_index = self.commands.all().iter().position(|cmd| {
            let lower = parsed.name.to_lowercase();
            cmd.name() == lower || cmd.aliases().iter().any(|a| *a == lower)
        });

        match cmd_index {
            Some(idx) => {
                // Take the command out temporarily - commands are stateless so this is safe
                // Actually, we can't take from a Vec easily. Instead, use a raw pointer approach.
                // The safest approach: copy out what we need.
                let commands_ptr = &self.commands as *const CommandRegistry;
                // SAFETY: commands don't modify the registry, and we're single-threaded
                unsafe {
                    (*commands_ptr).all()[idx].execute(&parsed, self);
                }
            }
            None => {
                self.log_error(format!(
                    "Unknown command: '{}'. Type 'help' for available commands.",
                    parsed.name
                ));
            }
        }
    }
}
