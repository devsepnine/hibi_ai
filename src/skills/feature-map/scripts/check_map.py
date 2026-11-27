"""Check feature map citations and labels against the supplied source roots."""

import argparse
import mmap
import os
import re
import sys
from pathlib import Path

CITED_PATH = r"`([^`/ ][^` ]*/[^` ]*\.[A-Za-z0-9]{1,10})`"


def check_document(document, labels):
    issues = []
    for number, line in enumerate(document.read_text(encoding="utf-8").splitlines(), 1):
        location = f"{document}:{number}"
        for path in re.findall(CITED_PATH, line):
            if not path.startswith("~/") and "://" not in path and not Path(path).exists():
                issues.append(f"MISSING {path} at {location}")
        for path, symbol in re.findall(r"`([^`]+)` → `([A-Za-z_$][A-Za-z0-9_$]*)`", line):
            cited = Path(path)
            if "/" in path and cited.suffix and not path.startswith(("/", "~/")) and "://" not in path:
                if not cited.is_file() or symbol.encode() not in cited.read_bytes():
                    issues.append(f"MISSING-SYMBOL {path} → {symbol} at {location}")
        if re.match(r"^- \*\*(UI strings|Controls|UI 문구|컨트롤)\*\*:", line):
            labels.update(re.findall(r'"([^"]+)"', line))
        if re.match(r"^- \*\*(Pitfalls|함정)\*\*:", line):
            if not line.endswith(": none") and not re.search(r": [^→]+ → .+, (source|근거): ", line):
                issues.append(f"NO-CAUSE {location}")
    return issues


def raise_walk_error(error):
    raise error


def source_files(roots):
    for root in roots:
        if root.is_file():
            yield root
            continue
        for directory, directories, files in os.walk(root, onerror=raise_walk_error):
            if ".git" in directories:
                directories.remove(".git")
            for name in files:
                path = Path(directory) / name
                if path.is_file() and not path.is_symlink():
                    yield path


def missing_labels(roots, labels, documents):
    remaining = {label.encode(): label for label in labels}
    excluded = {path.resolve() for path in documents}
    for path in source_files(roots):
        if not remaining:
            break
        if path.resolve() in excluded or not path.stat().st_size:
            continue
        with path.open("rb") as source, mmap.mmap(source.fileno(), 0, access=mmap.ACCESS_READ) as data:
            remaining = {key: label for key, label in remaining.items() if data.find(key) < 0}
    return sorted(remaining.values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source_roots", nargs="+", type=Path)
    roots = parser.parse_args().source_roots
    for root in roots:
        if not root.is_dir() and not root.is_file():
            parser.error(f"Missing source path: {root}")
    labels = set()
    docs = Path("docs")
    documents = [docs / "FEATURES.md", *sorted((docs / "features").glob("*.md"))]
    issues = []
    for document in documents:
        issues.extend(check_document(document, labels))
    issues.extend(f"MISSING-STRING {label}" for label in missing_labels(roots, labels, documents))
    for issue in issues:
        print(issue)
    return bool(issues)


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    try:
        sys.exit(main())
    except (OSError, UnicodeError) as error:
        sys.exit(str(error))
