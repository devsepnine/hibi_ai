---
name: feature-map
description: Build a symptom-to-code feature map in docs/FEATURES.md, indexing screen areas, user words, and UI strings to files, tests, and pitfalls. Use when creating or refreshing one. 기능맵, 기능 지도, 증상으로 코드 찾기.
---

# Feature Map

Write `docs/FEATURES.md`: a lookup table that takes what a user or teammate
**says**: "something's off in the sidebar", "the deposit memo won't take my
typing", and lands an AI on the files that implement it, without a codebase
crawl first.

## Why this shape

Reports arrive in product vocabulary: where on the screen, what the label
said, what the user was trying to do. Code is organized by module. The gap
between the two is where an AI burns its first ten tool calls by grepping for
"sidebar", finding forty hits, reading the wrong ones. The map closes that gap
by indexing features the way people describe them: **by screen location, by
the words they use, and by the literal strings they see**.

That is what separates it from a codemap, which is owned by `doc-updater` and lives in `docs/CODEMAPS/`.
A codemap answers "how is this code organized?"; the feature map answers
"where does this complaint land?". Link the two; never copy one into the other.

The reader is an AI loading it on every vague report, so every line must earn
its tokens: paths and strings, not prose.

## Workflow

### 1. Preflight: new map or update

```bash
test -f docs/FEATURES.md && grep -m1 -E '^(Verified at|검증 시점)' docs/FEATURES.md
git rev-parse --short HEAD
find . -path ./node_modules -prune -o -iname 'FEATURES*.md' -print
```

- **Another `FEATURES.md` outside `docs/`**, such as a roadmap or a marketing list, is a
  different document. Leave it alone and say so in the header's `Note` line, so
  nobody merges the two later.
- **No map**: full build, steps 2 to 6.
- **Map exists**: update mode. See [Update mode](#update-mode).

Example phrases the user gives, such as "sidebar" or "deposit memo", illustrate the kind
of report, not a feature list. When the product has no such feature, say so in
the report; do not attach the word to an unrelated feature to make it fit.

### 2. Find features from the outside in

Start from what a user can see or invoke, not from the directory tree. The
tree reflects how developers split work, which is exactly the vocabulary users
do not have.

| Surface | Where it usually lives |
|---|---|
| Routes / screens | router config, `app/` or `pages/` dirs, navigation stacks |
| Persistent regions such as sidebar, header, tab bar, and modals | layout components, shell/frame files |
| Menus and nav items | nav config arrays, menu definitions |
| Visible copy | i18n resource files such as `locales/`, `*.json`, `*.po`, `*.arb`, and `strings.xml`, plus hard-coded JSX text |
| Errors and toasts | error-message constants, toast/notification calls |
| CLI / API products | command definitions, flag parsers, route handlers, error codes |

One feature = one thing a user would name as a unit, such as "deposit", "sidebar
favourites", or "login". Split when two parts break independently and users name
them differently; merge when users never tell them apart.

### 3. Trace each feature down to data

For each feature, follow the real call chain by reading code, from UI component →
event handler / state store → API client → server handler → persistence. Record
only the hops that exist: a client-only app stops at browser storage, a CLI may
have no UI hop. When no feature has a given hop, say it once in the header's
`Premises` line, such as "no server, all state is client-side", instead of leaving an
empty line under every entry. Stop where the chain enters shared
infrastructure such as the HTTP client or ORM base and note the shared dependency instead
of listing it under every feature.

Persistent state is where tracing most often stops short. Record **both** the
write and the read side, plus every bootstrap point that touches it before the
app mounts, such as an inline script in `index.html` / `app.html` or an `init()` called
from several components. A key or constant defined in two places is a pitfall
candidate: the two copies drift.

Every path and symbol you write must be confirmed in this run: the file opened,
or the symbol found by grep and read in context.

### 4. Harvest the vocabulary

This is the part that makes the map work. Collect four kinds of words per
feature and keep them apart, because they are used differently on lookup:

- **Literal UI strings**: exact label, button, placeholder, and error text as
  shipped, in double quotes, plus the identifier the code looks it up by: the
  key for key-based i18n, as in `deposit.memo.placeholder`, the source-language
  `msgid` for source-as-key libraries such as Lingui and gettext. These are grep anchors,
  so copy them verbatim; never paraphrase. Hard-coded text with no translation
  entry is marked `(untranslated)`. For an interpolated string, copy the form
  the source code holds, as in `Step ${index + 1}/${total}`, not the compiled catalog
  form `Step {0}/{1}`. The catalog form greps only in the locale files.
  When users see a translated UI, add the translation they actually read,
  as in `"Format" · ko "포맷"`. Reporters quote the screen, not the source. A string
  that itself contains `"` cannot be quoted; copy its longest quote-free segment.
- **Controls**: every button or input a user would name, with its label and the
  handler it calls, as in `"Copy"` → `copyHeader`. "The copy button does nothing" is
  among the most common reports, and it is unanswerable when the entry lists
  strings but not the control behind them. Labels carry the translation users
  see, like UI strings do.
- **Screen location**: the region a user would point at, such as "left sidebar",
  "top of the deposit form", and "settings → notifications".
- **User aliases**: how people actually say it: colloquial terms, the other
  language, as in `입금` / `deposit`, near-synonyms such as `충전` and `송금 메모`, and common
  misspellings. Aliases are inferred, so mark them as such; they are matched,
  not grepped.

When one alias fits several features, keep it on all of them and record the
collision in the alias index with an `(ambiguous)` marker. The lookup step
must ask or check both, not pick one at random. Only real collisions count: a
word belongs to a feature when a user of that feature would say it. After
writing the index, scan it for the same or near-identical word on different rows,
such as "한글 깨짐" / "한글 입력", and merge them into one `(ambiguous)` row.

Record expected-but-absent features too. When users will plausibly ask for
something the product does not have, such as a copy button on a screen that has none,
add an alias row `→ none; nearest: <feature-id>`, dropping `nearest` when nothing
comes close. Find them by comparing screens: a common control such as copy, download,
or clear that sibling screens have and one screen lacks gets a screen-scoped row,
for example `copy in the JSON editor → none; nearest: <feature-id>`. Without it, the lookup
opens the screen's entries one by one looking for a control that does not exist.

### 5. Record pitfalls only with evidence

A pitfall is a known failure point for that feature: IME composition swallowing
Korean input, a debounce that drops the last keystroke, a cache that serves a
stale balance. Write it as **symptom → cause**, as in "typing Korean in a cell drops
the last syllable → Enter handled before IME composition ends". A fact you
cannot phrase as a symptom, such as "render is debounced 300 ms", is a description,
not a pitfall. Each one cites one of two sources:

- **A past fix**: find candidates by commit subject, then open each one, because
  a subject match is not yet evidence:

  ```bash
  git log --no-merges --format='%h %s' -- "<file>" ... \
    | grep -iE '^[0-9a-f]+ (revert|(fix|hotfix)[(!: ])|버그|오류' | head -10
  git show --stat <sha>     # confirm it touched this feature's failure
  ```

  The filter favours recall. It matches the type at the start of the subject, such as `fix:`,
  `Fix x`, or git's own `Revert "…"`, or a Korean bug word anywhere, and
  `git show` does the precision. Match the subject, not the body: `--grep`
  searches whole messages, and in a Korean history nearly every body says `수정`. Pass the feature's files
  explicitly, since with an empty list the command searches the whole repo.
- **The code itself**: a comment, a guard, or a statement that produces the
  behavior, cited as `path` → `symbol`. Cite the statement that causes it; a
  behavior you would need to run the app to confirm is not a pitfall yet.

No source, no pitfall. An invented pitfall sends the next debugger down the
wrong path with the authority of documentation behind it.

### 6. Verify, then wire it in

Before reporting, run the four checks below, then the review bullets after
them. Set `SRC` to every root that holds code or locale catalogs, such as `src` and
`public/locales`. Never use `docs/`, which contains the strings being checked:

```bash
MAP="docs/FEATURES.md $(ls docs/features/*.md 2>/dev/null)"; SRC="src"

# every path exists — repo-relative, contains `/`; route URLs start with `/` and are skipped
grep -ohE '`[^`/ ][^` ]*/[^` ]*\.[A-Za-z0-9]{1,5}`' $MAP | tr -d '`' | sort -u \
  | while read -r p; do test -e "$p" || echo "MISSING $p"; done

# every literal UI string and control label greps in the source
grep -hE '^- \*\*(UI strings|Controls|UI 문구|컨트롤)\*\*' $MAP | grep -oE '"[^"]+"' | tr -d '"' | sort -u \
  | while IFS= read -r s; do grep -rqF -- "$s" $SRC || echo "MISSING-STRING $s"; done

# every cited symbol exists in its file — written as `path` → `symbol`
grep -ohE '`[^`/ ][^` ]*/[^` ]*\.[A-Za-z0-9]{1,5}` → `[A-Za-z_$][A-Za-z0-9_$]*`' $MAP | tr -d '`' | sort -u \
  | while read -r p _ sym; do grep -qF -- "$sym" "$p" 2>/dev/null || echo "MISSING-SYMBOL $p → $sym"; done

# every pitfall states a cause: `symptom → cause` before the `source:` part
grep -hE '^- \*\*(Pitfalls|함정)\*\*:' $MAP | grep -vE ': none$|: [^→]+ → .+, (source|근거): ' | sed 's/^/NO-CAUSE /'
```

- A missing string is paraphrased or stale: fix it, do not keep it.
- Write paths in full from the repo root; a shortened path fails the check.
- Every user word in an entry's pitfalls, such as "align" or "copy", appears in the alias
  index on a row that lists that entry. Otherwise the pitfall is unreachable from
  the very report it describes.
- Every feature has at least one test path, or says `tests: none` explicitly,
  unless the repo has no tests at all, which the `Premises` line says once. The
  gap is information.
- Record `Verified at <sha> · <YYYY-MM-DD>` in the `docs/FEATURES.md` header, the **only** place the SHA lives. Entry
  files under `docs/features/` carry none; two SHAs drift apart, and update mode
  cannot tell which one to trust.

Then make sure an AI finds the map when a report comes in. Add a one-line
pointer to the project's root `CLAUDE.md`, under whichever section covers
debugging, and create the file with that line when it does not exist. When
`AGENTS.md` exists, add the same line there. On update, keep the line and fix it
only if the map moved. Report the change in the reply:

```markdown
- Vague bug report, such as "X on the sidebar doesn't work"? Look it up in `docs/FEATURES.md` before searching the code.
```

Writing the map is a content change, so the post-work review gate in `CLAUDE.md`
applies.

## Update mode

Diff from the recorded SHA: `git diff --name-status -M <sha>..HEAD`.

- Entries whose files appear in the diff are suspect, so re-verify those. Renamed
  files, marked `R`, get their paths repointed, entries whose files were all deleted,
  marked `D`, are dropped, and new routes, screens, or UI strings become new entries.
- Do not rewrite an entry that still verifies. A rewrite of a correct entry
  only adds review noise.
- **Wide diff**, where most entries are suspect: run the step-6 checks over the whole
  map, and re-open pitfall evidence only where the cited file or commit is in
  the diff. Re-reading every pitfall when nothing under it moved is cost with
  no finding.
- **Recorded SHA unreachable**, as after a rebase or in a shallow clone: say so and re-verify
  every entry, rather than rebuilding silently.
- Report entries added, changed, and removed, then move the header SHA to HEAD.

## Document template

Use `assets/feature-map-template.md`. Write the map's prose in the language the
project's docs use, and keep the template's headings and field labels in
English, because the step-6 checks grep for them. A map written earlier with
Korean labels such as `검증 시점` still verifies, since the checks accept both.
Keep the section order: the header, holding the SHA and
optional `Premises` and `Note` lines, then the lookup protocol, which the AI reads before
anything else, then the two indexes, then the entries. Nothing else goes
between the header and the protocol.

**Size**: when the single file passes ~500 lines, keep `docs/FEATURES.md` as the
header, protocol, both indexes, and a feature list of `ID | purpose | file`, and
move entries out to `docs/features/<screen>.md`, with **one file per screen or
route** holding every entry on it. Line count decides, not feature count; the
cost being managed is what a lookup has to read. Group by screen because a
report names a screen and its entries are read together; a file per feature
leaves dozens of 15-line files and an extra open per lookup. The indexes stay in
one file because lookup scans them whole.

## Writing rules

- **IDs are stable kebab-case**, such as `deposit-memo`, so indexes and other docs can
  reference them across renames of the UI label.
- **Paths, not descriptions**: `src/features/deposit/MemoInput.tsx` beats
  "the memo input component".
- **One line per hop** in the code path; skip hops that are pure pass-through.
- **No line numbers**: they drift with every edit and nothing re-checks them.
  In a large file, name the symbol instead, as in `src/features/deposit/MemoInput.tsx` → `handleCompositionEnd`.
- **Unknowns stay unknown**: `tests: none` and `pitfalls: none` are honest; a
  guessed value is not.

## Related

| Need | Where |
|---|---|
| Structural codemap of modules and layers | `doc-updater` agent, `/update-codemaps` |
| What changed for QA in a git range | `qa-handoff` skill |
| Whether a module's dependencies are healthy | `dependency-design` skill |
