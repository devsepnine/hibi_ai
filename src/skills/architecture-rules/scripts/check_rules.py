"""Check docs/ARCHITECTURE.md from the repository root."""

import re
import sys
from collections import defaultdict
from pathlib import Path


def findings(text):
    paths = re.findall(r"`([^`<> /\n][^`<> \n]*/[^`<> \n]*)`", text)
    for path in sorted(set(paths)):
        if not any(char in path for char in "*?{[") and not Path(path).exists():
            yield f"MISSING {path}"

    required = ("Level", "Rule", "Why", "Evidence", "Exceptions", "Enforced by")
    seen = set()
    sections = re.split(r"^(#{1,3} .+)$", text, flags=re.MULTILINE)
    for heading, body in zip(sections[1::2], sections[2::2]):
        rule = re.fullmatch(r"### `(arch-[a-z0-9-]+)`\s*", heading)
        if not rule:
            continue
        name = rule[1]
        if name in seen:
            yield f"DUPLICATE {name}"
        seen.add(name)
        values = defaultdict(list)
        for field, value in re.findall(r"^- \*\*([^*\n]+)\*\*:([^\n]*(?:\n[ \t]+[^\n]*)*)", body, re.MULTILINE):
            values[field].append(value.strip())
        for field in required:
            entries = values[field]
            if len(entries) != 1:
                yield f"FIELD {name}: {field} occurs {len(entries)} times"
            elif not entries[0]:
                yield f"FIELD {name}: {field} is empty"
    if not seen:
        yield "No architecture rule headings found"


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    try:
        issues = list(findings(Path("docs/ARCHITECTURE.md").read_text(encoding="utf-8")))
    except (OSError, UnicodeError) as error:
        sys.exit(str(error))
    for issue in issues:
        print(issue)
    sys.exit(bool(issues))
