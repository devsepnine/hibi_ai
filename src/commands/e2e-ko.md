---
description: Generate and run end-to-end tests with Playwright. Creates test journeys, runs tests, captures screenshots/videos/traces, and uploads artifacts.
allowed-tools: Task, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# E2E Command

**e2e-runner** 에이전트(`~/.claude/agents/e2e-runner.md`)를 호출하여 Playwright E2E 테스트를 생성, 유지, 실행한다.

## Argument Parsing

입력 형식: `/e2e <journey description>` (자유 형식). 선택적 플래그:
- `--file <path>` 생성 대신 기존 spec 실행
- `--headed` / `--debug` Playwright로 전달
- `--repeat-each=N` 플레이크 탐지 (기본값: flake hunt 시 미설정 시 10)

설명도 `--file`도 없는 경우, 어떤 journey를 테스트할지 사용자에게 확인한다.

## Invocation Flow

1. 인자 파싱 → journey 설명 + 플래그.
2. Playwright 설정 위치 파악: `playwright.config.ts`, `tests/e2e/`, `tests/pages/`.
3. `Task(subagent_type="e2e-runner", prompt=<journey + repo context>)` 디스패치.
4. 생성/업데이트된 spec + 실행 결과 수신.
5. 아티팩트 존재 확인 (HTML report, JUnit XML, 실패 시 screenshots/videos/traces).
6. 최종 보고서 형식화 (아래 Result Format 참조).

## When to Use

- 핵심 사용자 journey (login, trading, payments, wallet, search)
- 프론트엔드↔백엔드 통합이 필요한 다단계 흐름
- 배포 전 회귀 테스트
- Flaky 테스트 분류 (`--repeat-each=10`)

## Core Playwright Settings (delegated to agent — do not duplicate)

에이전트가 POM 작성, 설정 튜닝, flake 격리를 담당한다. 참조:
[Playwright config](https://playwright.dev/docs/test-configuration), [POM guide](https://playwright.dev/docs/pom).

에이전트가 강제하는 최소 설정:

```typescript
use: { baseURL, trace: 'on-first-retry', screenshot: 'only-on-failure', video: 'retain-on-failure' }
retries: process.env.CI ? 2 : 0
reporter: [['html'], ['junit', { outputFile: 'playwright-results.xml' }]]
```

## Test Authoring Rules (must hold for every generated spec)

- [ ] Page Object Model (`tests/pages/*.ts`)
- [ ] `data-testid` 로케이터만 사용 — CSS 클래스/텍스트만으로 매칭 금지
- [ ] `waitForResponse` / `waitForLoadState` — `waitForTimeout` 사용 금지
- [ ] 금융/거래 흐름에는 `test.skip(process.env.NODE_ENV === 'production')`
- [ ] 의미 있는 체크포인트에서 스크린샷 촬영
- [ ] 하드코딩된 URL 금지 — 설정의 `baseURL` 사용

## Journey Coverage (PMX)

상세 패턴 (예시 하나):

**Market Search & View** — `/markets`로 이동 → `searchMarkets('election')` → `/api/markets/search` 200 대기 → `marketCards.count() > 0` 어설션 → 첫 항목 클릭 → URL `/markets/[slug]` + 차트 가시성 어설션. 결과와 상세 화면 스크린샷.

기타 핵심 journey (1줄): wallet connect (`window.ethereum` 모킹), market creation (인증 필요), trade (testnet only, NODE_ENV 가드), withdrawal, market resolution. 중요: profile updates, real-time price, filters, mobile layout.

## Result Format

```
E2E Test Results
================
Status:    [PASS|FAIL|FLAKY]
Total:     N tests
Passed:    N (P%)
Failed:    N
Flaky:     N
Duration:  Ts

Spec: tests/e2e/<area>/<flow>.spec.ts

Artifacts:
- playwright-report/index.html
- playwright-results.xml (JUnit)
- artifacts/*.png (screenshots)
- test-results/*.webm (videos, on failure)
- test-results/*.zip (traces, on failure)

View report: npx playwright show-report
```

실패 시 각 항목에 대해 `file:line`, 오류 메시지, 아티팩트 경로, 권장 수정안을 포함한다.

플레이키 테스트 시: 테스트별 통과율, 근본 원인 가설, 격리 권고(`test.fixme()` + 이슈 링크) 명시.

## Quick Commands (forwarded to Bash)

```bash
npx playwright test                          # all
npx playwright test <path>                   # single
npx playwright test --headed --debug         # visual debug
npx playwright test --repeat-each=10         # flake detection
npx playwright codegen <url>                 # record actions
npx playwright show-report                   # view HTML
```

## Safety Rules

- 프로덕션 대상 테스트는 절대 금지
- 거래/금융 spec은 반드시 `test.skip(NODE_ENV === 'production')`을 포함해야 함
- 테스트넷 지갑과 테스트 자금만 사용
- 생성된 spec을 자동 커밋하지 않는다 — 사용자 검토를 기다린다

## Integration

- `/plan` → `/e2e` 전 journey 식별
- `/tdd` → 단위 테스트 (더 빠르고 세분화)
- `/e2e` → 통합 / 사용자 journey
- `/code-review` → 생성된 테스트 품질 검토
