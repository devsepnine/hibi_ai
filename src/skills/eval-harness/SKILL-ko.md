---
name: eval-harness
description: Eval-driven development — capability and regression suites, pass@k metrics. Use when evaluating AI output or running eval suites. 평가 프레임워크, 회귀 테스트, AI 평가, 성능 측정.
---

# Eval Harness Skill

Claude Code 세션을 위한 형식화된 평가 프레임워크로, eval-driven development (EDD) 원칙을 구현한다.

## 철학

Eval-Driven Development는 eval을 "AI 개발의 단위 테스트"로 다룬다:
- 구현 BEFORE 기대 동작 정의
- 개발 중 지속적으로 eval 실행
- 변경마다 회귀 추적
- 신뢰성 측정에 pass@k 메트릭 사용

## 실행 가능한 스크립트

여기서 두 가지 측정은 템플릿이 아니라 실행물이다. 둘 다 Python 3.9+ 와
`claude` CLI 만 필요하다. 아래 경로는 이 스킬 디렉터리 기준이므로 실제 위치를
앞에 붙인다 — 설치본은 `~/.claude/skills/eval-harness/`, hibi-ai 저장소는
`src/skills/eval-harness/`.

### 스킬 설명이 실제로 트리거되는가

```bash
python3 scripts/trigger_eval.py --skill iced_rs \
  --eval-set path/to/eval_set.json --timeout 300 --jobs 4
```

eval set 은 `{"query": ..., "should_trigger": true|false}` 의 JSON 리스트다.
각 질의를 nested `claude -p` 로 실행하고, 스트림에서 `input.skill` 이 스킬
**디렉터리명**과 일치하는 `Skill` tool_use 를 찾는다 — 런타임이 내보내는 값은
frontmatter 의 `name:` 이 아니라 디렉터리명이다.

이 스크립트가 강제하는 두 게이트, 그리고 이것이 없는 하네스가 측정처럼
보이지만 측정이 아닌 숫자를 내놓는 이유:

- **양방향 자기검증.** 채점 전에 알려진 양성이 발동하고 알려진 음성이 발동하지
  않음을 먼저 증명한다. 둘 중 하나라도 실패하면 exit 2 로 끝나고 점수를 아예
  보고하지 않는다. "dirty" 를 보고할 수 없는 검출기의 "clean" 은 아무것도
  증명하지 않는다.
- **완료 추적.** 타임아웃으로 죽은 실행은 `result` 이벤트를 내보내지 않으므로
  "Skill 호출 없음" 과 "실행 자체가 없음" 을 구별할 수 없다. 해당 행은
  INCONCLUSIVE 이며 PASS 도 FAIL 도 아니다. 이것이 없으면 모든 should-NOT 질의가
  공허하게 통과하고, 느린 양성은 실패로 읽힌다.

Exit status: 0 정상, 1 FAIL 존재, 2 자기검증 게이트 실패, 3 INCONCLUSIVE 존재,
4 하네스 자체가 실행되지 못함(PATH 에 `claude` 없음, eval set 사용 불가).
INCONCLUSIVE 행은 `--timeout` 을 늘려 순차 재실행한 뒤에 숫자를 인용한다 —
부분 배치는 점수가 아니다.

측정 대상은 작업 사본이 아니라 **설치된** 설명이다: nested 세션은
`~/.claude/skills/` 를 읽고, `src/skills/` 같은 저장소 경로는 Claude Code 가
로드하는 위치가 아니다. 수정한 스킬을 먼저 설치·동기화하지 않으면 이전 설명을
채점하게 된다.

`skill-creator` 플러그인의 `run_eval` 로 대체하지 말 것: 런타임이 절대 내보내지
않는 `<name>-skill-<uuid>` 문자열을 매칭하고, 첫 tool call 이 Skill/Read 가
아니면 즉시 포기하며, nested stderr 를 버린다 — 그래서 안정적이고 그럴듯하며
무의미한 숫자를 돌려준다.

### 스킬 목록 예산

```bash
python3 scripts/skill_budget.py ~/.claude/skills   # 모델이 실제로 보는 목록
python3 scripts/skill_budget.py src/skills         # 설치 전 저장소 트리
```

모델이 보는 스킬 목록에는 하드 문자 예산이 있다 —
`floor(context_window * 4 * 0.01)`, 200K 윈도우에서 8,000자. 초과 시 절삭은
**스킬 단위 전부-아니면-전무**다: 예산에 들어가지 못한 스킬은 `- name` 만 남고
설명을 잃으므로 자동 트리거가 완전히 멈춘다. 초과면 exit 1, 사용 오류는 exit 2
로 갈라서 잘못된 경로가 예산 초과로 읽히지 않게 했다. 200자 작성 목표를 넘는
설명과 디렉터리명과 어긋나는 `name:` 도 함께 표시한다.

이 스크립트가 담고 있는 두 사실, 둘 다 설명 작성 방식을 바꾼다:

- `when_to_use` 는 `description` 에 이어붙여 한 문자열로 측정되므로, 트리거
  어휘를 그쪽으로 옮겨도 절약되지 않는다.
- 길이는 UTF-16 단위로 세므로 한글 음절 1자가 ASCII 1자와 같다 — 한국어 트리거
  어휘는 예산 효율이 높다.

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
