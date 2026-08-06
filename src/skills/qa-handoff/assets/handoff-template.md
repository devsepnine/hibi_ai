# <Scope> QA Handoff — <YYYY-MM-DD>

Range: `<v1.14.0..HEAD>` · Commits: `<N>` · Prepared by: `<name>`
Build / version to test: `<1.14.1-rc3 on staging>` · Range interpretation: `<last 7 days, rolling>`

## At a glance

- **What changed**: <one sentence a non-developer can repeat in a standup>
- **Affected areas**: <screens / features / platforms>
- **Deploy notes**: <migration, config change, feature flag — or "none">
- **Test focus**: <the one or two things most worth checking>

## What changed (plain language)

| # | Change | What the user experiences | Commits |
|---|---|---|---|
| 1 | <change> | <observable difference, before → after> | `abc1234` |

## Internal changes — no verification needed

- <change> — <why it is invisible from outside> (`abc1234`)

## QA checklist

- [ ] **1. <title>** — priority: must-pass
  - Preconditions: <account / data / environment / OS>
  - Steps: 1) <action> 2) <action>
  - Expected: <observable result>
- [ ] **2. <title>** — priority: secondary
  - Preconditions: <...>
  - Steps: 1) <...>
  - Expected: <...>

## Regression watch

| Area | Why it is at risk | Suggested check |
|---|---|---|
| <untouched feature> | <shared module / config / migration it depends on> | <shortest check that would catch a break> |

## Glossary

| Term | Means |
|---|---|
| <technical term that could not be avoided> | <plain-language meaning> |

## Open questions for the developer

- <question> — ask: `<commit author>` · blocks: <which checklist item cannot be judged without it>
