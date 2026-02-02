# Contributing to Chronicle AI

Thanks for your interest in contributing! This document will help you get started.

## Development Setup

**Prerequisites:**
- Rust toolchain ([rustup.rs](https://rustup.rs/))
- Anthropic API key for testing ([console.anthropic.com](https://console.anthropic.com/))

**Setup:**
```bash
git clone https://github.com/SamuelSchlesinger/chronicle-ai.git
cd chronicle-ai
cp .env.example .env
# Edit .env and add your ANTHROPIC_API_KEY
```

**Build & Test:**
```bash
cargo build --workspace     # Build all crates
cargo test --workspace      # Run all tests
cargo clippy --workspace    # Check for lints
cargo fmt --check           # Check formatting
```

**Run the game:**
```bash
cargo run -p chronicle
```

## Project Structure

| Crate | Path | Purpose |
|-------|------|---------|
| `claude` | `claude/` | Minimal Anthropic API client |
| `chronicle-macros` | `chronicle-macros/` | Proc macros for tool definitions |
| `chronicle-core` | `chronicle-core/` | Game engine, rules, AI DM |
| `chronicle` | `chronicle-bevy/` | Bevy GUI application |

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write doc comments for public APIs
- Add tests for new functionality

## Adding a New DM Tool

Tools let the AI DM interact with game mechanics. To add one:

```rust
use chronicle_macros::Tool;
use serde::Deserialize;

/// Brief description of what the tool does
#[derive(Tool, Deserialize)]
#[tool(name = "your_tool_name")]
struct YourTool {
    /// Description of this parameter
    some_param: String,
    /// Optional parameters use Option
    optional_param: Option<i32>,
}
```

Then implement the tool handler in `chronicle-core/src/dm/tools.rs`.

## D&D Content Guidelines

This project uses **SRD 5.2** content under Creative Commons. When adding D&D content:

- **Use only SRD content** - Check `docs/SRD_CC_v5.2.pdf` if unsure
- **Safe:** 9 SRD races, 12 base classes, SRD spells/monsters
- **Not safe:** Content from PHB, MM, or other sourcebooks beyond the SRD

See `CLAUDE.md` for detailed licensing guidance.

## Pull Requests

1. Fork the repo and create a feature branch
2. Make your changes with clear commit messages
3. Ensure `cargo test`, `cargo clippy`, and `cargo fmt --check` pass
4. Open a PR with a description of what you changed and why

## Questions?

Open an issue for bugs, feature requests, or questions about the codebase.
