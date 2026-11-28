"""List a skill catalog and identify identical Markdown instruction bodies."""

import argparse
import hashlib
import json
import os
import stat
import sys
from collections import defaultdict
from pathlib import Path


def fail_walk(error):
    raise error


def instruction_files(folder):
    yield folder / "SKILL.md"
    for name in ("references", "rules"):
        directory = folder / name
        try:
            directory.stat()
        except FileNotFoundError:
            continue
        for parent, subdirs, files in os.walk(directory, onerror=fail_walk):
            subdirs.sort()
            for filename in sorted(files):
                path = Path(parent) / filename
                if path.suffix != ".md" or not path.is_file():
                    continue
                if path.stem.endswith("-ko") and path.with_name(path.stem[:-3] + ".md").is_file():
                    continue
                yield path


def instruction_body(path):
    try:
        text = path.read_text(encoding="utf-8-sig")
    except UnicodeDecodeError as error:
        raise ValueError(f"{path} is not valid UTF-8: {error}") from error
    lines = text.splitlines(keepends=True)
    if lines and lines[0].rstrip() == "---":
        for index, line in enumerate(lines[1:], 1):
            if line.rstrip() == "---":
                return "".join(lines[index + 1:]).strip()
    return text.strip()


def inventory(root):
    skills = []
    missing = []
    matches = defaultdict(list)
    for folder in sorted(root.iterdir()):
        if folder.name.startswith(".") or not folder.is_dir():
            continue
        try:
            has_entry = stat.S_ISREG((folder / "SKILL.md").stat().st_mode)
        except FileNotFoundError:
            has_entry = False
        if not has_entry:
            missing.append(folder.name)
            continue
        files = []
        for path in instruction_files(folder):
            relative = path.relative_to(root).as_posix()
            files.append(relative)
            body = instruction_body(path)
            if body:
                digest = hashlib.sha256(body.encode("utf-8")).hexdigest()
                matches[digest].append(relative)
        skills.append({"folder": folder.name, "instruction_files": files})

    return {
        "skills": skills,
        "directories_without_entry": missing,
        "matching_bodies": sorted(paths for paths in matches.values() if len(paths) > 1),
    }


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog", type=Path, help="directory containing the skill folders")
    args = parser.parse_args()
    try:
        report = inventory(args.catalog)
    except (OSError, ValueError) as error:
        parser.exit(2, f"Could not inspect {args.catalog}: {error}\n")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    if not report["skills"]:
        parser.exit(1, "No immediate skill folder with SKILL.md was found. Check the catalog path.\n")


if __name__ == "__main__":
    sys.exit(main())
