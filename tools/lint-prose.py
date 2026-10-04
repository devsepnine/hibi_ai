#!/usr/bin/env python3
"""Fail when distributed prose uses dash or parenthesis punctuation.

Banned outside code: em dash, en dash, a hyphen standing in for a dash, and
parentheses. Code fences, inline code, markdown link targets, wikilinks, and
YAML frontmatter keys are syntax, not prose, so they are exempt.

Usage: python tools/lint-prose.py [paths...]   (default: src docs README.md)
Exit status: 0 clean, 1 violations found.
"""
import re
import sys
from pathlib import Path

DEFAULT_ROOTS = ["src", "docs", "README.md"]
# The skill listing drops a description entirely once the combined budget
# overflows; docs/ARCHITECTURE.md arch-description-length caps each one.
MAX_DESCRIPTION = 220

INLINE_CODE = re.compile(r"(`+)(?:(?!\1).)+?\1")
WIKILINK = re.compile(r"\[\[[^\]]*\]\]")
LINK_TARGET = re.compile(r"\]\((?:[^()\s]|\([^()\s]*\))*\)")
FENCE = re.compile(r"^\s*(`{3,}|~{3,})")
LIST_MARKER = re.compile(r"^\s*(?:[-*+]|\d+[.)])\s+")

CHECKS = [
    ("em dash", re.compile("—")),
    ("en dash", re.compile("–")),
    ("hyphen as dash", re.compile(r"\S\s+-{1,3}\s+\S")),
    ("parenthesis", re.compile(r"[()]")),
]


def prose_of(line):
    line = INLINE_CODE.sub("", line)
    line = WIKILINK.sub("", line)
    line = LINK_TARGET.sub("]", line)
    return LIST_MARKER.sub("", line, count=1)


def lint_file(path):
    violations = []
    open_fence = None
    in_frontmatter = False
    in_block_description = False
    block_start, block_parts = 0, []

    def check_length(number, text):
        if len(text) > MAX_DESCRIPTION:
            violations.append((number, f"description over {MAX_DESCRIPTION} chars", text))

    def close_block():
        nonlocal in_block_description
        if in_block_description:
            check_length(block_start, " ".join(block_parts))
        in_block_description = False

    for number, raw in enumerate(path.read_text(encoding="utf-8-sig").splitlines(), 1):
        if number == 1 and raw.strip() == "---":
            in_frontmatter = True
            continue
        if in_frontmatter:
            if raw.strip() == "---":
                close_block()
                in_frontmatter = False
                continue
            if in_block_description and raw[:1] in (" ", "\t"):
                block_parts.append(raw.strip())
                prose = prose_of(raw)
                for label, pattern in CHECKS:
                    if pattern.search(prose):
                        violations.append((number, label, raw.strip()))
                continue
            close_block()
            key, _, value = raw.partition(":")
            if key.strip() != "description":
                continue
            raw = value.strip()
            if raw[:1] in (">", "|"):
                in_block_description = True
                block_start, block_parts = number, []
                continue
            # Rewriting a dash as a colon leaves ": " inside a plain scalar, which a
            # YAML parser rejects as a nested mapping; the value must be quoted.
            if raw and raw[0] not in "\"'" and ": " in raw:
                violations.append((number, "unquoted colon in description", raw))
            quoted = len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in "\"'"
            check_length(number, raw[1:-1] if quoted else raw)
        fence = FENCE.match(raw)
        if open_fence is None:
            if fence:
                open_fence = fence.group(1)
                continue
        else:
            # A fence closes only on the same character, at least as long, with no
            # info string, so a ```mermaid inside a ````markdown block stays inside.
            marker = fence.group(1) if fence else ""
            if marker[:1] == open_fence[:1] and len(marker) >= len(open_fence) and raw.strip() == marker:
                open_fence = None
            continue
        prose = prose_of(raw)
        for label, pattern in CHECKS:
            if pattern.search(prose):
                violations.append((number, label, raw.strip()))
    return violations


def markdown_files(roots):
    for root in roots:
        path = Path(root)
        if path.is_file():
            yield path
        elif path.is_dir():
            yield from sorted(path.rglob("*.md"))


def main(argv):
    roots = argv[1:] or DEFAULT_ROOTS
    total = 0
    for path in markdown_files(roots):
        for number, label, text in lint_file(path):
            total += 1
            print(f"{path.as_posix()}:{number}: {label}: {text[:120]}")
    if total:
        print(f"\n{total} violation(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    sys.exit(main(sys.argv))
