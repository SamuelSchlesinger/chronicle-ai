# Developer Handoff

## What was done

Fixed multiple TUI issues for better responsiveness and readability:

1. **Scroll synchronization** - Scroll was broken because `scroll_to_bottom()` set a huge value that scroll_up couldn't properly decrement. Added `estimate_max_scroll()` to cap scroll position.

2. **HP gauge readability** - Changed from green foreground (hard to read) to green background with contrasting black/white text.

3. **Dice roll text wrapping** - Added `Wrap` to prevent skill check purpose text from being cut off.

4. **Player action visibility** - Player actions no longer auto-scroll, so users can see their input. Added explicit `stdout().flush()` before async wait.

## Current state

- All TUI fixes committed
- Tests passing, clippy clean
- Ready for async UI refactor

## Next steps: Async UI Architecture

The current main loop blocks during AI processing, freezing the UI. Here's the plan to make it responsive:

### Phase 1: Background AI Task

**Files to modify:**
- `dnd/src/app.rs` - Add task handle storage
- `dnd/src/main.rs` - Spawn AI task instead of awaiting inline

**Changes:**
```rust
// In App struct:
ai_task: Option<tokio::task::JoinHandle<Result<DmResponse, SessionError>>>,

// In main loop, instead of:
app.process_player_input_without_echo(&input).await;

// Do:
let session_clone = app.session.clone(); // Need to make session Clone or use Arc
let input_clone = input.clone();
app.ai_task = Some(tokio::spawn(async move {
    session_clone.player_action(&input_clone).await
}));

// Then each loop iteration, check:
if let Some(task) = &mut app.ai_task {
    if task.is_finished() {
        let result = app.ai_task.take().unwrap().await.unwrap();
        // Process result...
    }
}
```

### Phase 2: Streaming Text Display

**Files to modify:**
- `dnd-core/src/session.rs` - Add streaming method
- `claude/src/lib.rs` - Already has streaming support
- `dnd/src/app.rs` - Handle streaming chunks

**The `streaming_text` field already exists** in App and NarrativeWidget renders it with a cursor indicator. Just need to feed it data.

### Phase 3: Cancellation Support

- Allow Ctrl+C or Escape to cancel pending AI request
- Use `tokio::select!` or `AbortHandle`

### Considerations

1. **Session cloning** - `GameSession` contains `DungeonMaster` which has non-Clone types. Options:
   - Wrap in `Arc<Mutex<>>`
   - Extract just the needed data for AI call
   - Use channels to communicate with a dedicated AI task

2. **State consistency** - Ensure game state isn't modified while AI is processing

3. **Error handling** - Handle task panics gracefully

## Known issues

- Many other files have uncommitted changes from previous sessions (run `git status` to see)
