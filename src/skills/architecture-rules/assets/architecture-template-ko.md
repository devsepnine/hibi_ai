# <Project> Architecture Rules

Verified at `<short-sha>` · `<YYYY-MM-DD>` · Structural view: `<docs/CODEMAPS/INDEX.md or "none">` · Feature map: `<docs/FEATURES.md or "none">`
Premises: <코드베이스 전체에 성립하는 사실. 예: "Next.js 앱 하나, workspace 없음". 없으면 이 줄을 지운다>

## How to use these rules

파일을 새 위치에 추가하거나, 새 모듈을 만들거나, unit을 넘는 import를 추가하기 전에:

1. **Units**에서 지금 있는 unit과 import하려는 unit을 찾는다.
2. 두 unit 중 하나라도 이름이 나오는 규칙을 모두 확인한다. `MUST` 위반은 결함이다. `SHOULD` 위반은 PR에 이유를 적는다.
3. **Known violations**에 있는 파일은 선례가 아니다. 그 패턴을 따라 하지 않는다.

## Units

| Unit | Path | Responsibility | May depend on |
|---|---|---|---|
| `<unit>` | `<path/>` | <한 문장> | `<unit>`, `<unit>` |

## Rules

### `arch-<kebab-id>`

- **Level**: MUST
- **Rule**: <diff에 대고 확인할 수 있는 한 문장>
- **Why**: <Decisions의 사용자 답변, 또는 근거>
- **Evidence**: <M개 중 N개 파일> · `<측정한 커맨드>`
- **Exceptions**: `<path>`, <이유> · 또는 none
- **Enforced by**: `<tool rule id>` · draft: `<tool>` · review

## Known violations

| Path | Rule | Decision |
|---|---|---|
| `<path>` | `arch-<id>` | fix · accepted, <이유> |

## Decisions

| Date | Question | Answer |
|---|---|---|
| `<YYYY-MM-DD>` | <물어본 회색 지대> | <답과 그 결과> |
