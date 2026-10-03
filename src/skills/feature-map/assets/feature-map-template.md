# <Project> Feature Map

Verified at `<short-sha>` · `<YYYY-MM-DD>` · Structural view: `<docs/CODEMAPS/INDEX.md or "none">`
Premises: <only what holds for every entry — "no server, state is localStorage" · "no tests" · "i18n: Lingui, msgid = source text" — or delete this line>
Note: <other documents this is not — "`FEATURES.md` at the root is the product roadmap" — or delete this line>

## How to use this map

On a vague report ("something in the sidebar is broken", "the memo field won't take input"):

1. **Where** did it happen? → Screen index → candidate feature IDs.
2. **What words** did they use? → Alias index → narrow the candidates. An alias marked `(ambiguous)` maps to several features — check each, or ask one question that tells them apart. A row marked `→ none` is something the product does not have.
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
| <영수증 출력> | → none; nearest: `<deposit-history>` |

## Features

<!-- Split mode (past ~500 lines): replace this section with a table
     `| ID | Purpose | File |` pointing at docs/features/<screen>.md.
     Entry files hold entries only — no Verified-at line. -->

### `<deposit-memo>` — <one-line purpose in product words>

- **Where**: <route / screen → region> (`<route path>`)
- **Aliases**: <inferred words users say>
- **UI strings**: "<exact placeholder>" (`<i18n key or source msgid>`), "<exact error text>" (`<i18n key or source msgid>`)
- **Controls**: "<button label>" → `<handlerName>`, "<input label>" → `<handlerName>`
- **Code path** (only the hops that exist):
  - UI: `<path/to/Component.tsx>`
  - State / handler: `<path/to/store-or-hook.ts>` → `<symbol>`
  - API: `<path/to/client.ts>` → `<METHOD /api/route>`
  - Server: `<path/to/handler.ts>`
  - Data: <table / collection / storage key> — written in `<path>` → `<symbol>`, read in `<path>` → `<symbol>`
- **Tests**: `<path/to/test>` — or `none`
- **Pitfalls**: <symptom> → <cause> — source: `<commit sha>` or `<path>` → `<symbol>`
- **Shares**: <shared module whose change breaks this feature too>
- **Related**: `<other-feature-id>`
