"""Prepare the selected project files for the Vercel fallback archive."""

import os
import shutil
import struct
import subprocess
import sys
import tarfile
from pathlib import Path, PurePosixPath


APPLEDOUBLE_V2 = bytes.fromhex("00051607 00020000")


def is_appledouble(content, size):
    header = content.read(26)
    if len(header) != 26 or header[:8] != APPLEDOUBLE_V2:
        return False
    count = int.from_bytes(header[24:26], "big")
    table_end = 26 + count * 12
    if not count or table_end > size:
        return False
    entries = struct.iter_unpack(">III", content.read(count * 12))
    return all(table_end <= offset <= offset + length <= size for _, offset, length in entries)


def extract_archive(source, destination):
    with tarfile.open(source, "r:gz") as archive:
        def filter_member(member, target):
            if member.isfile() and PurePosixPath(member.name).name.startswith("._"):
                with archive.extractfile(member) as content:
                    if is_appledouble(content, member.size):
                        return None
            return tarfile.data_filter(member, target)

        archive.extractall(destination, filter=filter_member)


def ignored_paths(candidates, matcher, rules):
    result = subprocess.run(
        ["git", "-C", matcher, "-c", f"core.excludesFile={rules}",
         "check-ignore", "--no-index", "--stdin", "-z"],
        input=b"\0".join(candidates) + b"\0", stdout=subprocess.PIPE,
    )
    if result.returncode not in (0, 1):
        raise SystemExit(result.returncode)
    return set(result.stdout.split(b"\0"))


def list_files(project, root, matcher):
    scratch = matcher.parent.resolve()
    source_rules = root / ".vercelignore"
    has_rules = source_rules.exists() or source_rules.is_symlink()
    rules = matcher / ".git" / "deployment-ignore"
    if has_rules:
        # An empty worktree isolates deployment rules from local .gitignore files
        # and lets Git match submodule paths like any other directory.
        subprocess.run(["git", "init", "-q", "--template=", matcher], check=True)
        shutil.copyfile(source_rules, rules)
    pending = [project]
    while pending:
        entries = {}
        for directory in pending:
            with os.scandir(directory) as children:
                for child in children:
                    if child.name in {".git", "node_modules", ".vercel", ".env", ".npmrc"} or child.name.startswith(".env."):
                        continue
                    path = Path(child.path)
                    if path == scratch:
                        continue
                    is_dir = child.is_dir(follow_symlinks=False)
                    relative = path.relative_to(root)
                    if has_rules and is_dir:
                        # Git needs the directory type to distinguish "logs/" from
                        # "logs/**", whose children can be included individually.
                        (matcher / relative).mkdir(parents=True, exist_ok=True)
                    key = b"./" + os.fsencode(relative)
                    entries[key] = (path, is_dir)
        ignored = ignored_paths(entries, matcher, rules) if has_rules and entries else set()
        pending = []
        for key, (path, is_dir) in entries.items():
            if key in ignored:
                continue
            yield path, is_dir
            if is_dir:
                pending.append(path)


def stage_project(project, root, matcher, staging):
    staging.mkdir()
    for path, is_dir in list_files(project, root, matcher):
        destination = staging / path.relative_to(project)
        if is_dir:
            destination.mkdir()
        elif path.is_symlink() or path.is_file():
            if path.name.startswith("._") and not path.is_symlink():
                with path.open("rb") as content:
                    if is_appledouble(content, os.fstat(content.fileno()).st_size):
                        continue
            shutil.copy2(path, destination, follow_symlinks=False)


if __name__ == "__main__":
    if sys.argv[1] == "--extract":
        extract_archive(*sys.argv[2:])
    else:
        stage_project(*map(Path, sys.argv[1:]))
