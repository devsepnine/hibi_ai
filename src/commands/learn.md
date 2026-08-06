---
description: Extract reusable patterns from the current session into skills. Saves successful workflow patterns for future use.
allowed-tools: Read, Grep, Write
model: haiku
effort: low
---

# /learn - Extract Reusable Patterns

Analyze the current session and extract any patterns worth saving as skills.

## Trigger

Run `/learn` at any point during a session when you've solved a non-trivial problem.

## What to Extract

Look for:

1. **Error Resolution Patterns**
   - What error occurred?
   - What was the root cause?
   - What fixed it?
   - Is this reusable for similar errors?

2. **Debugging Techniques**
   - Non-obvious debugging steps
   - Tool combinations that worked
   - Diagnostic patterns

3. **Workarounds**
   - Library quirks
   - API limitations
   - Version-specific fixes

4. **Project-Specific Patterns**
   - Codebase conventions discovered
   - Architecture decisions made
   - Integration patterns

## Output Format

Write a real skill: `~/.claude/skills/<kebab-case-name>/SKILL.md`. A flat file
without frontmatter never loads, so a pattern saved that way is lost work — the
directory plus frontmatter is what makes it discoverable.

`description` is what decides whether the skill ever triggers, so write it as one
or two sentences and end with the phrases a user would actually type, Korean
included; a short imperative description underperforms on Korean queries. Replace
every `<...>` below — a placeholder left in the file loads fine and silently
never matches.

```markdown
---
name: <kebab-case-name>
description: <what it does and when to use it, ending with real trigger phrases>
keywords: [<english-terms>, <한국어용어>]
---

# <Descriptive Pattern Name>

## Problem
<the failure this prevents, specifically — what went wrong and how it looked>

## Solution
<the pattern, stated so it can be applied without re-deriving it>

## Example
<code, if it clarifies>

## When to Use
<trigger conditions, and when NOT to use it>
```

## Process

1. Review the session for extractable patterns
2. Identify the most valuable/reusable insight
3. Draft the skill file
4. Ask user to confirm before saving
5. Save to `~/.claude/skills/<name>/SKILL.md`
6. If the pattern would help users beyond this machine, run `/upstream-pr` to propose it for the distributed config — it ships as `src/skills/<name>/SKILL.md` with a `-ko.md` twin

## Notes

- Don't extract trivial fixes (typos, simple syntax errors)
- Don't extract one-time issues (specific API outages, etc.)
- Focus on patterns that will save time in future sessions
- Keep skills focused - one pattern per skill
