"""Check docs/ARCHITECTURE.md from the repository root."""

import re
import sys
from collections import Counter
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
        counts = Counter(re.findall(r"^- \*\*(.+?)\*\*:", body, re.MULTILINE))
        for field in required:
            if counts[field] != 1:
                yield f"FIELD {name}: {field} occurs {counts[field]} times"
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
