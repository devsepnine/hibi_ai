# TUI Gotchas (Things That Will Bite You)

A catalog of subtle bugs that don't crash but produce a wrecked terminal, lost work, or a "why isn't anything happening" feeling. Each entry: what, why, fix.

## 1. `eprintln!` While Raw Mode Is Active = Corrupted Screen

**What**: Calling `eprintln!`, `println!`, `dbg!`, or `panic!` (without ratatui's panic hook) while the alternate screen is up writes raw bytes that interleave with ratatui's output. Result: a smeared screen the user has to `reset` to fix.

**Why**: ratatui draws via the backend; arbitrary writes to stdout/stderr bypass it. The terminal interprets them in the middle of escape sequences.

**Fix**:

```rust
// 1) Use ratatui::init() — it installs a panic hook that calls restore() first.
let mut terminal = ratatui::init();
// 2) Save errors to log to a FILE, not stderr.
// 3) ONLY eprintln! AFTER ratatui::restore().
let result = run(&mut terminal);
ratatui::restore();
if let Err(e) = result {
    eprintln!("Error: {e:?}"); // safe now
}
```

If you must debug from inside the loop, write to a file:

```rust
use std::io::Write;
let mut log = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/tui.log")?;
writeln!(log, "selected={}", app.selected)?;
```

## 2. `drop(JoinHandle)` Does NOT Join the Thread

**What**: You spawn a thread, drop the handle, assume it's joined or cleaned up. It isn't — it's *detached*. The thread keeps running.

**Why**: `JoinHandle::drop` only releases the handle, not the thread. There's no way to forcibly stop a Rust thread; it must cooperate.

**Fix**: explicitly `.join()` when you want to wait, or design the thread to exit on signal:

```rust
let handle = thread::spawn(move || work());

// Want to wait?
let _ = handle.join();

// Want it to be ignored?
// That's fine — but be sure the thread terminates by itself
// (e.g., its channel sender is dropped, its receiver disconnects, the child exits).
```

For the cancelable-process pattern, the read threads exit naturally when the child closes its stdout/stderr. So dropping their handles is intentional. Just be conscious of it.

## 3. Forgetting Panic Hook = Black Terminal Forever

**What**: App panics. You're back at the shell. Type a key — nothing visible. Echo is off, raw mode still on. User has to `stty sane` or reopen the terminal.

**Why**: The `Drop` impl for `Terminal` does NOT restore the terminal. `enable_raw_mode()` and `EnterAlternateScreen` are pure side effects with no automatic undo.

**Fix**: use `ratatui::init()` (installs the hook automatically) OR install a panic hook manually:

```rust
let prev = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let _ = ratatui::restore();
    prev(info);
}));
```

## 4. Blocking `event::read()` Means No Background Updates

**What**: A spinner doesn't spin. A timer doesn't tick. Background download progress doesn't update.

**Why**: `event::read()` blocks until a key is pressed. While blocked, your loop can't redraw or check channels.

**Fix**: always poll first:

```rust
if event::poll(Duration::from_millis(50))? {
    if let Event::Key(key) = event::read()? { /* ... */ }
}
// Then drain background work, redraw, repeat.
```

50–100ms is the sweet spot — tighter wastes CPU on syscalls, looser feels laggy.

## 5. `try_recv()` in a Tight Loop Without Sleep = 100% CPU

**What**: Loop wakes constantly to `try_recv()`, never blocks.

**Why**: `try_recv` returns immediately whether or not data exists.

**Fix**: pair it with a poll/sleep:

```rust
loop {
    while let Ok(event) = rx.try_recv() {
        handle(event);
    }
    if event::poll(Duration::from_millis(50))? { /* ... */ }
    // No need for an extra sleep — event::poll provides the pacing.
}
```

If you have NO event::poll (e.g., a pure background task), use `recv_timeout`:

```rust
match rx.recv_timeout(Duration::from_millis(100)) {
    Ok(event) => handle(event),
    Err(RecvTimeoutError::Timeout) => tick(),
    Err(RecvTimeoutError::Disconnected) => break,
}
```

## 6. Widening `Layout::Min(0)` Vs `Layout::Fill(1)`

**What**: Two `Min(0)` regions don't split remaining space the way you expect — one of them eats almost everything.

**Why**: `Min(0)` means "at least 0", with no upper bound. The solver picks somewhat arbitrarily.

**Fix**: use `Constraint::Fill(weight)` for proportional distribution:

```rust
// BAD: ambiguous division of remaining space
[Constraint::Min(0), Constraint::Min(0)]

// GOOD: 1:1 split of remaining space
[Constraint::Fill(1), Constraint::Fill(1)]

// GOOD: 1:2 split
[Constraint::Fill(1), Constraint::Fill(2)]
```

## 7. Storing `&mut Frame` Across Yields / Awaits

**What**: You try to keep a reference to `Frame` across an `.await` and the borrow checker rejects it.

**Why**: `Frame` is borrowed for the duration of `terminal.draw(|f| ...)`. It must not outlive that closure.

**Fix**: collect everything you need to render into plain data first, then call `draw` with a closure that does only rendering. No I/O inside `draw`.

```rust
// Compute outside
let snapshot = state.snapshot();

// Render with a self-contained closure
terminal.draw(|f| ui(f, &snapshot))?;
```

## 8. Resizing the Terminal Mid-Modal Loses Focus

**What**: User resizes the terminal while a modal is open; the modal moves but the focus indicator doesn't follow.

**Why**: Layouts are recomputed on every draw, but app state about "where focus is" is in cells, not in semantic units.

**Fix**: store focus by semantic ID (e.g., `FieldId::Email`), not by `(x, y)`. Recompute the cell on each draw from the current layout.

## 9. `Constraint::Length(n)` Doesn't Subtract Borders

**What**: A `Block::bordered()` with `Constraint::Length(3)` shows 1 line of content, not 3.

**Why**: Borders take 2 lines (top + bottom). The 3 = 2 borders + 1 content.

**Fix**: account for borders, or get the inner area:

```rust
let block = Block::bordered();
let inner = block.inner(area);
f.render_widget(block, area);
f.render_widget(content, inner);
```

## 10. Forgetting `--release` Makes ratatui Feel Slow

**What**: A complex UI feels laggy in development.

**Why**: ratatui does a lot of buffer diffing per frame. In debug builds, it's slow.

**Fix**: profile with `cargo run --release`. Most "performance issues" disappear.

---

## Quick Self-Check Before Shipping

- [ ] Used `ratatui::init()` / `ratatui::restore()` (or installed a panic hook)
- [ ] No `eprintln!`/`println!`/`dbg!` inside the run loop
- [ ] `event::poll(timeout)` used, not bare `event::read()`
- [ ] Background channels drained per loop iteration
- [ ] `Constraint::Fill(n)` used for proportional space (not `Min(0)`)
- [ ] All threads either explicitly joined or designed to exit
- [ ] Tested with `--release`
