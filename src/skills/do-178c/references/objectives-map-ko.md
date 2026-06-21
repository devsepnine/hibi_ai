# DO-178C to Harness Objectives Map

이 문서는 중복 방지를 위한 핵심 지도다. 각 DO-178C 프로세스 영역에 대해 이미 해당 작업을 담당하는 기존 하니스 구성요소를 명시하여, `do-178c` 스킬이 그 내용을 다시 반복하지 않도록 한다. `do-178c` 스킬은 얇은 오버레이다. 아래의 담당자들을 cross-reference 할 뿐이며, 오직 네 가지 신규 항목만 추가한다.

DO-178C는 작업을 다음 프로세스로 묶는다: planning, development, verification, configuration management, quality assurance, 그리고 tool qualification (DO-330). 하니스는 이미 기존 skill, agent, `CLAUDE.md` 정책을 통해 이들 대부분을 구현하고 있다. 아래 표는 각 DO-178C 프로세스 영역을 기존 담당자에 매핑하고, `do-178c`가 무엇을 (있다면) 추가하는지를 기술한다.

Action legend:
- `CROSS-REF` — 담당자만으로 충분하다. `do-178c`는 이를 가리킬 뿐이며, 상위 tier에서는 명시적으로 요구할 수 있다.
- `EXTEND` — 담당자는 존재하지만 `do-178c`가 그 범위를 넓힌다.
- `NEW` — 현재 담당자가 없으며 `do-178c`가 새로 도입한다.

## Mapping Table

| DO-178C process / objective | Existing harness owner | `do-178c` action (CROSS-REF / EXTEND / NEW) |
|---|---|---|
| Planning (the 5 plans: PSAC / SDP / SVP / SCMP / SQAP) | `CLAUDE.md` section 1 Problem 1-Pager + plan mode | CROSS-REF — 비자명한 작업마다 작성하는 단일 사전 spec이 5개 계획 문서를 대체한다. 별도 계획 문서는 없다 |
| Requirements (HLR / LLR) | Problem 1-Pager + `eval-harness` skill | CROSS-REF — 1-Pager가 의도를 담고, `eval-harness`가 수용 기준을 eval로 인코딩한다 |
| Coding standards | `coding-standards` skill | CROSS-REF — 이 skill이 naming, style, code smell의 단일 출처다 |
| Requirements-based testing (normal + robustness) | `tdd-workflow` skill | CROSS-REF — 추가로 happy-path뿐 아니라 robustness 및 boundary 케이스를 명시적으로 요구한다 |
| Test coverage thresholds | `tdd-workflow` skill | CROSS-REF — 수치는 해당 skill에 위임한다. 여기서 숫자를 다시 적지 않는다 |
| Structural coverage (statement / decision; MC/DC) | none today | NEW — statement 다음 decision/branch에서 gate한다. MC/DC는 지향 목표일 뿐이다 (아래 정책 참조) |
| Verification (review / analysis / test; verification of verification) | `verification-loop` skill + `code-reviewer` agent | CROSS-REF — 더해서 테스트가 tautological(자기충족적)이지 않은지 meta-check 한다 (검증의 검증) |
| Bidirectional traceability | none today | NEW — requirement에서 test, code로, 그리고 양방향으로 탐색 가능하게 연결한다 |
| Independence of verification (reviewer != author) | `CLAUDE.md` section 4 post-work review + `code-reviewer` agent | CROSS-REF — A-tier에서는 `code-reviewer` 위에 `assurance-auditor` agent를 추가한다 |
| Configuration management / baselines | `commit-rules` skill + `pull-request` skill | CROSS-REF — commit과 PR이 baseline이자 변경 통제 수단이다 |
| Problem reporting | issue tracker / ticket scheme | CROSS-REF — ticket이 문제 보고 기록이다. `commit-rules`에 따라 commit에서 참조한다 |
| SQA gate | `verification-loop` READY state + `pull-request` pre-PR checklist | CROSS-REF — 기존 gate들이 품질 sign-off다 |
| Assurance levels (DAL) | none today | NEW — criticality/assurance tier dial (A부터 E까지) |
| Derived requirements feedback | none today | NEW — `CLAUDE.md` section 3 self-improvement / `MEMORY.md` 패턴을 재사용하여 발견된 requirement를 다시 피드백한다 |
| Tool qualification (DO-330) for AI tools / checkers | `eval-harness` skill | CROSS-REF / EXTEND — 의존하는 AI 도구와 checker를 eval하고, 통과한 eval 스위트를 qualification 증거로 취급한다 |

## The Four Net-New Items

위의 모든 항목은 기존 담당자를 가리키는 포인터다. 예외는 `do-178c` 스킬이 직접 소유하는 다음 네 가지다:

1. **Criticality / assurance tier dial (DAL A through E).** 기존 gate들을 얼마나 강하게 밀어붙일지 설정하는 단일 dial이다. 새 gate를 추가하지 않고, `tdd-workflow`, `verification-loop`, `code-reviewer`, `assurance-auditor`, `security-review`의 rigor를 tier에 따라 올리거나 내린다.

2. **Bidirectional traceability.** 각 requirement에서 test, code로, 그리고 역방향으로 탐색 가능한 연결이다. 기존 어떤 skill도 이 매핑을 유지하지 않는다.

3. **Structural-coverage discipline.** statement coverage 다음 decision/branch coverage에서 gate한다. MC/DC는 A-tier의 이상일 뿐이다. 대부분의 JS/TS/Python 스택은 진정한 MC/DC를 측정할 수 없으므로 이를 gate 기준으로 삼지 않는다. 실제 coverage 수치는 여기가 아니라 `tdd-workflow`에 있다.

4. **Derived-requirement feedback.** 구현이나 테스트 과정에서 원래 spec에 없던 requirement(derived requirement)가 드러나면, 이를 spec에 다시 반영하고 criticality tier를 재평가한다. 이때 section 3 self-improvement / `MEMORY.md` 루프를 재사용한다.
