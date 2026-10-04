# <프로젝트> 기능맵

검증 시점 `<short-sha>` · `<YYYY-MM-DD>` · 구조 뷰: `<docs/CODEMAPS/INDEX.md 또는 "없음">`
전제: <모든 항목에 공통인 것만, 예: "서버 없음, 상태는 localStorage" · "테스트 없음" · "i18n: Lingui, msgid = 원문". 없으면 이 줄 삭제>
참고: <이 맵과 다른 문서, 예: "루트의 `FEATURES.md` 는 제품 로드맵". 없으면 이 줄 삭제>

## 이 맵 사용법

"사이드바에 뭐가 잘 안돼요", "입금 문구가 잘 입력이 안돼요" 같은 모호한 제보를 받으면:

1. **어디서** 일어났나? → 화면 인덱스 → 후보 기능 ID.
2. **어떤 단어**를 썼나? → 별칭 인덱스 → 후보 좁히기. `(ambiguous)` 표시된 별칭은 여러 기능에 걸린다. 전부 확인하거나, 둘을 가르는 질문 하나만 한다. `→ 없음` 행은 제품에 없는 기능이다.
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
| <메뉴> `(ambiguous)` | `<sidebar-nav>`, `<header-menu>` |
| <영수증 출력> | → 없음; 가장 가까운 기능: `<deposit-history>` |

## 기능

<!-- 분할 모드, ~500줄 초과: 이 섹션을 `| ID | 목적 | 파일 |` 표로 바꾸고
     docs/features/<화면>.md 를 가리킨다. 항목 파일에는 항목만 두고 검증 시점 줄은 넣지 않는다. -->

### `<deposit-memo>`: <제품 용어로 쓴 한 줄 목적>

- **위치**: <라우트 / 화면 → 영역>, `<route path>`
- **별칭**: <사용자가 쓰는 말, 추정>
- **UI 문구**: "<실제 placeholder>" → `<i18n 키 또는 원문 msgid>`, "<실제 에러 문구>" → `<i18n 키 또는 원문 msgid>`
- **컨트롤**: "<버튼 라벨>" → `<handlerName>`, "<입력 라벨>" → `<handlerName>`
- **코드 경로**, 실제 있는 단계만:
  - UI: `<path/to/Component.tsx>`
  - 상태 / 핸들러: `<path/to/store-or-hook.ts>` → `<symbol>`
  - API: `<path/to/client.ts>` → `<METHOD /api/route>`
  - 서버: `<path/to/handler.ts>`
  - 데이터: <테이블 / 컬렉션 / 저장 키>, 쓰기 `<path>` → `<symbol>`, 읽기 `<path>` → `<symbol>`
- **테스트**: `<path/to/test>` 또는 `none`
- **함정**: <증상> → <원인>, 근거: `<commit sha>` 또는 `<path>` → `<symbol>`
- **공유 의존**: <바뀌면 이 기능도 깨지는 공용 모듈>
- **관련**: `<other-feature-id>`
