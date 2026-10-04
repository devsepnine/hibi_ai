#!/usr/bin/env python3
"""Check the mechanical MUST rules in docs/ARCHITECTURE.md.

Each check yields (rule_id, path) pairs. A pair listed in the document's
Known violations table is accepted; any other pair fails, and a listed pair
that no longer occurs is reported as stale so the table shrinks with the fixes.

Usage: python tools/lint-arch.py   (from anywhere; paths resolve to the repo)
Exit status: 0 clean, 1 new violations or stale entries, 2 unreadable document.
"""
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOC = Path("docs/ARCHITECTURE.md")
INSTALLER = Path("tools/installer/src")
STATUSLINE = Path("tools/statusline")
LEAF_MODULES = ["component.rs", "mcp.rs", "plugin.rs", "theme.rs", "source"]
INNER_MODULES = "app|ui|cli|fs|loading|tree|process_exec"
CONFIG_WRITERS = ("fs/installer/", "fs/manifest.rs", "source/")

# Only an inline `mod tests {` opens test code. `#[cfg(test)] mod tests;` merely
# declares a sibling tests.rs, so the code after it is still production code.
TEST_MODULE_START = re.compile(r"^\s*mod tests\s*\{")
LINE_COMMENT = re.compile(r"(^|\s)//.*$")
PATH_DEPENDENCY = re.compile(r"^[A-Za-z0-9_-]+\s*=\s*\{[^}]*\bpath\s*=")


def posix(path):
    return path.as_posix()


def production_lines(path):
    """Lines before the test module, with line comments removed."""
    if path.name in ("tests.rs", "test_support.rs"):
        return []
    lines = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if TEST_MODULE_START.match(line):
            break
        lines.append(LINE_COMMENT.sub("", line))
    return lines


def rust_files(root):
    return sorted(p for p in root.rglob("*.rs") if "target" not in p.parts)


def files_matching(root, pattern):
    regex = re.compile(pattern)
    for path in rust_files(root):
        if any(regex.search(line) for line in production_lines(path)):
            yield path


def fs_no_app_import():
    yield from files_matching(INSTALLER / "fs", r"crate::app\b")


def fs_no_ui_import():
    yield from files_matching(INSTALLER / "fs", r"crate::ui\b")


def ui_reads_state_only():
    """Any filesystem import or call fails; the one allowed read is `crate::fs::VERSION`."""
    fs_use = re.compile(r"\bstd::fs\b|\bcrate::fs\b|\bcrate::\{[^}]*\bfs\b|^\s*use\s+std::\{[^}]*\bfs\b")
    for path in rust_files(INSTALLER / "ui"):
        for line in production_lines(path):
            if fs_use.search(line.replace("crate::fs::VERSION", "")):
                yield path
                break


def leaf_modules_no_crate_import():
    pattern = rf"crate::({INNER_MODULES})\b"
    for name in LEAF_MODULES:
        target = INSTALLER / name
        if target.is_dir():
            yield from files_matching(target, pattern)
        elif any(re.search(pattern, line) for line in production_lines(target)):
            yield target


def process_spawn_in_fs():
    for path in files_matching(INSTALLER, r"Command::new|\.spawn\(\)|\.status\(\)|\.output\(\)"):
        if not posix(path).startswith(posix(INSTALLER / "fs") + "/"):
            yield path


def config_writes_in_fs_or_source():
    for path in files_matching(INSTALLER, r"\bfs::write\("):
        inner = posix(path.relative_to(INSTALLER))
        if not inner.startswith(CONFIG_WRITERS):
            yield path


def statusline_standalone():
    yield from files_matching(STATUSLINE / "src", r"\bhibi_ai\b")
    for manifest in (STATUSLINE / "Cargo.toml", Path("tools/installer/Cargo.toml")):
        lines = manifest.read_text(encoding="utf-8").splitlines()
        if any(PATH_DEPENDENCY.match(line.strip()) for line in lines):
            yield manifest


def ko_mirror():
    for path in sorted(Path("src").rglob("*.md")):
        if not path.stem.endswith("-ko") and not path.with_name(path.stem + "-ko.md").exists():
            yield path


def skill_name_kebab():
    name_field = re.compile(r"^name:\s*(\S+)", re.M)
    for path in sorted(p for p in Path("src/skills").iterdir() if p.is_dir()):
        skill = path / "SKILL.md"
        declared = name_field.search(skill.read_text(encoding="utf-8")) if skill.exists() else None
        kebab = re.fullmatch(r"[a-z0-9]+(-[a-z0-9]+)*", path.name)
        if not kebab or not declared or declared.group(1) != path.name:
            yield path


CHECKS = {
    "arch-fs-no-app-import": fs_no_app_import,
    "arch-fs-no-ui-import": fs_no_ui_import,
    "arch-ui-reads-state-only": ui_reads_state_only,
    "arch-leaf-modules-no-crate-import": leaf_modules_no_crate_import,
    "arch-process-spawn-in-fs": process_spawn_in_fs,
    "arch-config-writes-in-fs-or-source": config_writes_in_fs_or_source,
    "arch-statusline-standalone": statusline_standalone,
    "arch-ko-mirror": ko_mirror,
    "arch-skill-name-kebab": skill_name_kebab,
}


def known_violations():
    text = DOC.read_text(encoding="utf-8")
    if "\n## Known violations" not in text:
        print(f"{DOC}: no '## Known violations' section", file=sys.stderr)
        raise SystemExit(2)
    section = text.split("\n## Known violations", 1)[1].split("\n## ", 1)[0]
    row = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*`(arch-[a-z0-9-]+)`\s*\|")
    return {(m.group(2), m.group(1)) for m in map(row.match, section.splitlines()) if m}


def main():
    os.chdir(ROOT)
    found = {(rule, posix(path)) for rule, check in CHECKS.items() for path in check()}
    known = {pair for pair in known_violations() if pair[0] in CHECKS}
    problems = [f"{path}: {rule}" for rule, path in sorted(found - known)]
    problems += [
        f"{path}: {rule}: stale, no longer violates; remove it from Known violations"
        for rule, path in sorted(known - found)
    ]
    for problem in problems:
        print(problem)
    if problems:
        print(f"\n{len(problems)} problem(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
    sys.exit(main())
