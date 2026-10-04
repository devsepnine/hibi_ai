---
name: blast-radius
description: "Pre-merge blast radius: what a diff could break beyond itself, proving the one fact it is safe because of by running real code. Use when asked what a change could break or whether a small diff is safe to merge. 블라스트 레디어스, 뭐가 깨질까, 머지해도 안전해, 사이드 이펙트 검증."
---

# Blast Radius

Find what a change breaks somewhere else, before it ships. Listing callers is not the job, since grep does that in a second. The job is the breakage grep won't show you.

This is not `dependency-design`: that skill judges coupling while you design; this one interrogates one concrete diff before it merges. It is not `code-reviewer` either: the reviewer judges quality line by line; this skill hunts effects outside the diff.

## Don't trust your own writeup

A blast-radius writeup that sounds right is worthless, because it reads as convincing whether or not it is true. Find the one or two facts the change's safety depends on, and prove them by running code.

### Evidence ladder

Take each load-bearing fact as far down this ladder as is cheap, and say where it stopped:

1. **Said so**: worthless on its own.
2. **Pointed at the line**: a real `file:line`, or the library's own source.
3. **Walked the failure**: traced the bad case step by step and showed it cannot be reached.
4. **Ran it**: a script or test that calls the real code and fails loudly if you are wrong.
5. **Reproduced it in the running app.**

Step 4 is usually one small script that imports the same library version the app ships and calls the exact function in question. The required floor scales with the `do-178c` tier: A/B need step 4 for the safety fact; C needs step 3; D/E may stop at 2.

## Steps

1. **Read the change.** The diff, the symbols it adds, changes, and deletes, and what it now does differently, including what the diff does not spell out. Pull context with `git log -p --follow <file>` and `git blame -L <range> <file>`; `gh pr view <n>` when a PR number turns up.
2. **Find the one fact it is safe because of.** Most risky-looking changes are safe because of a single fact, such as "this call only drops cache entries that are already dead". If it holds, most risks clear at once. Spend your time here, not on a long list of maybes.
3. **Look where grep stops.**
   - Library source at the pinned version from the lockfile, and any local patch.
   - When things run: microtasks, effects, unmount and teardown, retries, process restart.
   - What a symbol search misses: JSON an API returns, a DB column, a wire or file format, another language reading the same bytes, a feature flag, config, code three hops downstream.
4. **Rate each risk honestly**, with a real chance of happening and a real cost if it does. Keep confirmed risks; list checked-and-cleared ones separately. Cite real `file:line`; a search that finds nothing is still an answer; never invent a caller or an API.
5. **Prove the safety fact.** Write the script or test that runs the real code, run it, and paste the output.
6. **For a wide change**, fan out: one `Agent` per surface, namely persistence, wire/API, concurrency/timing, and UI, in a single message, each with this skill's ladder, then merge. Keep a risk two lenses found independently, and re-verify any single-lens finding before reporting it.

## What to hand back

```
What it does:   <what changed, including the non-obvious part>
Safe because:   <the one fact>: ladder step N, <proof: command + output, or UNPROVEN>
Risks:
  [HIGH|MED|LOW] <how it breaks>: file:line, likelihood / cost, how to check
Cleared:
  <what was checked>: <why it is fine>
Before you merge:
  <the cheapest test or repro that catches the real bug, including the script you wrote>
```

If the safety fact stops above step 4 for an A/B change, the verdict is `UNPROVEN`, not safe. Strip anything private before the writeup goes anywhere public. Do not commit the proof script unless asked. Leave it in the working tree and name its path.

## Related

- `dependency-design`: coupling and dependency direction at design time.
- `verification-loop`: Phase 7 checks the change works; this skill checks what else it breaks.
- `do-178c`: owns the tiers; this skill maps each tier to an evidence-ladder floor.
