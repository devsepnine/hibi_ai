---
description: Save or verify a workflow checkpoint. Captures git state, working tree, and progress markers.
argument-hint: "[create|verify|list] [name]"
allowed-tools: Bash, Read, Write
model: haiku
effort: low
---

# Checkpoint Command

워크플로우 체크포인트를 생성하거나 검증한다.

## Usage

`/checkpoint [create|verify|list] [name]`

## Create Checkpoint

체크포인트 생성 시:

1. `/verify quick`을 실행해 현재 상태가 깨끗한지 확인한다
2. 체크포인트 이름으로 git stash 또는 commit을 생성한다
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
- Status (current, behind, ahead)

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
- `clear` - 오래된 체크포인트 제거 (최근 5개 유지)
