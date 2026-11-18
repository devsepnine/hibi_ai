# Cancelable Background Processes in TUI

TUI에서 자식 프로세스를 spawn하면서 UI를 응답성 있게 유지하고 동시에 사용자가, 앱을 종료하지 않고 Esc / Ctrl+C로, 취소할 수 있게 하는 프로덕션 패턴이다.

이는 프로덕션 TUI에서 가장 중요한 자명하지 않은 단일 패턴이다. 순진한 `Command::output()`은 이벤트 루프를 block한다, 사용자가 'q'조차 누를 수 없다.

## The Race Condition That Bites Everyone

자식을 spawn하고, cancel 신호를 읽고, 자식을 kill한다. 간단해 보인다. 버그:

```text
T0: child finishes successfully (status=0), wrote results to disk
T1: user presses Esc (sends cancel signal)
T2: your loop checks cancel FIRST, sees the signal, kills the (already-dead) child,
    rolls back the work the user just successfully completed
```

**Rule**: 각 루프 반복에서 **completion first, cancel second**를 확인한다. 성공적인 종료가 늦은 cancel을 이긴다.

## Reference Implementation

이 예시는 워커에서 직계 자식을 실행하고 취소 시 그 자식을 종료·회수한다. 자손까지 중지해야 하는 작업에는 운영체제의 프로세스 그룹이나 Job Object가 필요하다. 표준 오류는 `tempfile`의 임시 파일로 받아 자손이 출력 핸들을 상속해도 자식의 종료를 즉시 처리할 수 있다. 실패 시점의 마지막 8 KiB만 읽으며, 실행 중 로그의 디스크 사용량은 별도로 관리한다.

```rust
use anyhow::{bail, Context};
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Stdio};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use tempfile::NamedTempFile;
use wait_timeout::ChildExt;

pub struct ProcessConfig {
    pub cancel_rx: Receiver<()>,
    pub poll_interval: Duration,
}

pub fn spawn_cancelable_process(
    command: &mut Command,
    config: ProcessConfig,
) -> anyhow::Result<()> {
    let mut diagnostics = NamedTempFile::new()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(diagnostics.reopen()?)
        .spawn()
        .context("spawn background process")?;

    let status = loop {
        if let Some(status) = child.wait_timeout(config.poll_interval)? {
            break status;
        }
        if config.cancel_rx.try_recv().is_ok() {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if let Err(error) = child.kill() {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
                return Err(error).context("stop background process");
            }
            let status = child.wait().context("reap background process")?;
            if status.success() {
                break status;
            }
            bail!("cancelled by user");
        }
    };

    if status.success() {
        return Ok(());
    }
    const TAIL_BYTES: u64 = 8192;
    let end = diagnostics.as_file().metadata()?.len();
    let start = end.saturating_sub(TAIL_BYTES);
    diagnostics.seek(SeekFrom::Start(start))?;
    let mut tail = Vec::new();
    diagnostics.take(end - start).read_to_end(&mut tail)?;
    bail!("process failed ({status}): {}", String::from_utf8_lossy(&tail))
}
```

## Wiring It Into the TUI Loop

TUI 측: 각 spawn이 자체 새 채널을 얻도록 "current cancel sender"를 소유한다. spawn마다 receiver를 교체하면 job N의 취소가 job N+1을 죽이지 않는다.

```rust
use std::sync::mpsc::{self, Sender};

pub struct Channels {
    /// Replaced on each spawn — keeps per-job isolation.
    pub current_cancel_tx: Sender<()>,
    pub processing_active: bool,
}

impl Channels {
    /// Take the receiver for a new spawn. Replaces internal tx so the
    /// next call to `cancel()` only affects the NEXT job.
    pub fn take_cancel_rx(&mut self) -> mpsc::Receiver<()> {
        let (new_tx, new_rx) = mpsc::channel();
        self.current_cancel_tx = new_tx;
        new_rx
    }

    pub fn cancel(&self) {
        let _ = self.current_cancel_tx.send(());
    }
}
```

**TUI key handler:**

```rust
match key.code {
    KeyCode::Esc | KeyCode::Char('q') if app.processing_active => {
        app.channels.cancel();
        // Don't quit the app — just cancel the in-flight job.
    }
    KeyCode::Char('q') => app.should_quit = true,
    _ => {}
}
```

## Cooperative Cancellation in Pure-Rust Tasks

모든 긴 task가 자식 프로세스인 것은 아니다. in-process task, 예를 들어 재귀 스캔의 경우, 작업 단위 사이에 cancel receiver를 확인한다:

```rust
fn scan_dir(root: &Path, cancel_rx: &Receiver<()>) -> anyhow::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    for entry in walkdir::WalkDir::new(root) {
        // Cooperative check between iterations.
        if cancel_rx.try_recv().is_ok() {
            anyhow::bail!("cancelled");
        }
        results.push(entry?.into_path());
    }
    Ok(results)
}
```

**세분성 규칙**: ~100ms마다 적어도 한 번 실행되는 루프 경계에서 cancel을 확인한다. 더 자주 = 더 응답성 있는 cancel; 덜 자주 = `try_recv`에 낭비된 CPU.

## Why This Pattern, Not `tokio::select!`

많은 튜토리얼이 async + `tokio::select!`에 손을 댄다. 작동하지만, 모든 TUI가 tokio에 의존하고 모든 sync 코드를 다시 쓰도록 강제한다. 위의 mpsc-channel 패턴은:

- std blocking `Command` API와 함께 작동, async wrapper 불필요,
- TUI 루프의 `event::poll(timeout)`과 조합,
- 단위 테스트의 일반 `mpsc::channel`로 테스트 가능.

프로젝트가 이미 async-first라면 `tokio::process::Command` + `tokio::select!`가 괜찮다. completion을 먼저 확인한다는 원칙은 여전히 적용된다.
