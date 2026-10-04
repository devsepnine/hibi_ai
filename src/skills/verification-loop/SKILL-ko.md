---
name: verification-loop
description: Build, type, test, and security verification loop. Use before reporting completion or when a build or type check fails. 빌드 검증, 타입 체크, 테스트 실행, 검증 루프, 완료 전 검증.
---

# Verification Loop Skill

Claude Code 세션을 위한 종합 검증 시스템.

Phase 1~6은 대리 지표다: build·타입 체크·테스트가 green이면 코드가 일관적이라는 뜻이지, 변경이 요청대로 동작한다는 뜻이 아니다. Phase 7이 실제 결과물을 확인한다. Phase 7을 포함한 실행만 `Overall: READY`를 보고할 수 있다. `/verify quick`, `pre-commit`, 지속 모드 같은 부분 범위는 `Overall: PARTIAL`을 보고하고, Phase 7은 완료 시점에 한 번 실행한다.

## 사용 시점

다음 상황에 이 skill을 호출한다:
- 기능 완료 또는 의미 있는 코드 변경 직후
- PR 생성 전
- 품질 게이트 통과를 보장하고 싶을 때
- 리팩토링 후

## 검증 단계

### Phase 1: Build Verification
```bash
# Check if project builds
npm run build 2>&1 | tail -20
# OR
pnpm build 2>&1 | tail -20
```

빌드가 실패하면 STOP, 진행 전에 고친다.

### Phase 2: Type Check
```bash
# TypeScript projects
npx tsc --noEmit 2>&1 | head -30

# Python projects
pyright . 2>&1 | head -30
```

모든 type 에러를 보고한다. 진행 전에 critical한 것들을 수정한다.

### Phase 3: Lint Check
```bash
# JavaScript/TypeScript
npm run lint 2>&1 | head -30

# Python
ruff check . 2>&1 | head -30
```

### Phase 4: Test Suite
```bash
# Run tests with coverage
npm run test -- --coverage 2>&1 | tail -50

# Check coverage threshold
# Target: 80% minimum
```

보고:
- Total tests: X
- Passed: X
- Failed: X
- Coverage: X%

### Phase 5: Security Scan
```bash
# Check for secrets
grep -rn "sk-" --include="*.ts" --include="*.js" . 2>/dev/null | head -10
grep -rn "api_key" --include="*.ts" --include="*.js" . 2>/dev/null | head -10

# Check for console.log
grep -rn "console.log" --include="*.ts" --include="*.tsx" src/ 2>/dev/null | head -10
```

### Phase 6: Diff Review
```bash
# Show what changed
git diff --stat
git diff HEAD~1 --name-only
```

변경된 각 파일에 대해 검토:
- 의도하지 않은 변경
- 누락된 에러 처리
- 잠재적 엣지 케이스

### Phase 7: 실제 결과물 관찰

변경된 동작을 직접 실행하고 결과를 읽는다:

| 변경 | 확인 |
|------|------|
| CLI | 실제 입력으로 실제 커맨드 실행 |
| UI | 실행 중인 앱에서 변경된 플로우를 직접 조작한다. `/e2e`, `run` skill, 브라우저 자동화를 쓴다 |
| 파서 / 마이그레이션 | 저장해 둔 실제 입력을 재생하고 출력을 diff |
| 저장소 / 설정 | 쓴 값을 다시 읽음 |
| 성능 | 같은 하네스에서 전후 측정값 비교 |
| 문서만 | 독자의 입장에서 diff를 다시 읽음; 동작을 좌우하는 텍스트, 즉 skill·agent·command 프롬프트가 바뀐 경우에만 eval 실행 |

- 캐시되거나 파생된 표현, 예를 들어 파일 mtime, 에이전트의 자기 보고, 오래된 스크린샷이 아니라 실제 값을 읽는다.
- 확인이 실패하면 시스템보다 관찰 방법을 먼저 의심한다.
- 한 번 눈으로 보기보다 비교를 다시 돌릴 수 있는 스크립트를 선호하고, 그 출력을 증거로 남긴다.
- 직접 돌릴 수 있는 확인을 사용자에게 넘기지 않는다. 기기나 자격증명이 없어 여기서 정말 실행할 수 없다면 `INCONCLUSIVE`로 표시하고 무엇이 판정할지 적는다. `INCONCLUSIVE`이거나 변경된 곳과 다른 환경에서 실행한 확인은 `NOT READY`다.

## 출력 포맷

모든 phase를 실행한 후 검증 보고서를 산출한다:

```
VERIFICATION REPORT
==================

Build:     [PASS/FAIL]
Types:     [PASS/FAIL] (X errors)
Lint:      [PASS/FAIL] (X warnings)
Tests:     [PASS/FAIL] (X/Y passed, Z% coverage)
Security:  [PASS/FAIL] (X issues)
Diff:      [X files changed]
Artifact:  [PASS/FAIL/INCONCLUSIVE] (what was run, what was observed)

Overall:   [READY/NOT READY/PARTIAL] for PR

Issues to Fix:
1. ...
2. ...
```

보고서와 완료 메시지의 모든 주장에 **measured**, 즉 직접 실행해 출력을 본 것, **inferred**, 즉 측정된 것에서 따라 나오는 것, **guess** 중 하나를 붙인다. inferred 주장은 inferred로 보고하고, 판정은 measured 주장만 근거로 삼는다.

## 지속 모드

긴 세션에서는 15분마다 또는 주요 변경 후 검증을 실행한다:

```markdown
멘탈 체크포인트 설정:
- 함수 완료 후
- 컴포넌트 완료 후
- 다음 작업으로 넘어가기 전

실행: /verify
```

## Hooks와의 통합

이 skill은 PostToolUse 훅을 보완하면서 더 깊은 검증을 제공한다.
훅은 이슈를 즉시 잡고, 이 skill은 종합 리뷰를 제공한다.
