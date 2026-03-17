use bot_api::config::{ConfigField, ScriptConfig};
use bot_api::context::ScriptContext;
use bot_api::events::{ActionExecutedEvent, GameEvent};
use bot_api::script::{BotScript, LoopAction, ScriptManifest};

/// Example script demonstrating the scripting framework.
/// Equivalent to Java's ExampleScript.
#[bot_macros::export_script]
#[derive(Default)]
pub struct ExampleScript {
    loop_count: u32,
    max_loops: u32,
    message: String,
}

impl BotScript for ExampleScript {
    fn manifest(&self) -> ScriptManifest {
        bot_macros::script_manifest!(
            name: "Example Script",
            version: "1.0.0",
            author: "BotWithUs",
            description: "An example script demonstrating the framework"
        )
    }

    fn on_start(&mut self, ctx: &ScriptContext) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ExampleScript] Started! Message: {}", self.message);

        // Subscribe to action events
        ctx.event_bus().subscribe(
            "ActionExecuted",
            Box::new(|event: &dyn GameEvent| {
                if let Some(action) = event.as_any().downcast_ref::<ActionExecutedEvent>() {
                    println!(
                        "[ExampleScript] Action executed: id={}, entity={}",
                        action.action_id, action.entity_id
                    );
                }
            }),
        );

        // Store something in shared state
        ctx.shared_state()
            .put("example.started", serde_json::json!(true));

        Ok(())
    }

    fn on_loop(&mut self) -> Result<LoopAction, Box<dyn std::error::Error>> {
        self.loop_count += 1;
        println!(
            "[ExampleScript] Loop {} of {} - {}",
            self.loop_count, self.max_loops, self.message
        );

        if self.max_loops > 0 && self.loop_count >= self.max_loops {
            return Ok(LoopAction::Stop);
        }

        Ok(LoopAction::Sleep(1000))
    }

    fn on_stop(&mut self) {
        println!(
            "[ExampleScript] Stopped after {} loops",
            self.loop_count
        );
    }

    fn config_fields(&self) -> Vec<ConfigField> {
        vec![
            ConfigField::int("max_loops", "Maximum Loops", 0, 10000, 100),
            ConfigField::string("message", "Status Message", "Hello from Rust!"),
            ConfigField::bool("verbose", "Verbose Logging", false),
            ConfigField::choice(
                "mode",
                "Operating Mode",
                vec!["Normal".into(), "Fast".into(), "Careful".into()],
                "Normal",
            ),
        ]
    }

    fn on_config_update(&mut self, config: &ScriptConfig) {
        self.max_loops = config.get_int("max_loops", 100) as u32;
        self.message = config.get_string("message", "Hello from Rust!");
        println!(
            "[ExampleScript] Config updated: max_loops={}, message={}",
            self.max_loops, self.message
        );
    }
}
