---
type: debug
status: open | resolved | archived
severity: low | medium | high | critical
created: YYYY-MM-DD
updated: YYYY-MM-DD
tags: [type/debug, project/<slug>, topic/<area>]
project: <slug>
resolved_in: null      # 수정 완료 시 "[[ADR-NNNN …]]" 또는 "PR #123"
related: []
aliases: []
---

# Debug — <one-line symptom> (YYYY-MM-DD)

> [!bug] Symptom
> 사용자(또는 모니터링)가 보는 것. 정확한 에러 메시지, 스택 트레이스
> 일부, 또는 동작 설명을 포함한다. 의역하지 말 것 — 나중에 검색할 때
> 문자 그대로의 신호가 중요하다.

## Environment

- 버전 / 커밋: `<sha>` 또는 `v1.9.3`
- OS / 런타임: `<macOS 15.4, Rust 1.85>`
- 재현율: `<항상 | 20% 플레이키 | 한 번>`
- 최초 발견: <어디서 / 누가 보고했는지>

## Timeline

| Time | What happened |
|------|---------------|
| HH:MM | 보고 도착 / 알림 발생 |
| HH:MM | 첫 가설: <X> |
| HH:MM | <체크>로 <X>를 배제 |
| HH:MM | <증거>로 가설 <Y> 확정 |
| HH:MM | 수정이 <PR #>에 반영됨; 검증 완료 |

## Hypotheses explored

### ✗ Hypothesis 1 — <name>

- 의심한 이유: …
- 검증 방법: …
- 결과: 배제 — 이유 … [evidence](<link>)

### ✓ Hypothesis 2 — <name>

- 의심한 이유: …
- 증거: `<log snippet / repro command>`
- 근본 원인: <한 단락 설명>

## Root cause

> [!info] One-paragraph RC
> 결함을 평이한 말로 한 단락 설명. 이로부터 ADR이 나온다면 링크한다:
> [[ADR-NNNN …]].

## Fix

- 변경 내용: `<file:line>` — before / after 설명.
- PR: <url> — <date>에 머지됨.
- 추가된 테스트: [[<test file or description>]] — 이 특정 경로에 대한
  회귀 커버리지.

## Prevention

- 더 일찍 잡았더라면 무엇이 잡았을까? (린트 규칙, 테스트, 알림, 리뷰
  체크리스트). 실행 가능한 [ ] 작업으로 변환하고 후속 항목을 링크한다.
  - [ ] <preventive action> — [[<tracking note>]]

## Related

- [[ADR-NNNN …]] 여기서 결정이 나왔다면
- [[<수정을 출시하는 릴리스 노트>]]
- [[<유사한 과거 디버그>]] 패턴 인식을 위해
