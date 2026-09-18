# CLAUDE.md — Orchestration Flow

always-on 워크플로우와 의사결정 절차를 정의한다. 상세하고 상황 의존적인 정책은 **Skill**(필요할 때 로드)이 담당한다 — 문서 하단의 정책 라우팅 표를 참조한다.

## Pre-work checklist

- **현재 상태 파악**: `git status`로 기존 변경사항을 드러낸다
- **파일 구조 이해**: 관련 코드와 의존성을 읽는다
- **범위 준수**: 요청 범위를 벗어난 것은 바꾸지 않는다
- **간단한 요청**: 즉시 실행한다. 복잡한 요청만 계획 단계를 거친다.

## Thinking and response language policy (CRITICAL)

- **사고(thinking) 단계**: 영어로 추론 — 더 정확한 reasoning
- **응답(output)**: 한국어 — 사용자 가독성 우선
- **코드·명령어·기술 용어·에러 메시지**: 원문(영어) 유지

## Workflow orchestration

### 1. Plan-mode default
- 3단계 이상 또는 아키텍처 결정이 걸린 작업 → 플랜 모드 진입
- 의도에서 벗어나기 시작하면 즉시 STOP하고 재계획한다
- 모호함을 줄이기 위해 사전에 상세 스펙을 쓴다 (질의 문답 먼저)
- 복잡하거나 불명확한 문제는 **Problem 1-Pager**를 먼저 작성한다: Background / Problem / Goal / Non-goals / Constraints — 항목이 모호하면 인터뷰를 요청한다

### 2. Subagent strategy
- 메인 컨텍스트를 깨끗하게 유지하려면 서브에이전트를 쓴다 — 단 Opus 5는 이미 위임 성향이 강하므로 부추기지 않는다: 도구 호출 몇 번으로 직접 끝낼 일에는 위임하지 않는다
- 리서치·탐색·병렬 분석은 서브에이전트에 위임한다. 위임했으면 그 결과를 받아들이고 다시 도출하지 않는다
- 서브에이전트당 작업 하나 (Anthropic 가이드: low effort + 명시적 체크리스트)
- 팀 결과 종합이 끝나면 더 필요 없는 팀메이트에게 shutdown을 요청한다 — idle 팀메이트는 리드 세션이 끝날 때까지 살아 있다 (주소 지정 가능, 토큰 비용 없음)

### 3. Self-improvement loop
- 사용자 수정이 있을 때마다 그 패턴을 `MEMORY.md`에 기록한다
- 같은 실수를 막는 규칙을 쓰고 — 즉시 적용한다
- 세션 시작 시 관련 레슨을 검토한다
- 이 프로젝트를 넘어 일반화되는 교훈은 배포 설정에 반영하자고 제안하고, 사용자가 동의하면 `/upstream-pr` 커맨드를 실행한다 — 이 경로를 소유한 `pull-request` 스킬을 로드한다. 개인적인 것은 `MEMORY.md`에 남긴다

### 4. Verify before completion
- 작동을 증명하지 않고 작업을 완료로 표시하지 않는다
- 자문한다: "시니어 엔지니어가 이걸 승인할까?"
- 테스트를 실행하고, 로그를 확인하고, 정확성을 입증한다
- **작업 후 필수 리뷰**: 코드/콘텐츠를 변경한 뒤 완료를 보고하기 전에 diff를 리뷰한다 — 변경된 파일에 `code-reviewer` 에이전트(또는 `/code-review`)를 실행하고, 의존성·결합도·모듈·모노레포 변경은 `dependency-design` skill도 적용한다. 각 발견사항은 반영하거나 명시적으로 보류한다. 순수 대화나 사소한 비코드 편집만 예외다.

### 5. Pursue elegance (with balance)
- 자명하지 않은 변경에는 자문한다: "더 우아한 방법이 없을까?"
- 수정이 임시방편처럼 느껴진다면: "지금 알게 된 모든 것을 고려해 명백하고 명료한 해법을 구현한다"
- 간단하고 명확한 변경에는 이 단계를 생략한다 — 과잉 설계 금지
- **판단 기준**: "3개월 후에 내가 이 코드를 이해할 수 있을까?"
- **리팩토링 신호**: 같은 패턴이 3번 반복될 때 (Rule of Three)

### 6. Autonomous bug fixes
- 버그 리포트를 받으면 그냥 고친다 — 단계별 지시를 구하지 않는다
- 로그·에러·실패한 테스트를 직접 추적한다
- **근본 원인 우선**: 증상만 덮지 않는다
- **재발 방지**: 같은 유형의 버그가 다른 곳에 있는지 확인한다

### 7. Parallel execution principle
- 독립적인 작업은 항상 병렬로 실행한다 (단일 메시지에서 `Agent` 여러 번 호출)
- 의존성이 없는 3개의 무관한 분석을 순차 실행하지 않는다

### 8. Git and change safety (CRITICAL)
- **절대 금지**: 사용자가 명시적으로 요청하지 않은 `commit`, `push`, `gh pr create`, 브랜치 전략 변경 — 기준은 아래 "Absolute commit and push rules"에 있다
- **기존 변경 보호**: 사용자의 변경을 조용히 되돌리지 않는다
- **예상치 못한 변경 감지**: 내가 만들지 않은 변경을 발견하면 멈추고 확인한다
- **파괴적 명령**: `reset --hard`, `rm -rf`, `push --force`는 명시적 승인이 필요하다

### 9. Assurance level & traceability (DO-178C) — always-on
- **먼저 분류한다**: 작업 시작 시 최악의 blast radius 기준으로 criticality tier(A–E)를 부여한다. 이것이 master dial이다 — 아래 게이트들의 강도를 조절할 뿐, 별도 프로세스를 추가하지 않는다.
  - A (Catastrophic): auth, 결제, 암호화, 데이터 마이그레이션/삭제, 비가역적인 모든 것
  - B (Hazardous): 핵심 비즈니스 로직, 공개 API 계약, 영속 상태
  - C (Major): 내부 기능, 대시보드, 비핵심 엔드포인트 · D (Minor): 로깅, 카피, 스타일 · E (No effect): 폐기용 스크립트, 스파이크
- **tier가 기존 게이트를 다이얼한다** (새 SSOT 없음): 커버리지 → `tdd-workflow`; 검증 깊이 → 작업 후 리뷰 게이트 + `verification-loop`; 결합도 → `dependency-design`; 보안 sign-off → `security-review`. A/B는 최대로 올리고, D/E는 생략할 수 있다.
- **양방향 추적성 (A/B)**: 모든 요구사항(Problem 1-Pager / eval / ticket)이 코드와 테스트에 매핑되고, 변경된 모든 함수는 요구사항으로 역추적된다. orphan code와 미검증 요구사항을 표시한다.
- **독립 검증 (A/B)**: 구현자 ≠ 유일한 검증자 — 작업 후 필수 `code-reviewer` 리뷰 게이트가 이미 이를 강제한다. A-tier는 `assurance-auditor` 에이전트도 실행하고 사람 리뷰를 요구한다.
- **파생 요구 피드백**: 명세가 요청하지 않은 동작(retry, cache, error code, default)을 추가했다면 명세 오너에게 표면화한다 — 조용히 묻어두지 않는다.
- 전체 방법론: `do-178c` skill (`/do-178c`).

## Effort × model policy

Anthropic Opus 5 가이드 기준.

| Effort | 모델 | 용도 |
|---|---|---|
| `low` | `claude-haiku-4-5` | 단일 도구 체크리스트, 좁은 범위 (서브에이전트, 분류, 빠른 조회) |
| `medium` | `claude-sonnet-5` | 균형형 — 도구 호출 + 일부 추론 |
| `high` | `claude-sonnet-5` | 복잡한 추론, 신중한 판단 |
| `xhigh` | `claude-opus-5` | 코딩, 탐색, 다단계 (반복 도구 호출, 깊은 검색) |
| `max` | `claude-fable-5` | 진정한 최전선만 — 최고난도 장기 작업 (프리미엄 $10/$50 과금, 옵트인; 일반 워크로드 비권장) |

**핵심 원칙**: *"프롬프트로 우회하지 말고 — effort를 올려라."* Opus 5는 effort를 엄격히 따른다. 낮은 effort에서는 요청된 범위만 하고 그 이상은 하지 않는다 — 그리고 Opus 5의 `low`/`medium`은 체급 이상으로 강하므로, 평가가 유지되는 선까지 낮춰 쓴다.

**모델 선택**: 빈번한 경량 워커는 Haiku 4.5; 메인 개발·도구 중심 작업은 Sonnet 5; 깊은 추론과 장시간(30분+) 에이전트 작업은 Opus 5; Fable 5는 최고난도 프론티어 작업에 명시적으로 선택할 때만 (기본 업그레이드 경로가 아니다).

**낮은 effort에서의 도구 사용**: 호출을 결합하고 더 적게 쓰고 직접 행동한다 → 간결한 확인.
**높은 effort에서의 도구 사용**: 행동 전에 계획을 설명하고, 더 많이 호출하고, 상세히 요약한다.

## Agent routing

에이전트는 격리된 워커다(자체 컨텍스트 윈도우, 제한된 도구) — 메인 컨텍스트를 깨끗하게 유지하고 병렬화하기 위해 쓴다. 서브에이전트는 기본적으로 **`low`/`medium` effort + 명시적 체크리스트**이고, agentic 탐색(다단계 검색, 반복 도구 호출)에만 `xhigh`로 올린다.

| 트리거 | 에이전트 | Effort / 모델 |
|---|---|---|
| 복잡한 기능 / 리팩토링 계획 | built-in `Plan` 에이전트 | high / sonnet-5 |
| 아키텍처 결정 | `architect` | xhigh / opus-5 |
| 새 기능 또는 버그 수정 (테스트 우선) | `tdd-guide` | medium / sonnet-5 |
| 코드 작성 직후, 그리고 `security-review` skill을 통한 커밋 전 보안 점검(시크릿, injection, auth) | `code-reviewer` | medium / sonnet-5 |
| 빌드 / 타입 실패 | `build-error-resolver` | medium / sonnet-5 |
| 핵심 사용자 흐름 | `e2e-runner` | xhigh / sonnet-5 |
| dead code 정리 | `refactor-cleaner` | xhigh / sonnet-5 |
| 문서화 | `doc-updater` | xhigh / opus-5 |
| 독립 보증 / 추적성 감사 (A/B-tier) | `assurance-auditor` | high / sonnet-5 |

**병렬 실행**: 독립적인 에이전트는 단일 메시지에서 함께 띄운다 (`Agent` 여러 번 호출). 무관한 분석을 순차 실행하지 않는다.
**다관점 분석**: 복잡한 문제는 초점이 다른 서브에이전트로 쪼갠다 (사실 / 시니어 엔지니어 / 보안 / 일관성 / 중복), 각각 범위 하나씩.

## Completion report format

1. **변경 사항**: 파일 경로와 라인 번호
2. **변경 이유**: 근거
3. **검증**: 어떻게 작동을 증명했는지 (테스트 / 빌드 결과)
4. **다음 단계**: 자연스러운 후속 작업을 번호 목록으로

**예시:**
```
Changed: src/auth/login.ts:42-58 — login validation logic
Why: empty email caused a server error → added client-side validation
Verified:
  - unit tests pass (`npm test auth.test.ts`)
  - E2E pass (happy path + empty input)
Next:
  1. Apply the same pattern to password validation
  2. Add multi-language error messages
```

## Core code-quality principles

- **단순함 우선**: 최소 변경. YAGNI / KISS.
- **게으름 금지**: 근본 원인 분석. 임시 수정 없음. 시니어 수준의 기준.
- **최소 blast radius**: 필요한 것만 바꾼다. 사이드 이펙트를 피한다.
- **최소 두 가지 대안을 비교한다** → 트레이드오프를 진술한다 → 되돌림 가능성을 확인한다.
- **작게 유지한다**: 작업·커밋·PR을 작게. 가정은 Issue/PR/ADR에 기록한다.
- **입력을 검증하고 출력을 인코딩한다**: 검증되지 않은 입력을 신뢰하지 않는다.
- **추상화보다 이름**: 의도를 드러내는 이름, 성급한 추상화 금지.
- **코드가 곧 명세다**: *무엇*은 이름·타입·구조가 담고, 주석은 코드가 표현할 수 없는 *왜*에만 존재한다. 절대 규칙은 아래 "Absolute comment rules", 전체 규칙은 `coding-standards`.

## Absolute commit and push rules (always apply — skill carries the rest)

`commit-rules`·`pull-request` skill이 로드되기 전이라도 다음은 협상 불가다:
- **`commit`, `push`, `gh pr create`는 그 커밋·푸시·PR 각각에 대한 명시적 요청이 있을 때만 실행한다.** 작업이 끝났다는 사실, 게이트 통과, 다음 단계가 명백하다는 판단은 요청이 아니다. 요청이 없으면 트리를 그대로 두고, 무엇이 준비됐는지 보고하고, 묻는다.
- **이 셋에는 포괄적 사전 승인이 없다.** 이전의 "알아서 해줘" / "just handle it", 직전 커밋이나 푸시에 대한 승인, 승인받은 계획, 명령을 자동 승인하는 권한 모드 — 어느 것도 그 요청이 아니다. 코드를 쓸 권한은 결코 그것을 내보낼 권한이 아니다. "사용자가 당연히 원할 테니까"라는 판단으로 실행하는 것이 이 규칙이 막으려는 바로 그 실패다.
- 사용자가 방금 입력한 슬래시 커맨드는 그 턴에 한해, 그 커맨드가 지칭하는 행위 하나에 대한 요청이며 그 이상은 아니다: `/commit`은 그 커밋을 승인하고, `/pull-request`·`/upstream-pr`은 `push`도 `gh pr create`도 승인하지 않으며 이 둘은 별도의 승인이 필요하다 — `pull-request` skill 참조.
- **이모지 금지, 생성 마커 금지** (`Co-Authored-By`, "Generated with Claude Code" 등)
- 형식: `<type>: [<ticket>] <title>`, 브랜치와 이력에 티켓이 없으면 `<type>: <title>` — 전체 컨벤션은 `commit-rules` skill.

## Absolute comment rules (always apply — skill carries the rest)

**코드가 곧 명세다.** 이름·타입·구조가 코드가 *무엇*을 하는지 표현한다. 주석은 코드가 말할 수 없는 것 — *왜*: 의도, 제약, 트레이드오프, 불변식, 외부 맥락 — 에만 존재한다. `coding-standards` skill이 로드되기 전이라도, 코드를 쓰거나 고칠 때 다음은 협상 불가다:

- **'무엇'을 설명하는 주석 → 대신 리팩토링한다** (이름 변경, 함수/상수 추출). 코드를 되풀이하는 주석은 문서가 아니라 결함이다.
- **과잉 주석 금지**: 기본값은 주석 없음 — 의미 없는 주석은 정작 중요한 소수를 묻어버리는 잡음이다.
- **절대 금지**: 코드 나열식 서술, 낡은 주석, 주석 처리된 코드, 변경 이력 주석(`// fixed 2026-01-02`), 이모지.
- **주석은 자신이 설명하는 코드를 따라간다**: 추출/분리/이름 변경은 주석을 함께 옮기고 남은 쪽을 다시 읽는다 — 엉뚱한 선언에 doc을 남기거나 한 선언에 doc 블록을 두 개 쌓아두지 않는다. 그 다음 옛 이름을 트리 전체에서 grep한다: 낡은 주석은 diff에 나타나지 않은 파일에 있는 경우가 많다.
- **검증할 수 있는 것만 주장한다**: *왜* 주석은 시스템에 대한 사실을 주장하는 것이다. 그것을 참으로 만드는 코드 경로·설정·외부 근거를 짚거나, 뒷받침 가능한 더 좁은 주장만 쓴다 — 지어낸 이유는 코드가 옳더라도 결함이다.
- **언어**: 대상 파일의 기존 주석 언어를 따른다 (응답 언어 정책은 응답에만 적용되고 코드 주석에는 적용되지 않는다).
- **완료 보고 전 자가 점검**: 추가하거나 손댄 모든 주석 — 그리고 이번 변경이 다른 곳에서 무효화한 주석까지 — 이 위 규칙과 skill의 Comments 섹션 전체(두괄식/BLUF, 같은 편집에서 갱신, 주석 유지보수)를 통과한다 — 위반은 실패한 테스트처럼 취급한다.

## Policy routing (DRY — each policy has ONE source of truth)

상세 정책은 전부 **Skill**이다: 트리거될 때만 로드되어 always-on 컨텍스트를 가볍게 유지한다. 상황이 맞으면 skill(또는 그 `/command`)을 호출한다.

| 정책 | 단일 출처(SSOT) | 로드 방법 |
|---|---|---|
| 커밋 컨벤션 | `commit-rules` skill | `/commit` 또는 git commit 시 트리거 |
| PR 가이드 · 업스트림 설정 기여 | `pull-request` skill | `/pull-request`, `/upstream-pr` 또는 PR 작업 시 트리거 |
| 보안 규칙 / OWASP | `security-review` skill | `/security-review` 또는 인증·입력·시크릿 시 트리거 |
| 테스트 & TDD | `tdd-workflow` skill | `/tdd` 또는 신규 기능·버그 수정 시 트리거 |
| 코딩 스타일 / 클린 코드 | `coding-standards` skill | 코드 리뷰·작성 시 트리거 |
| 의존성 / 결합 설계 | `dependency-design` skill | `/deps` 또는 모듈·결합도·의존성·모노레포 설계 시 트리거 |
| 빌드 & 타입 에러 | `verification-loop` skill | `/verify`, `/build-fix` |
| 보증 수준 / 추적성 (안전필수) | `do-178c` skill | `/do-178c` 또는 안전필수·고위험·보증수준·추적성 작업 시 트리거 |

필요할 때 읽는 참조 (`coding-standards` skill 내부):
- **코드 임계값 (LOC, 복잡도)**: `references/code-thresholds.md`
- **리뷰 체크리스트 (SOLID, severity, concurrency, cross-platform)**: `references/review-checklist.md`
- **공통 TS 패턴**: `references/patterns.md`
