# <프로젝트> 기능맵

검증 시점 `<short-sha>` · `<YYYY-MM-DD>` · 구조 뷰: `<docs/CODEMAPS/INDEX.md 또는 "없음">`

## 이 맵 사용법

모호한 제보("사이드바에 뭐가 잘 안돼요", "입금 문구가 잘 입력이 안돼요")를 받으면:

1. **어디서** 일어났나? → 화면 인덱스 → 후보 기능 ID.
2. **어떤 단어**를 썼나? → 별칭 인덱스 → 후보 좁히기. `(ambiguous)` 표시된 별칭은 여러 기능에 걸린다 — 전부 확인하거나, 둘을 가르는 질문 하나만 한다.
3. **어떤 문구**를 봤나? → 항목에 적힌 실제 문자열로 grep.
4. 항목의 코드 경로를 위에서부터 연다. 디버깅 전에 **함정**부터 읽는다.
5. 어디에도 없으면 → 코드를 검색한 뒤, 빠진 기능·별칭·문구를 이 맵에 추가한다.

## 화면 인덱스

| 화면 / 영역 | 기능 ID |
|---|---|
| <좌측 사이드바> | `<sidebar-nav>`, `<sidebar-favorites>` |
| <입금 폼> | `<deposit-amount>`, `<deposit-memo>` |

## 별칭 인덱스

| 사용자 표현 | 기능 ID |
|---|---|
| <입금, deposit, 충전> | `<deposit-amount>` |
| <입금 문구, 메모, 받는 분 표시> | `<deposit-memo>` |
| <메뉴> (ambiguous) | `<sidebar-nav>`, `<header-menu>` |

## 기능

### `<deposit-memo>` — <제품 용어로 쓴 한 줄 목적>

- **위치**: <라우트 / 화면 → 영역> (`<route path>`)
- **별칭**: <사용자가 쓰는 말 (추정)>
- **UI 문구**: "<실제 placeholder>" (`<i18n.key>`), "<실제 에러 문구>" (`<i18n.key>`)
- **코드 경로**:
  - UI: `<path/to/Component.tsx>`
  - 상태 / 핸들러: `<path/to/store-or-hook.ts>`
  - API: `<path/to/client.ts>` → `<METHOD /api/route>`
  - 서버: `<path/to/handler.ts>`
  - 데이터: `<table / collection / key>`
- **테스트**: `<path/to/test>` — 또는 `none`
- **함정**: <알려진 실패 지점> — 근거: `<commit sha / 코드 주석 경로>`
- **공유 의존**: <바뀌면 이 기능도 깨지는 공용 모듈>
- **관련**: `<other-feature-id>`
