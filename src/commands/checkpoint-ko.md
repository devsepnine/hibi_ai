---
description: Save, verify, pause, or resume a workflow checkpoint. Captures git state and progress markers, and writes a resume note a fresh session can pick up from.
argument-hint: "[create|verify|list|pause|resume] [name]"
allowed-tools: Bash, Read, Write, Agent
model: sonnet
effort: medium
---

# Checkpoint Command

워크플로우 체크포인트를 생성하거나 검증한다.

## Usage

`/checkpoint [create|verify|list|pause|resume] [name]`

## Create Checkpoint

체크포인트 생성 시:

1. `/verify quick`을 실행해 현재 상태가 깨끗한지 확인한다
2. 체크포인트 이름으로 git stash를 생성한다. commit은 기본 동작이 아니다. `/checkpoint`가 지칭하는 것은 체크포인트이지 커밋이 아니므로, 커밋에는 별도의 명시적 요청이 필요하다. 근거는 `commit-rules` skill이다
3. 체크포인트를 `.claude/checkpoints.log`에 기록한다:

```bash
echo "$(date +%Y-%m-%d-%H:%M) | $CHECKPOINT_NAME | $(git rev-parse --short HEAD)" >> .claude/checkpoints.log
```

4. 체크포인트 생성 결과를 보고한다

## Verify Checkpoint

체크포인트 대비 검증 시:

1. 로그에서 체크포인트를 읽는다
2. 현재 상태를 체크포인트와 비교한다:
   - 체크포인트 이후 추가된 파일
   - 체크포인트 이후 수정된 파일
   - 현재 vs 당시의 테스트 통과율
   - 현재 vs 당시의 커버리지

3. 보고:
```
CHECKPOINT COMPARISON: $NAME
============================
Files changed: X
Tests: +Y passed / -Z failed
Coverage: +X% / -Y%
Build: [PASS/FAIL]
```

## List Checkpoints

다음 정보를 포함한 모든 체크포인트를 표시한다:
- Name
- Timestamp
- Git SHA
- Status: current, behind, ahead

## Pause

명시적으로 멈추라는 요청이 있을 때만 한다. "계속해" 또는 "멈추지 마"는 그런 요청이 아니다.

1. 안전한 경계에서 멈춘다: 지금의 원자적 단계를 끝내거나 되돌리고, 새 작업을 시작하지 않으며, 백그라운드 서브에이전트를 멈춘다.
2. 멈추려고 되돌릴 수 없는 동작을 하지 않는다. 사용자가 요청하지 않는 한 commit, push, PR을 하지 않는다. 커밋하지 않은 편집은 이미 디스크에 남아 있는 작업 트리에 그대로 둔다. 멈추려고 stash하지 않는다. `create`가 stash를 남겼다면 그 ref를 노트에 적는다.
3. 작업 파일이 있는 worktree의 `.claude/resume/<name>.md`에 재개 노트를 쓴다. 섹션: Intent, Done and verified, Current state, Next step, Key files, Gotchas. "완료" 주장마다 measured 또는 inferred를 표시한다. 기존 결정 로그가 있으면 복사하지 말고 가리킨다. 노트는 세션 전용이다: 프로젝트가 `.claude/resume/`를 무시하지 않으면 커밋하지 말고 `.gitignore`에 추가하자고 제안한다.

   Current state에 작업 파일이 있는 worktree의 Git 루트 절대 경로, 브랜치 또는 detached 상태, 전체 HEAD 커밋 ID를 적는다. 해당 worktree에서 `git rev-parse --show-toplevel`, `git symbolic-ref --short -q HEAD`, `git rev-parse --verify HEAD`를 실행한다.
4. 어디서 멈췄는지, 노트의 절대 경로, 기록한 worktree가 깨끗한지, 재개할 때 첫 작업이 무엇인지 알린다.

## Resume

1. `.claude/resume/<name>.md`부터 읽는다. 세션을 다른 경로에서 시작했으면 중단 시 보고한 절대 경로를 사용한다. 노트 옆의 `-decisions.tsv`, 체크포인트 로그, 지정된 이전 transcript를 읽는다. 그 세션은 `claude --resume`으로 다시 열거나, 보통 `~/.claude/projects/` 아래 인코딩된 작업 디렉터리 이름의 폴더에 있는 JSONL을 읽는다. 긴 transcript는 서브에이전트에서 파싱하고 줄인 타임라인만 남긴다.
2. 편집 전에 기록한 worktree 루트, 브랜치 또는 detached 상태, 전체 HEAD를 확인한다. 하나라도 다르면 `git worktree list`, `git log`, `git diff`로 현재 worktree와 그사이 변경을 확인한다. 이 항목이 없는 이전 노트는 기록한 경로와 Git 이력을 활용한다. 이어갈 체크아웃이나 리비전이 여전히 불분명하면 사용자에게 확인한다.
3. 확인한 worktree에서 이력과 base 대비 diff를 읽는다. 재개 지점을 밝히고 이미 내린 결정을 이어받는다. 이어받은 "완료" 주장은 후속 작업을 하기 전에 실제 결과물에서 다시 검증한다.
4. 남은 작업을 `/bugfix`, `/refactor`, `/perf`, `/plan` 중 맞는 커맨드에 넘기고, 무엇을 이어받았고 무엇을 다시 했으며 왜 그랬는지 말한다.

## 장시간 무인 실행

`/loop`로 돌리거나 밤새 켜 두는 작업에 적용한다:

- 첫 반복 전에 종료 조건을 확인 가능한 술어로 정한다. 예: "테스트 green이고 재현이 통과한다". 시간은 종료 조건이 아니다.
- 반복마다 증거가 정당화하는 가장 작은 변경을 하고, 술어에 비추어 확인하고, 남기거나 버린다. 버린다는 것은 이번 반복이 직접 한 편집만 되돌리는 것이며, `git reset --hard`나 이 실행이 바꾸지 않은 파일의 checkout은 하지 않는다. 무엇이 바뀌었고 술어가 움직였는지를 반복당 한 줄씩 `.claude/resume/<name>-decisions.tsv`에 기록한다.
- 정체는 멈출 이유가 아니다: 접근을 바꾸고 계속한다. 술어가 충족되거나, 이유를 적은 진짜 막다른 길이거나, 되돌릴 수 없는 단계나 제품 결정일 때 멈춘다. 승리를 선언하려고 술어를 느슨하게 하지 않는다.
- 커밋은 `CLAUDE.md`대로 여전히 명시적 요청이 필요하다. 무인 실행이라고 달라지지 않는다.

## Decision log

장시간 실행, `/perf`, 그리고 변경을 시도하고 남기거나 되돌리는 모든 반복은 로그 하나를 같이 쓴다.
그래야 재개한 세션이 모든 흔적을 같은 방식으로 읽는다.

- **위치**: 재개 노트 옆, 커밋 밖의 `.claude/resume/<name>-decisions.tsv`. 프로젝트가
  `.claude/resume/`를 무시하지 않으면 `.gitignore`에 추가하자고 제안한다.
- **열**: 탭으로 구분하고, 시도마다 한 행, 맨 위에 헤더:

  ```
  id	time	change	evidence	verdict	note
  ```

  `evidence`에는 측정한 것을 적는다. 예: `p50 412ms -> 371ms`, `tests 151/151`. `verdict`는
  `kept`, `reverted`, `stopped` 중 하나다.
- 남겼든 아니든 모든 시도마다 행을 추가한다. 이긴 것만 있는 로그로는 결과를 설명할 수 없다.

## Workflow

일반적인 체크포인트 흐름:

```
[Start] --> /checkpoint create "feature-start"
   |
[Implement] --> /checkpoint create "core-done"
   |
[Test] --> /checkpoint verify "core-done"
   |
[Refactor] --> /checkpoint create "refactor-done"
   |
[PR] --> /checkpoint verify "feature-start"
```

## Arguments

$ARGUMENTS:
- `create <name>` - 명명된 체크포인트 생성
- `verify <name>` - 명명된 체크포인트 대비 검증
- `list` - 모든 체크포인트 표시
- `pause <name>`: 안전한 경계에서 멈추고 재개 노트를 쓴다
- `resume <name>`: 끝난 작업을 다시 하지 않고 재개 노트에서 이어받는다
- `clear`: 오래된 체크포인트를 제거하고 최근 5개를 유지한다
