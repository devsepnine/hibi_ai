---
name: feature-map
description: Build a symptom-to-code feature map (docs/FEATURES.md) — screen areas, user words, UI strings → files, tests, pitfalls. Use when creating or refreshing one. 기능맵, 기능 지도, 증상으로 코드 찾기.
---

# Feature Map

Write `docs/FEATURES.md`: a lookup table that takes what a user or teammate
**says** — "something's off in the sidebar", "the deposit memo won't take my
typing" — and lands an AI on the files that implement it, without a codebase
crawl first.

## Why this shape

Reports arrive in product vocabulary: where on the screen, what the label
said, what the user was trying to do. Code is organized by module. The gap
between the two is where an AI burns its first ten tool calls — grepping for
"sidebar", finding forty hits, reading the wrong ones. The map closes that gap
by indexing features the way people describe them: **by screen location, by
the words they use, and by the literal strings they see**.

That is what separates it from a codemap (`doc-updater`, `docs/CODEMAPS/`).
A codemap answers "how is this code organized?"; the feature map answers
"where does this complaint land?". Link the two; never copy one into the other.

The reader is an AI loading it on every vague report, so every line must earn
its tokens: paths and strings, not prose.

## Workflow

### 1. Preflight — new map or update

```bash
test -f docs/FEATURES.md && grep -m1 -E '^(Verified at|검증 시점)' docs/FEATURES.md
git rev-parse --short HEAD
```

- **No map**: full build (steps 2–6).
- **Map exists**: update mode. Take the recorded SHA and diff from it:
  `git diff --name-status -M <sha>..HEAD`. Entries whose files appear in the
  diff are suspect — re-verify those; renamed (`R`) files get their paths
  repointed, entries whose files were all deleted (`D`) are dropped, and new
  routes, screens, or UI strings become new entries. Do not regenerate untouched
  entries — rewriting a correct entry only adds review noise. Report which
  entries were added, changed, and removed.
- **Recorded SHA unreachable** (rebased, shallow clone): say so and fall back
  to a full re-verify of every path, not a silent rebuild.

### 2. Find features from the outside in

Start from what a user can see or invoke, not from the directory tree — the
tree reflects how developers split work, which is exactly the vocabulary users
do not have.

| Surface | Where it usually lives |
|---|---|
| Routes / screens | router config, `app/` or `pages/` dirs, navigation stacks |
| Persistent regions (sidebar, header, tab bar, modals) | layout components, shell/frame files |
| Menus and nav items | nav config arrays, menu definitions |
| Visible copy | i18n resource files (`locales/`, `*.json`, `*.arb`, `strings.xml`), hard-coded JSX text |
| Errors and toasts | error-message constants, toast/notification calls |
| CLI / API products | command definitions, flag parsers, route handlers, error codes |

One feature = one thing a user would name as a unit ("deposit", "sidebar
favourites", "login"). Split when two parts break independently and users name
them differently; merge when users never tell them apart.

### 3. Trace each feature down to data

For each feature, follow the real call chain by reading code — UI component →
event handler / state store → API client → server handler → persistence. Record
the files at each hop. Stop where the chain enters shared infrastructure (HTTP
client, ORM base) and note the shared dependency instead of listing it under
every feature.

Every path you write must come from a file you opened in this run.

### 4. Harvest the vocabulary

This is the part that makes the map work. Collect three kinds of words per
feature and keep them apart, because they are used differently on lookup:

- **Literal UI strings** — exact label, button, placeholder, and error text as
  shipped, plus the i18n key. These are grep anchors, so copy them verbatim
  from the source; never paraphrase.
- **Screen location** — the region a user would point at ("left sidebar",
  "top of the deposit form", "settings → notifications").
- **User aliases** — how people actually say it: colloquial terms, the other
  language (`입금` / `deposit`), near-synonyms (`충전`, `송금 메모`), common
  misspellings. Aliases are inferred, so mark them as such; they are matched,
  not grepped.

When one alias fits several features, keep it on all of them and record the
collision in the alias index with an `(ambiguous)` marker — the lookup step
must ask or check both, not pick one at random.

### 5. Record pitfalls only with evidence

A pitfall is a known failure point for that feature: IME composition swallowing
Korean input, a debounce that drops the last keystroke, a cache that serves a
stale balance. Each one needs a source you can point at — a code comment, a
guard in the code, a past fix:

```bash
git log --oneline -i -E --grep='fix|bug|revert|hotfix|수정|버그' -- "<file>" ... | head -10
```

Pass the feature's files explicitly — with an empty list the command searches
the whole repo and every fix looks relevant.

No source, no pitfall. An invented pitfall sends the next debugger down the
wrong path with the authority of documentation behind it.

### 6. Verify, then wire it in

Before reporting:

```bash
# every path in the map exists (a path needs a `/`, so i18n keys like `deposit.memo` are skipped)
grep -ohE '`[^` ]*/[^` ]*\.[A-Za-z0-9]{1,5}`' docs/FEATURES.md docs/features/*.md 2>/dev/null \
  | tr -d '`' | sort -u | while read -r p; do test -e "$p" || echo "MISSING $p"; done
```

- Every literal UI string greps in the source (`grep -rF "<string>"`); one that
  does not is either paraphrased or stale — fix it, do not keep it.
- Every feature has at least one test path, or says `tests: none` explicitly.
  The gap is information.
- Record `Verified at <sha> · <YYYY-MM-DD>` (`검증 시점` in the Korean template)
  in the header — update mode reads it back.

Then make sure an AI finds the map when a report comes in. Propose a one-line
pointer in the project's `CLAUDE.md` / `AGENTS.md`, under whichever section
covers debugging, and add it when the user agrees:

```markdown
- Vague bug report ("X on the sidebar doesn't work")? Look it up in `docs/FEATURES.md` before searching the code.
```

Writing the map is a content change, so the post-work review gate in `CLAUDE.md`
applies.

## Document template

Use `assets/feature-map-template.md`. Keep the section order: the lookup
protocol first (the AI reads it before anything else), then the two indexes,
then the entries.

**Size**: past ~30 features or ~400 lines, keep `docs/FEATURES.md` as the
protocol plus both indexes, and move entries to `docs/features/<feature-id>.md`.
The indexes stay in one file because lookup scans them whole; entries are opened
one at a time.

## Writing rules

- **IDs are stable kebab-case** (`deposit-memo`), so indexes and other docs can
  reference them across renames of the UI label.
- **Paths, not descriptions** — `src/features/deposit/MemoInput.tsx` beats
  "the memo input component".
- **One line per hop** in the code path; skip hops that are pure pass-through.
- **No line numbers** — they drift with every edit and nothing re-checks them.
  In a large file, name the symbol instead (`MemoInput.tsx` → `handleCompositionEnd`).
- **Unknowns stay unknown** — `tests: none` and `pitfalls: none` are honest; a
  guessed value is not.

## Related

| Need | Where |
|---|---|
| Structural codemap (modules, layers) | `doc-updater` agent, `/update-codemaps` |
| What changed for QA in a git range | `qa-handoff` skill |
| Whether a module's dependencies are healthy | `dependency-design` skill |
