#!/usr/bin/env python3
"""Re-measure the skill-listing character budget.

Claude Code builds the model-facing skill listing under a hard character
budget. Over budget the truncation is **all-or-nothing per skill**: a skill
that does not fit keeps only `- name` and loses its description entirely, so
description-based auto-triggering stops working for it.

The formula, as the client computes it:

    listed  = description + " - " + when_to_use   (when when_to_use is set)
    entry   = len(name) + 4 + min(len(listed), 1536)
    F       = sum(entry) + (count - 1)
    budget  = floor(context_window * chars_per_token * budget_fraction)
            = floor(200000 * 4 * 0.01) = 8000

Two consequences worth keeping in mind while writing descriptions:

- Moving trigger vocabulary into `when_to_use` saves nothing; it is
  concatenated into the same measured string.
- Lengths count UTF-16 units, as JS `.length` does, so one Hangul syllable
  costs 1 -- the same as one ASCII letter. Korean vocabulary is budget-efficient.

The repository uses simple single-line frontmatter. With PyYAML installed,
YAML block scalars and other supported YAML syntax are decoded before counting.

Exit status: 0 within budget, 1 over budget, 2 usage error.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    yaml = None

CHARS_PER_TOKEN = 4
BUDGET_PERCENT = 1
ENTRY_OVERHEAD = 4
DESCRIPTION_CAP = 1536
AUTHORING_TARGET = 200


def utf16_len(text: str) -> int:
    """Count as the client does, so an astral character costs its real 2."""
    return len(text.encode("utf-16-le")) // 2


def simple_fields(header: str) -> dict[str, str]:
    """Read the repository's one-line strings when PyYAML is unavailable."""
    unsupported = "use Python with PyYAML for this frontmatter, e.g. uv run --with pyyaml python"
    fields = {}
    in_metadata = False
    for line in header.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        match = re.fullmatch(r"(  )?([\w-]+):(?:[ \t]+(.*))?", line)
        if not match:
            raise ValueError(unsupported)
        indent, key, value = match.groups()
        if indent and not in_metadata:
            raise ValueError(unsupported)
        value = (value or "").strip()
        if not indent:
            in_metadata = key == "metadata" and not value
            if in_metadata:
                continue
        quoted = re.fullmatch(r"""("(?:[^"\\]|\\.)*"|'(?:[^']|'')*')(?:\s+#.*)?""", value)
        if quoted:
            literal = quoted.group(1)
            value = json.loads(literal) if literal.startswith('"') else literal[1:-1].replace("''", "'")
        else:
            value = re.split(r"\s+#", value, maxsplit=1)[0]
            # A bare argument hint such as <file-or-pattern> is also a YAML string.
            plain_string = value[:1].isalpha() or re.fullmatch(r"<[\w|-]+>", value)
            if not plain_string or re.search(r":(?:\s|$)", value):
                raise ValueError(unsupported)
            if value.lower() in {"yes", "no", "true", "false", "on", "off", "null", "~"}:
                raise ValueError(unsupported)
        if not indent:
            fields[key] = value
    return fields


def parse_frontmatter(text: str) -> dict[str, str]:
    lines = text.splitlines(keepends=True)
    if not lines or lines[0].rstrip() != "---":
        raise ValueError("missing frontmatter opening delimiter")
    end = next((i for i in range(1, len(lines)) if lines[i].rstrip() == "---"), None)
    if end is None:
        raise ValueError("missing frontmatter closing delimiter")
    header = "".join(lines[1:end])
    if yaml is None:
        values = simple_fields(header)
    else:
        try:
            values = yaml.safe_load(header)
        except yaml.YAMLError as exc:
            raise ValueError(str(exc)) from exc
    if not isinstance(values, dict):
        raise ValueError("frontmatter must be a mapping")
    for field in ("name", "description", "when_to_use"):
        if field == "when_to_use" and field not in values:
            continue
        if not isinstance(values.get(field), str):
            raise ValueError(f"frontmatter {field} must be a string")
    return values


def listed_length(fields: dict[str, str]) -> int:
    listed = fields.get("description", "")
    when = fields.get("when_to_use", "")
    if when:
        listed = f"{listed} - {when}"
    return min(utf16_len(listed), DESCRIPTION_CAP)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("skills_dir", type=Path,
                    help="directory of skill directories, e.g. src/skills or ~/.claude/skills")
    ap.add_argument("--context-window", type=int, default=200000,
                    help="token context window the budget scales with (default 200000)")
    ap.add_argument("--quiet", action="store_true", help="totals only")
    args = ap.parse_args()

    if not args.skills_dir.is_dir():
        ap.error(f"{args.skills_dir} is not a directory")

    budget = args.context_window * CHARS_PER_TOKEN * BUDGET_PERCENT // 100
    if budget < 1:
        ap.error("--context-window must allow a skill listing budget of at least one character")
    rows = []
    for skill_md in sorted(args.skills_dir.glob("*/SKILL.md")):
        name = skill_md.parent.name
        try:
            fields = parse_frontmatter(skill_md.read_text(encoding="utf-8"))
        except (OSError, ValueError) as exc:
            ap.error(f"{skill_md}: {exc}")
        listed = listed_length(fields)
        rows.append({
            "name": name,
            "listed": listed,
            "entry": utf16_len(name) + ENTRY_OVERHEAD + listed,
            "declared": fields.get("name", ""),
            "has_when_to_use": bool(fields.get("when_to_use")),
        })

    if not rows:
        ap.error(f"no */SKILL.md under {args.skills_dir}")

    total = sum(r["entry"] for r in rows) + (len(rows) - 1)

    if not args.quiet:
        print(f"{'skill':<28}{'listed':>8}{'entry':>8}   flags")
        for row in sorted(rows, key=lambda r: -r["entry"]):
            flags = []
            if row["listed"] > AUTHORING_TARGET:
                flags.append(f"over {AUTHORING_TARGET}-char target")
            if row["declared"] and row["declared"] != row["name"]:
                flags.append(f"name: {row['declared']} != dir")
            if row["has_when_to_use"]:
                flags.append("when_to_use counts toward budget")
            print(f"{row['name']:<28}{row['listed']:>8}{row['entry']:>8}   {', '.join(flags)}")
        print()

    used = total / budget * 100
    print(f"skills={len(rows)}  F={total}  budget={budget}  used={used:.0f}%")
    if total > budget:
        print(f"OVER BUDGET by {total - budget} chars.")
        print("Truncation is all-or-nothing per skill: whichever skills do not fit keep")
        print("only `- name` and lose their descriptions, so they stop auto-triggering.")
        print("Cut the longest descriptions above until F fits.")
        return 1
    print(f"within budget, {budget - total} chars spare")
    return 0


if __name__ == "__main__":
    sys.exit(main())
