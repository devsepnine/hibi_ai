---
name: technical-writing
description: "Technical writing standard: one Diátaxis mode per doc, plain direct sentences, an AI-slop cut list, and no dashes or parentheses in prose. Use when writing or reviewing docs, READMEs, guides, or skill and agent text. 기술 문서 작성, 문서 리뷰, AI 문체 제거, 글 다듬기."
---

# Technical Writing

The goal is writing that a tired engineer understands on the first read. This skill owns how prose reads. Other skills own what a document must contain: `pull-request` for PR bodies, `commit-rules` for commit messages, `qa-handoff` and `obsidian-notes` for their templates. Apply this skill on top of them.

Three rules sit above the rest:

- **Cut every word that does no work.** "In order to" is "to". "It is important to note that" is nothing.
- **Use the short, everyday word.** "Use", not "utilize". "Help", not "facilitate".
- **When a rule makes a sentence worse, fix the sentence another way.** The rules serve the reader. A sentence that obeys every rule and reads like a machine wrote it has failed.

Write the real symbol, file, flag, or command name, not a synonym or a description of it.

## Punctuation: no dashes, no parentheses

Prose uses no em dash, no en dash, no hyphen standing in for a dash, and no parentheses. End the sentence, or join with a comma. A colon is fine before a list or an example, and after a label as in "**Label**: text". It is not a drop-in for a dash.

| Instead of | Write |
|---|---|
| `X — Y` | `X. Y` or `X, Y` |
| `X (aside) Y` | fold the aside into the sentence, give it its own sentence, or cut it |
| `MC/DC (modified condition/decision coverage)` | `MC/DC, short for modified condition/decision coverage` |
| `50–100ms` | `50 to 100ms` |

Code, inline code, link targets, wikilinks, and frontmatter keys are syntax and stay as they are. Do not wrap ordinary prose in backticks to get around the rule. A `description:` value that contains `: ` must be quoted, or YAML rejects it.

The rule is enforced, not remembered: run `python tools/lint-prose.py [paths]` before reporting done. It exits 1 and prints `file:line` for each violation.

## Pick the mode first: Diátaxis

One document, one mode. Two questions pick it. Does the content inform action or understanding? Does it serve learning or work?

- **Tutorial**, action plus learning. Every step produces a visible result. Tell the reader what they should see.
- **How-to**, action plus work. Solve a problem the reader has. Assume competence, skip teaching, allow forks: "If you want x, do y."
- **Reference**, understanding plus work. Describe and only describe. Mirror the structure of the thing described.
- **Explanation**, understanding plus learning. One bounded topic, anchored on a real why question. Opinion belongs here and nowhere else.

Do not mix modes. Split and link instead.

## Write sentences to the reader

- Talk to the reader as "you", in the present tense.
- Say who does what: "the compiler checks", not "is checked".
- Put the condition before the instruction: "To delete the document, click Delete."
- Put the common case first and exceptions after.
- One instruction per sentence. Split an instruction past about 20 words.
- Keep "only" and "not" next to the word they change.
- Make every "it", "they", and "this" point at one obvious thing. Repeat the noun when in doubt.
- Call each thing by one name everywhere. Do not reword a sentence that did not change.
- Headings carry the point, not the topic: "Pick the mode first", not "Modes".
- Never write "simply", "easy", or "quickly" in a procedure.

## Cut the AI tells

The full catalog with examples is `references/slop-patterns.md`. The ones that show up most:

- **AI vocabulary**: delve, crucial, pivotal, leverage, showcase, underscore, landscape, tapestry, robust, seamless. Use the plain word.
- **Fancy "is"**: "serves as", "stands as", "boasts". Write "is" or "has".
- **Abstract metaphor nouns**: substrate, vector, surface, north star, flywheel. Name the concrete thing.
- **Feeling instead of mechanism**: "types that follow your schema" says nothing. "A column rename fails the build" does. If the sentence could sit unchanged in another project's docs, cut it.
- **Over-compression**: "bad date → exit 2, no write" makes the reader decode. Write "The parser rejects a bad date, exits with code 2, and writes nothing."
- **Chatbot and filler phrases**: "I hope this helps", "Great question", "It is worth noting that". Delete them.

## Korean prose

The same rules apply to Korean text, with these additions:

- No dash or parenthesis asides here either. Write `X, 즉 Y` or `X. Y`, and `MC/DC, 곧 modified condition/decision coverage`.
- Keep one English term per concept and do not gloss it twice. Pick `결합도` or `coupling` for a document and stay with it.
- Cut translationese: "~하는 것이 중요하다", "~에 있어서", "~를 통해" when a plain verb works.

## Before handing back

- [ ] The document is one Diátaxis mode
- [ ] Every sentence names a mechanism, a fact, or an instruction
- [ ] No pattern from `references/slop-patterns.md`
- [ ] `python tools/lint-prose.py <changed paths>` prints nothing
- [ ] EN and `-ko.md` mirrors still say the same thing
