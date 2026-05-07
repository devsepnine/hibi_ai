---
name: eval-harness
description: Eval-driven development framework with capability/regression testing and pass@k metrics. Use when evaluating AI outputs, running eval suites, 평가 프레임워크, 회귀 테스트, AI 평가, 성능 측정.
keywords: [eval, evaluation, 평가, regression-test, 회귀테스트, pass-at-k]
---

# Eval Harness Skill

Claude Code 세션을 위한 형식화된 평가 프레임워크로, eval-driven development (EDD) 원칙을 구현한다.

## 철학

Eval-Driven Development는 eval을 "AI 개발의 단위 테스트"로 다룬다:
- 구현 BEFORE 기대 동작 정의
- 개발 중 지속적으로 eval 실행
- 변경마다 회귀 추적
- 신뢰성 측정에 pass@k 메트릭 사용

## Eval 타입

### Capability Evals
Claude가 이전에 못 했던 것을 할 수 있는지 테스트:
```markdown
[CAPABILITY EVAL: feature-name]
Task: Description of what Claude should accomplish
Success Criteria:
  - [ ] Criterion 1
  - [ ] Criterion 2
  - [ ] Criterion 3
Expected Output: Description of expected result
```

### Regression Evals
변경이 기존 기능을 깨뜨리지 않도록 보장:
```markdown
[REGRESSION EVAL: feature-name]
Baseline: SHA or checkpoint name
Tests:
  - existing-test-1: PASS/FAIL
  - existing-test-2: PASS/FAIL
  - existing-test-3: PASS/FAIL
Result: X/Y passed (previously Y/Y)
```

## Grader 타입

### 1. Code-Based Grader
코드 사용 결정론적 체크:
```bash
# Check if file contains expected pattern
grep -q "export function handleAuth" src/auth.ts && echo "PASS" || echo "FAIL"

# Check if tests pass
npm test -- --testPathPattern="auth" && echo "PASS" || echo "FAIL"

# Check if build succeeds
npm run build && echo "PASS" || echo "FAIL"
```

### 2. Model-Based Grader
Claude를 사용해 개방형 출력을 평가:
```markdown
[MODEL GRADER PROMPT]
Evaluate the following code change:
1. Does it solve the stated problem?
2. Is it well-structured?
3. Are edge cases handled?
4. Is error handling appropriate?

Score: 1-5 (1=poor, 5=excellent)
Reasoning: [explanation]
```

### 3. Human Grader
수동 리뷰 플래그:
```markdown
[HUMAN REVIEW REQUIRED]
Change: Description of what changed
Reason: Why human review is needed
Risk Level: LOW/MEDIUM/HIGH
```

## 메트릭

### pass@k
"k 시도 중 적어도 한 번 성공"
- pass@1: 첫 시도 성공률
- pass@3: 3 시도 내 성공
- 일반 목표: pass@3 > 90%

### pass^k
"k 시도 모두 성공"
- 신뢰성 기준이 더 높음
- pass^3: 3회 연속 성공
- 크리티컬 패스에 사용

## Eval 워크플로우

### 1. 정의 (코딩 전)
```markdown
## EVAL DEFINITION: feature-xyz

### Capability Evals
1. Can create new user account
2. Can validate email format
3. Can hash password securely

### Regression Evals
1. Existing login still works
2. Session management unchanged
3. Logout flow intact

### Success Metrics
- pass@3 > 90% for capability evals
- pass^3 = 100% for regression evals
```

### 2. 구현
정의된 eval을 통과하도록 코드를 작성한다.

### 3. 평가
```bash
# Run capability evals
[Run each capability eval, record PASS/FAIL]

# Run regression evals
npm test -- --testPathPattern="existing"

# Generate report
```

### 4. 리포트
```markdown
EVAL REPORT: feature-xyz
========================

Capability Evals:
  create-user:     PASS (pass@1)
  validate-email:  PASS (pass@2)
  hash-password:   PASS (pass@1)
  Overall:         3/3 passed

Regression Evals:
  login-flow:      PASS
  session-mgmt:    PASS
  logout-flow:     PASS
  Overall:         3/3 passed

Metrics:
  pass@1: 67% (2/3)
  pass@3: 100% (3/3)

Status: READY FOR REVIEW
```

## 통합 패턴

### 구현 전
```
/eval define feature-name
```
`.claude/evals/feature-name.md`에 eval 정의 파일 생성

### 구현 중
```
/eval check feature-name
```
현재 eval을 실행하고 상태를 보고

### 구현 후
```
/eval report feature-name
```
전체 eval 리포트 생성

## Eval 저장

프로젝트에 eval 저장:
```
.claude/
  evals/
    feature-xyz.md      # Eval definition
    feature-xyz.log     # Eval run history
    baseline.json       # Regression baselines
```

## 모범 사례

1. **코딩 BEFORE eval 정의** - 성공 기준에 대한 명확한 사고를 강제
2. **자주 eval 실행** - 회귀를 빨리 잡는다
3. **pass@k 시간 추적** - 신뢰성 추세 모니터링
4. **가능하면 코드 grader 사용** - 결정론적 > 확률적
5. **보안에는 사람 리뷰** - 보안 체크는 절대 완전 자동화하지 말 것
6. **eval은 빠르게 유지** - 느린 eval은 실행되지 않는다
7. **코드와 함께 eval 버전 관리** - eval은 1급 아티팩트

## 예시: 인증 추가

```markdown
## EVAL: add-authentication

### Phase 1: Define (10 min)
Capability Evals:
- [ ] User can register with email/password
- [ ] User can login with valid credentials
- [ ] Invalid credentials rejected with proper error
- [ ] Sessions persist across page reloads
- [ ] Logout clears session

Regression Evals:
- [ ] Public routes still accessible
- [ ] API responses unchanged
- [ ] Database schema compatible

### Phase 2: Implement (varies)
[Write code]

### Phase 3: Evaluate
Run: /eval check add-authentication

### Phase 4: Report
EVAL REPORT: add-authentication
==============================
Capability: 5/5 passed (pass@3: 100%)
Regression: 3/3 passed (pass^3: 100%)
Status: SHIP IT
```
