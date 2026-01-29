# dnd-ai

A D&D 5e game with an AI Dungeon Master, built in Rust.

## Quick Start

```bash
# Set your Anthropic API key
export ANTHROPIC_API_KEY=your_key_here
# Or create a .env file with: ANTHROPIC_API_KEY=your_key_here

# Run the TUI game
cargo run -p dnd

# Or run in headless mode
cargo run -p dnd -- --headless --name "Thorin" --class fighter --race dwarf

# Or run the Bevy GUI
cargo run -p dnd-bevy
```

## Features

- **Full D&D 5e mechanics**: Dice rolling, skill checks, combat, conditions
- **AI Dungeon Master**: Powered by Claude with context management and story memory
- **Multiple interfaces**: Terminal UI (vim-style) and Bevy GUI
- **Campaign persistence**: Save and load your adventures
- **Character creation**: Races, classes, and backgrounds
- **Consequence system**: AI tracks and triggers story consequences

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              dnd / dnd-bevy (applications)                   │
│  Terminal UI (ratatui) │ GUI (Bevy)                          │
└─────────────────────────────┬───────────────────────────────┘
                              │ uses
┌─────────────────────────────▼───────────────────────────────┐
│                    dnd-core (library)                        │
│  GameSession, RulesEngine, AI Dungeon Master, Persistence    │
└─────────────────────────────┬───────────────────────────────┘
                              │ uses
┌─────────────────────────────▼───────────────────────────────┐
│                     claude (library)                         │
│  Anthropic API client: completions, streaming, tool use      │
└─────────────────────────────────────────────────────────────┘
```

## Workspace Structure

| Crate | Description |
|-------|-------------|
| `claude` | Minimal Anthropic Claude API client |
| `dnd-macros` | Procedural macros for tool definitions |
| `dnd-core` | D&D 5e game engine with AI Dungeon Master |
| `dnd` | Terminal UI application |
| `dnd-bevy` | Bevy GUI application |

## TUI Controls

| Mode | Key | Action |
|------|-----|--------|
| Normal | `i` | Enter insert mode |
| Normal | `:` | Enter command mode |
| Normal | `?` | Toggle help |
| Normal | `j`/`k` | Scroll narrative |
| Insert | `Enter` | Send message |
| Insert | `Esc` | Return to normal mode |
| Command | `:q` | Quit |
| Command | `:w` | Save game |
| Command | `:e <file>` | Load game |

## Using the Claude Client

The `claude` crate can be used independently:

```rust
use claude::{Claude, Request, Message};

#[tokio::main]
async fn main() -> Result<(), claude::Error> {
    let client = Claude::from_env()?;

    let response = client.complete(
        Request::new(vec![Message::user("Hello!")])
            .with_system("You are a helpful assistant.")
    ).await?;

    println!("{}", response.text());
    Ok(())
}
```

See `claude/examples/` for more examples including tool use.

## Development

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run Claude API examples
cargo run -p claude --example simple_chat
cargo run -p claude --example tool_use
```

## License

MIT
