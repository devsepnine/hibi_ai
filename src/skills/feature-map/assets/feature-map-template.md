# <Project> Feature Map

Verified at `<short-sha>` · `<YYYY-MM-DD>` · Structural view: `<docs/CODEMAPS/INDEX.md or "none">`

## How to use this map

On a vague report ("something in the sidebar is broken", "the memo field won't take input"):

1. **Where** did it happen? → Screen index → candidate feature IDs.
2. **What words** did they use? → Alias index → narrow the candidates. An alias marked `(ambiguous)` maps to several features — check each, or ask one question that tells them apart.
3. **What text** did they see? → grep the literal strings listed in the entry.
4. Open the entry's code path top-down; read **Pitfalls** before debugging.
5. No match anywhere → search the code, then add the missing feature, alias, or string here.

## Screen index

| Screen / region | Feature IDs |
|---|---|
| <left sidebar> | `<sidebar-nav>`, `<sidebar-favorites>` |
| <deposit form> | `<deposit-amount>`, `<deposit-memo>` |

## Alias index

| User says | Feature IDs |
|---|---|
| <입금, deposit, 충전> | `<deposit-amount>` |
| <입금 문구, 메모, 받는 분 표시> | `<deposit-memo>` |
| <메뉴> (ambiguous) | `<sidebar-nav>`, `<header-menu>` |

## Features

### `<deposit-memo>` — <one-line purpose in product words>

- **Where**: <route / screen → region> (`<route path>`)
- **Aliases**: <inferred words users say>
- **UI strings**: "<exact placeholder>" (`<i18n.key>`), "<exact error text>" (`<i18n.key>`)
- **Code path**:
  - UI: `<path/to/Component.tsx>`
  - State / handler: `<path/to/store-or-hook.ts>`
  - API: `<path/to/client.ts>` → `<METHOD /api/route>`
  - Server: `<path/to/handler.ts>`
  - Data: `<table / collection / key>`
- **Tests**: `<path/to/test>` — or `none`
- **Pitfalls**: <known failure point> — source: `<commit sha / code comment path>`
- **Shares**: <shared module whose change breaks this feature too>
- **Related**: `<other-feature-id>`
