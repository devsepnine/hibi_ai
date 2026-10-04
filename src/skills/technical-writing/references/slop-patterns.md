# Slop patterns

Scan for these, then rewrite. Preserve the meaning and the intended tone. Adapted from pstack's `unslop`; its rule numbers are kept so the two lists can be compared.

## Content

- **3. Superficial -ing phrases.** "highlighting...", "ensuring...", "reflecting...", "fostering...". Delete them or expand with a real source.
- **5. Vague attributions.** "Experts believe", "Industry reports suggest". Name the source or delete the claim.

## Language

- **7. AI vocabulary.** Additionally, crucial, delve, enduring, enhance, fostering, garner, interplay, intricate, landscape, pivotal, robust, seamless, showcase, tapestry, testament, underscore, vibrant. Replace with plain words.
- **8. Fancy ways to say "is".** "serves as", "stands as", "boasts", "features". Write "is" or "has".
- **9. "Not just X, but Y."** State the point directly.
- **10. Rule of three.** Forcing ideas into groups of three. Use the natural number.
- **11. Synonym cycling.** Four names for one thing in one paragraph. Pick one and repeat it.
- **12. False ranges.** "from X to Y" where X and Y sit on no real scale. List the items.

## Style

- **13. Dashes and parentheses.** Banned in prose. End the sentence or use a comma. `tools/lint-prose.py` enforces it.
- **14. Colon overuse.** A colon is fine before a list or an example and after a label. It is not a mid-sentence connector.
- **15. Boldface overuse.** Do not bold every proper noun or acronym.
- **16. Inline-header lists.** A bold label whose line restates it, as in "**Performance**: performance improved". Convert to prose. A label followed by genuinely new detail is fine.
- **17. Title case headings.** Use sentence case.
- **18. Decorative emojis.** Remove them.
- **19. Curly quotes.** Use straight quotes.

## Communication artifacts

- **20. Chatbot phrases.** "I hope this helps!", "Let me know if...", "Certainly!". Remove them.
- **22. Sycophantic tone.** "Great question! You're absolutely right!" Respond directly.

## Filler

- **23. Filler phrases.** "In order to" becomes "to". "Due to the fact that" becomes "because". "It is important to note that" goes.
- **24. Excessive hedging.** "could potentially possibly be argued that it might" becomes "may".
- **25. Generic conclusions.** "The future looks bright." State the specific plan or fact.

## Jargon

- **26. Abstract metaphor nouns.** Substrate, wedge, vector, locus, nexus, primitive as a noun, surface as in "API surface", bedrock, scaffolding as a metaphor, paradigm, gold-plating, ratchet as a metaphor, endgame, north star, flywheel. "Substrate" becomes "base". "Vector" becomes "way". "Gold-plating" becomes "more than the job needs". "Ratchet" becomes "a limit that only tightens".

## Plain speech

- **27. Say what it does, not how it feels.** "SQL you can read" names a feeling. "`.toSQL()` returns the exact string sent to the database" names the mechanism. If the sentence could appear unchanged in another project's docs, it says nothing about this one.
- **28. Shorten or split dense sentences.** If the reader has to backtrack, break the sentence in two.
- **29. Active voice.** "Queries are validated" becomes "the compiler validates queries". Passive is fine only when the actor is unknown or does not matter.
- **30. Cut adverbs.** "significantly improves" becomes the measured delta.
- **31. Prefer the plain word.** "utilize" becomes "use", "facilitate" becomes "help", "in the event that" becomes "if".
- **32. Mannered prose.** Aphorisms, rhetorical fragments, personified code, figurative verbs. Say what you mean.
- **33. Over-compression.** Dropped articles, verbless fragments, and arrows that make the reader decode. Write whole sentences.
