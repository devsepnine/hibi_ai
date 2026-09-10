#!/usr/bin/env python3
"""Measure whether a skill's description actually auto-triggers.

Spawns a nested `claude -p` per query and inspects the emitted stream for a
`Skill` tool_use whose `input.skill` names the target skill -- that is what the
nested session actually emits, and it is the skill's *directory* name, not the
frontmatter `name:`.

Two properties every number here depends on:

- Bidirectional self-test. A known positive must fire and a known negative must
  not, in the same batch, or the run is void. A detector that cannot report
  "dirty" proves nothing when it reports "clean".
- Completion tracking. Only a run that reached its own answer -- a `result`
  event with subtype `success` -- can testify that the skill was *not* chosen. A
  timeout emits no `result` at all, and `error_max_turns` means the turn budget
  ran out first: the nested session spends early turns on the pre-work checks
  its CLAUDE.md mandates, so a skill it would have picked at turn 3 never gets
  the chance. An unfired row that ended either way is INCONCLUSIVE -- never
  PASS, never FAIL. A row that fired still scores by expectation even if the run
  died afterwards, because firing is evidence no later failure retracts. Without
  this, every should-NOT query passes vacuously and the harness scores its own
  `--max-turns` instead of the description.

Exit status: 0 all clear, 1 at least one FAIL, 2 self-test gate failed (no
scores produced), 3 no FAIL but at least one INCONCLUSIVE, 4 the harness could
not run at all. 4 is separate on purpose: a missing CLI or an unusable eval set
is not a measurement, and must not land on a code that reads as FAIL.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tempfile
import traceback
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

SELFTEST_POSITIVE = "{skill} 스킬을 사용해서 진행해줘"
SELFTEST_NEGATIVE = "다음 문장을 영어로 번역해줘: 하늘이 맑다"


@dataclass
class Outcome:
    query: str
    expect: bool
    fired: bool = False
    completed: bool = False
    subtype: str = ""
    tools: list[str] = field(default_factory=list)
    skills: list[str] = field(default_factory=list)
    stderr: str = ""

    @property
    def verdict(self) -> str:
        # Firing is positive evidence however the run ended; not firing only
        # counts against the description if the run got far enough to answer.
        if self.fired:
            return "PASS" if self.expect else "FAIL"
        if self.subtype != "success":
            return "INCONCLUSIVE"
        return "FAIL" if self.expect else "PASS"


def cannot_run(reason: str) -> SystemExit:
    """Exit 4 keeps "never ran" out of the codes that mean PASS or FAIL."""
    print(reason, file=sys.stderr)
    return SystemExit(4)


def as_text(buf: bytes | str | None) -> str:
    """Decode leniently: a nested run's bytes are not guaranteed to be UTF-8.

    Both paths need this. A timeout kill lands mid-codepoint wherever it lands,
    and even a clean run can emit an invalid byte. Strict decoding would turn a
    truncated or malformed Korean stream into a crash instead of the row it
    actually is, losing every complete event before the bad byte.
    """
    if isinstance(buf, bytes):
        return buf.decode(errors="replace")
    return buf or ""


def parse_stream(raw: str, target: str) -> tuple[bool, bool, str, list[str], list[str]]:
    fired, completed, subtype = False, False, ""
    tools: list[str] = []
    skills: list[str] = []
    for line in raw.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") == "result":
            completed = True
            subtype = event.get("subtype", "")
        # `message` is a plain string on some system events, and `content` is a
        # string on text-only assistant turns. Assuming either is structured
        # crashes the whole batch on one unexpected event.
        message = event.get("message")
        content = message.get("content") if isinstance(message, dict) else None
        if not isinstance(content, list):
            continue
        for block in content:
            if not isinstance(block, dict) or block.get("type") != "tool_use":
                continue
            tools.append(block.get("name", ""))
            if block.get("name") == "Skill":
                tool_input = block.get("input")
                name = tool_input.get("skill", "") if isinstance(tool_input, dict) else ""
                skills.append(name)
                if name == target:
                    fired = True
    return fired, completed, subtype, tools, skills


def run_query(query: str, expect: bool, target: str, cwd: Path, timeout: int,
              max_turns: int) -> Outcome:
    cmd = [
        "claude", "-p", query,
        "--output-format", "stream-json",
        "--verbose",
        "--max-turns", str(max_turns),
    ]
    # The prompt travels in argv, so an inherited stdin only makes the nested CLI
    # stall 3s waiting for input it will never get and emit a warning on every
    # row, which would crowd out the real stderr this reports.
    try:
        proc = subprocess.run(
            cmd, cwd=cwd, stdin=subprocess.DEVNULL,
            capture_output=True, timeout=timeout,
        )
        raw, err = as_text(proc.stdout), as_text(proc.stderr)
    except subprocess.TimeoutExpired as exc:
        raw = as_text(exc.stdout)
        err = f"{as_text(exc.stderr)}\ntimeout after {timeout}s".strip()
    except FileNotFoundError:
        # Also raised when `cwd` has been deleted under us, so name both causes
        # rather than sending the operator after a PATH problem they don't have.
        raise cannot_run(f"could not spawn `claude` in {cwd} -- missing CLI, or the "
                         "scratch cwd no longer exists")

    fired, completed, subtype, tools, skills = parse_stream(raw, target)
    return Outcome(query, expect, fired, completed, subtype, tools, skills, err.strip())


def run_batch(rows, target, cwd, timeout, jobs, max_turns) -> list[Outcome]:
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = [
            pool.submit(run_query, q, e, target, cwd, timeout, max_turns)
            for q, e in rows
        ]
        return [f.result() for f in futures]


def load_eval_set(path: Path) -> list[tuple[str, bool]]:
    """Reject a malformed set outright rather than scoring a misread one.

    `should_trigger` is checked for an actual bool because the JSON is
    hand-written: `"false"` is a truthy string, and coercing it would silently
    flip a should-NOT row into a should-trigger one.
    """
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise cannot_run(f"{path}: {exc}")
    if not isinstance(raw, list) or not raw:
        raise cannot_run(f"{path} must be a non-empty JSON list of queries")
    rows: list[tuple[str, bool]] = []
    for i, row in enumerate(raw):
        if not isinstance(row, dict) or not isinstance(row.get("query"), str):
            raise cannot_run(f'{path}[{i}] needs a "query" string')
        flag = row.get("should_trigger")
        if not isinstance(flag, bool):
            raise cannot_run(f'{path}[{i}] "should_trigger" must be true or false, got {flag!r}')
        rows.append((row["query"], flag))
    return rows


def render(outcomes: list[Outcome], label: str) -> None:
    print(f"\n{label}")
    print("-" * len(label))
    for out in outcomes:
        want = "should-trigger" if out.expect else "should-NOT"
        # Keyed off the same condition the verdict uses, so an INCONCLUSIVE row
        # never prints without the reason it is one.
        note = ""
        if out.subtype != "success":
            if not out.completed:
                note = "  [no result event]"
            else:
                note = f"  [{out.subtype or 'result without subtype'}]"
        query = out.query if len(out.query) <= 58 else out.query[:57] + "…"
        print(f"  {out.verdict:<13} {want:<15} {query}{note}")
        if out.stderr:
            print(f"                  stderr: {out.stderr.splitlines()[0][:100]}")


def main() -> int:
    # Every query printed back is Korean, so a locale-encoded stream (Windows
    # cp1252, or any redirected pipe under a C locale) would crash on the first
    # rendered row rather than report a result.
    for stream in (sys.stdout, sys.stderr):
        stream.reconfigure(encoding="utf-8", errors="replace")

    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--skill", required=True, help="skill directory name, as emitted in Skill{skill:...}")
    ap.add_argument("--eval-set", required=True, type=Path, help='JSON list of {"query","should_trigger"}')
    ap.add_argument("--timeout", type=int, default=300, help="per-query seconds (default 300)")
    ap.add_argument("--max-turns", type=int, default=6,
                    help="nested turn budget (default 6; below ~4 the pre-work "
                         "checks in CLAUDE.md consume the budget first)")
    ap.add_argument("--jobs", type=int, default=4, help="parallel queries (default 4)")
    ap.add_argument("--cwd", type=Path, help="scratch working directory (default: a temp dir)")
    ap.add_argument("--negative-probe", default=SELFTEST_NEGATIVE, help="self-test query that must NOT fire")
    ap.add_argument("--json", type=Path, help="also write machine-readable results here")
    args = ap.parse_args()

    if not shutil.which("claude"):
        raise cannot_run("`claude` not found on PATH -- the harness needs the CLI")
    if args.jobs < 1:
        raise cannot_run(f"--jobs must be at least 1, got {args.jobs}")

    rows = load_eval_set(args.eval_set)

    scratch = args.cwd or Path(tempfile.mkdtemp(prefix="trigger-eval-"))
    scratch.mkdir(parents=True, exist_ok=True)

    print(f"skill={args.skill}  queries={len(rows)}  jobs={args.jobs}  "
          f"timeout={args.timeout}s  max-turns={args.max_turns}")
    print(f"scratch cwd: {scratch}")

    gate_rows = [
        (SELFTEST_POSITIVE.format(skill=args.skill), True),
        (args.negative_probe, False),
    ]
    gate = run_batch(gate_rows, args.skill, scratch, args.timeout,
                     min(2, args.jobs), args.max_turns)
    render(gate, "Self-test gate")
    if any(g.verdict != "PASS" for g in gate):
        print("\nGATE FAILED -- the detector is not known-good, so no score is reported.")
        print("A positive that will not fire, or a negative that does, invalidates every row.")
        print("An INCONCLUSIVE probe means it never got far enough to answer: raise")
        print("--max-turns or --timeout, which is the same cure as for an eval row.")
        return 2

    outcomes = run_batch(rows, args.skill, scratch, args.timeout, args.jobs,
                         args.max_turns)
    render(outcomes, f"Eval set ({args.eval_set})")

    passed = sum(1 for o in outcomes if o.verdict == "PASS")
    failed = [o for o in outcomes if o.verdict == "FAIL"]
    unknown = [o for o in outcomes if o.verdict == "INCONCLUSIVE"]
    print(f"\n{passed}/{len(outcomes)} PASS   {len(failed)} FAIL   {len(unknown)} INCONCLUSIVE")
    if unknown:
        print("INCONCLUSIVE rows never reached an answer -- re-run them serially with a")
        print("longer --timeout, or a larger --max-turns if the subtype is error_max_turns,")
        print("before quoting any figure. They are not passes and not failures.")

    if args.json:
        # A clean run that dies here would exit 1 and read as "some FAIL". The
        # scores above are already valid, but a consumer expecting the file got
        # nothing, which is closer to "did not run" than to a verdict.
        try:
            args.json.write_text(json.dumps({
                "skill": args.skill,
                "eval_set": str(args.eval_set),
                "self_test": [vars(g) | {"verdict": g.verdict} for g in gate],
                "results": [vars(o) | {"verdict": o.verdict} for o in outcomes],
                "summary": {"pass": passed, "fail": len(failed), "inconclusive": len(unknown)},
            }, ensure_ascii=False, indent=2), encoding="utf-8")
        except OSError as exc:
            raise cannot_run(f"results computed but --json could not be written: {exc}")
        print(f"wrote {args.json}")

    if failed:
        return 1
    return 3 if unknown else 0


if __name__ == "__main__":
    # Any uncaught exception would exit 1, which is the documented code for "at
    # least one FAIL" -- a crash would be read as a measurement. `Exception`
    # spares SystemExit and KeyboardInterrupt, so 0-4 and 130 keep their meaning.
    try:
        sys.exit(main())
    except Exception:
        traceback.print_exc()
        print("harness error -- not a measurement", file=sys.stderr)
        sys.exit(4)
