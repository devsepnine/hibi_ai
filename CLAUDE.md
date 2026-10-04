# hibi-ai repository

Instructions for working on this repository. They are not shipped: the installer ships what is under `src/`, and `src/CLAUDE.md` is the always-on file users receive.

## Before you change code

- Vague bug report, such as "X on the sidebar doesn't work"? Look it up in `docs/FEATURES.md` before searching the code.
- Before adding a module, a file in a new place, or an import across units, check `docs/ARCHITECTURE.md`.

## Conventions

- Every markdown file under `src/` has a `-ko.md` Korean mirror beside it. Change both in the same edit; the installer never ships the mirror.
- Prose in `src/`, `docs/`, and this file uses no dashes and no parentheses; the `technical-writing` skill owns the rule.

## Before you commit

Run these from the repository root; the release workflow runs the same set before it builds:

```bash
python tools/lint-prose.py
python tools/lint-arch.py
cargo fmt --manifest-path tools/installer/Cargo.toml --check
cargo test --manifest-path tools/installer/Cargo.toml
```
