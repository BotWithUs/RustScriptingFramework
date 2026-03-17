# BotWithUs - Rust Scripting Framework

A Rust port of the JBotWithUsV2 game scripting framework. Provides a plugin-based architecture for loading and managing native scripts that interact with a game client over named pipes (RPC).

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────────┐
│  bot-cli    │────▶│  bot-core    │────▶│  bot-api (traits) │
│  (ImGui GUI)│     │  (runtime,   │     │  BotScript,       │
│             │     │   RPC, loader)│     │  GameApi, etc.    │
└─────────────┘     └──────────────┘     └──────────────────┘
                          │
                    ┌─────┴──────┐
                    │  Scripts   │
                    │  (.dll/.so)│
                    └────────────┘
```

### Crates

| Crate | Description |
|-------|-------------|
| **bot-api** | Core traits and interfaces — `BotScript`, `GameApi`, `EventBus`, `ScriptContext`, etc. |
| **bot-core** | Runtime implementation — script loading (`libloading`), RPC client (msgpack over named pipes), event/message buses, config store |
| **bot-cli** | Interactive application with ImGui-based GUI, command system, and connection management (binary: `botwithus`) |
| **bot-macros** | Proc macros — `#[export_script]` for FFI export, `script_manifest!()` for metadata |
| **example-script** | Sample script plugin demonstrating the framework |

## Building

```bash
cargo build --release
```

The main binary is `botwithus`. Scripts compile as dynamic libraries (`cdylib`).

## Usage

```bash
# Launch GUI (default)
botwithus --pipe <pipe_name> --scripts ./scripts

# Launch in headless mode
botwithus --pipe <pipe_name> --scripts ./scripts --headless

# Auto-start all scripts on launch
botwithus --pipe <pipe_name> --scripts ./scripts --auto-start
```

### GUI

The GUI has three panels:

- **Console** — Command input with history (up/down arrows), scrollable output, connection status prompt
- **Scripts** — Table view of loaded scripts with start/stop/restart controls
- **Settings** — Connection info and script directory configuration

A status bar at the bottom shows connection state, pipe name, and running script counts.

### Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `help [cmd]` | `h`, `?` | Show available commands or detailed help |
| `connect <pipe\|scan\|disconnect\|status>` | `conn` | Manage game pipe connections |
| `ping` | | Test RPC connectivity with round-trip latency |
| `scripts <list\|start\|stop\|restart\|info> [name]` | `s` | Manage loaded scripts |
| `reload [--start]` | | Reload scripts from disk, optionally auto-start |
| `metrics [reset]` | | Show or reset RPC call metrics |
| `clear` | `cls` | Clear console output |
| `exit` | `quit`, `q` | Stop scripts and exit |

## Writing Scripts

Scripts are native dynamic libraries that implement the `BotScript` trait:

```rust
use bot_api::script::*;
use bot_api::context::ScriptContext;
use bot_macros::export_script;

#[export_script]
pub struct MyScript;

impl BotScript for MyScript {
    fn manifest(&self) -> ScriptManifest {
        script_manifest!(
            name: "MyScript",
            version: "1.0.0",
            author: "You",
            description: "Does something useful"
        )
    }

    fn on_start(&mut self, ctx: &ScriptContext) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize
        Ok(())
    }

    fn on_loop(&mut self) -> Result<LoopAction, Box<dyn std::error::Error>> {
        // Main logic runs each tick
        Ok(LoopAction::Continue)
    }

    fn on_stop(&mut self) {
        // Cleanup
    }
}
```

Build as a cdylib and place the resulting `.dll`/`.so` in the scripts directory:

```toml
[lib]
crate-type = ["cdylib"]
```

### Script Lifecycle

1. **Load** — Runtime discovers and loads the dynamic library
2. **Register** — Script is added to the registry (visible in GUI)
3. **Start** — `on_start()` called with `ScriptContext` providing access to game API, events, messaging, and shared state
4. **Loop** — `on_loop()` called repeatedly; return `LoopAction::Continue` or `LoopAction::Stop`
5. **Stop** — `on_stop()` called for cleanup

### Available Services via ScriptContext

- **GameApi** — Full game interaction (player, inventory, NPCs, world, movement, combat, etc.)
- **EventBus** — Subscribe to game events (chat, ticks, login state, key input, var changes)
- **MessageBus** — Inter-script communication
- **SharedState** — Thread-safe key-value store shared across scripts
- **ConfigStore** — Persistent per-script configuration

## RPC Protocol

Communication with the game client uses MessagePack-RPC over Windows named pipes:

- Length-prefixed framing (4-byte LE header)
- Request/response with method name and params
- Automatic retry with configurable policy
- Per-method call metrics (count, latency, errors)

## Dependencies

- **GUI**: imgui 0.12, winit 0.30, glutin 0.32, glow (OpenGL)
- **Serialization**: serde, rmp-serde (MessagePack), rmpv
- **Async**: tokio (script execution)
- **Dynamic loading**: libloading
- **Concurrency**: dashmap
