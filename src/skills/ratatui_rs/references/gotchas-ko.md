# TUI Gotchas (Things That Will Bite You)

크래시는 하지 않지만 망가진 터미널, 잃어버린 작업, "왜 아무 일도 일어나지 않지" 느낌을 만드는 미묘한 버그들의 카탈로그이다. 각 항목: what, why, fix.

## 1. `eprintln!` While Raw Mode Is Active = Corrupted Screen

**What**: 대체 화면이 켜져 있는 동안 `eprintln!`, `println!`, `dbg!`, `panic!` (ratatui의 panic hook 없이) 호출은 ratatui의 출력과 interleave되는 raw bytes를 쓴다. 결과: 사용자가 `reset`해야 고쳐지는 번진 화면.

**Why**: ratatui는 backend를 통해 그린다; stdout/stderr로의 임의 write는 이를 우회한다. 터미널은 escape sequence 중간에서 그것들을 해석한다.

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

루프 내부에서 디버깅해야 한다면 파일에 쓴다:

```rust
use std::io::Write;
let mut log = std::fs::OpenOptions::new().create(true).append(true).open("/tmp/tui.log")?;
writeln!(log, "selected={}", app.selected)?;
```

## 2. `drop(JoinHandle)` Does NOT Join the Thread

**What**: thread를 spawn하고, handle을 drop하고, join되거나 cleanup되었다고 가정한다. 그렇지 않다 — *detached*된 것이다. thread는 계속 실행된다.

**Why**: `JoinHandle::drop`은 handle만 release하고, thread는 release하지 않는다. Rust thread를 강제로 stop할 방법은 없다; 협력해야 한다.

**Fix**: 기다리고 싶을 때 명시적으로 `.join()`하거나, signal에서 종료하도록 thread를 설계한다:

```rust
let handle = thread::spawn(move || work());

// Want to wait?
let _ = handle.join();

// Want it to be ignored?
// That's fine — but be sure the thread terminates by itself
// (e.g., its channel sender is dropped, its receiver disconnects, the child exits).
```

cancelable-process 패턴의 경우, read thread는 자식이 stdout/stderr를 닫을 때 자연스럽게 종료된다. 따라서 그들의 handle을 drop하는 것은 의도적이다. 그저 그것을 의식하라.

## 3. Forgetting Panic Hook = Black Terminal Forever

**What**: 앱이 panic한다. 셸로 돌아왔다. 키를 입력 — 보이는 것 없음. echo가 꺼져 있고, raw mode가 여전히 켜져 있다. 사용자가 `stty sane`하거나 터미널을 재오픈해야 한다.

**Why**: `Terminal`의 `Drop` impl은 터미널을 복원하지 *않는다*. `enable_raw_mode()`와 `EnterAlternateScreen`은 자동 undo가 없는 순수 사이드 이펙트이다.

**Fix**: `ratatui::init()`을 사용한다 (자동으로 hook 설치) 또는 panic hook을 수동으로 설치:

```rust
let prev = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    let _ = ratatui::restore();
    prev(info);
}));
```

## 4. Blocking `event::read()` Means No Background Updates

**What**: 스피너가 회전하지 않는다. 타이머가 tick하지 않는다. 백그라운드 다운로드 진행이 업데이트되지 않는다.

**Why**: `event::read()`는 키가 눌릴 때까지 block한다. block된 동안, 루프는 다시 그리거나 채널을 확인할 수 없다.

**Fix**: 항상 먼저 poll한다:

```rust
if event::poll(Duration::from_millis(50))? {
    if let Event::Key(key) = event::read()? { /* ... */ }
}
// Then drain background work, redraw, repeat.
```

50–100ms가 sweet spot이다 — 더 짧으면 syscall에 CPU를 낭비하고, 더 길면 lag를 느낀다.

## 5. `try_recv()` in a Tight Loop Without Sleep = 100% CPU

**What**: 루프가 계속 깨어나서 `try_recv()`하고, 절대 block하지 않는다.

**Why**: `try_recv`는 데이터 존재 여부와 관계없이 즉시 반환한다.

**Fix**: poll/sleep과 짝지운다:

```rust
loop {
    while let Ok(event) = rx.try_recv() {
        handle(event);
    }
    if event::poll(Duration::from_millis(50))? { /* ... */ }
    // No need for an extra sleep — event::poll provides the pacing.
}
```

event::poll이 없다면 (예: 순수 백그라운드 task), `recv_timeout`을 사용한다:

```rust
match rx.recv_timeout(Duration::from_millis(100)) {
    Ok(event) => handle(event),
    Err(RecvTimeoutError::Timeout) => tick(),
    Err(RecvTimeoutError::Disconnected) => break,
}
```

## 6. Widening `Layout::Min(0)` Vs `Layout::Fill(1)`

**What**: 두 개의 `Min(0)` 영역이 기대하는 방식으로 남은 공간을 분할하지 않는다 — 그 중 하나가 거의 모든 것을 먹는다.

**Why**: `Min(0)`은 "최소 0", 상한 없음을 의미한다. solver가 다소 임의로 선택한다.

**Fix**: 비례 분배에 `Constraint::Fill(weight)`을 사용한다:

```rust
// BAD: ambiguous division of remaining space
[Constraint::Min(0), Constraint::Min(0)]

// GOOD: 1:1 split of remaining space
[Constraint::Fill(1), Constraint::Fill(1)]

// GOOD: 1:2 split
[Constraint::Fill(1), Constraint::Fill(2)]
```

## 7. Storing `&mut Frame` Across Yields / Awaits

**What**: `.await`을 가로질러 `Frame`에 대한 참조를 유지하려 하고 borrow checker가 이를 거부한다.

**Why**: `Frame`은 `terminal.draw(|f| ...)` 동안 borrow된다. 그 closure보다 오래 살아서는 안 된다.

**Fix**: 렌더링에 필요한 모든 것을 먼저 평이한 데이터로 수집한 다음, 렌더링만 하는 closure로 `draw`를 호출한다. `draw` 내부에 I/O 없음.

```rust
// Compute outside
let snapshot = state.snapshot();

// Render with a self-contained closure
terminal.draw(|f| ui(f, &snapshot))?;
```

## 8. Resizing the Terminal Mid-Modal Loses Focus

**What**: 모달이 열려 있는 동안 사용자가 터미널 크기를 조정한다; 모달이 이동하지만 focus 표시기가 따라오지 않는다.

**Why**: 레이아웃은 매 draw마다 재계산되지만, "focus가 어디에 있는지"에 대한 앱 state는 의미적 단위가 아니라 셀에 있다.

**Fix**: focus를 `(x, y)`가 아니라 의미적 ID (예: `FieldId::Email`)로 저장한다. 매 draw마다 현재 레이아웃에서 셀을 재계산한다.

## 9. `Constraint::Length(n)` Doesn't Subtract Borders

**What**: `Constraint::Length(3)`인 `Block::bordered()`가 3줄이 아닌 1줄의 콘텐츠를 보여준다.

**Why**: 테두리가 2줄을 차지한다 (top + bottom). 3 = 2 borders + 1 content.

**Fix**: 테두리를 고려하거나, 내부 영역을 얻는다:

```rust
let block = Block::bordered();
let inner = block.inner(area);
f.render_widget(block, area);
f.render_widget(content, inner);
```

## 10. Forgetting `--release` Makes ratatui Feel Slow

**What**: 복잡한 UI가 개발에서 lag를 느낀다.

**Why**: ratatui는 프레임당 많은 buffer diffing을 한다. debug 빌드에서는 느리다.

**Fix**: `cargo run --release`로 프로파일한다. 대부분의 "성능 이슈"가 사라진다.

---

## Quick Self-Check Before Shipping

- [ ] `ratatui::init()` / `ratatui::restore()` 사용 (또는 panic hook 설치)
- [ ] run 루프 내부에 `eprintln!`/`println!`/`dbg!` 없음
- [ ] bare `event::read()`가 아닌 `event::poll(timeout)` 사용
- [ ] 루프 반복마다 백그라운드 채널 drain
- [ ] 비례 공간에 `Constraint::Fill(n)` 사용 (`Min(0)`이 아님)
- [ ] 모든 thread가 명시적으로 join되거나 종료하도록 설계됨
- [ ] `--release`로 테스트됨
