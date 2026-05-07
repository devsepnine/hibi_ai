---
name: verification-loop
description: Comprehensive verification system for build, types, tests, and security checks. Use when 빌드 검증, 타입 체크, 테스트 실행, 검증 루프, 완료 전 검증.
keywords: [verification, 검증, build-check, type-check, 빌드검증, 타입체크]
---

# Verification Loop Skill

Claude Code 세션을 위한 종합 검증 시스템.

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

Overall:   [READY/NOT READY] for PR

Issues to Fix:
1. ...
2. ...
```

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
