# Slop patterns

Scan for these, then rewrite. Preserve the meaning and the intended tone. The numbered catalog adapts pstack's `unslop`; its rule numbers are kept so the two lists can be compared.

## Reader clarity

Look for the work the sentence leaves to the reader. This follows the reader's perspective in Joseph M. Williams and Joseph Bizup's *Style: The Basics of Clarity and Grace*.

ASD-STE100 Issue 9, sections 5 and 6, gives English technical authors limits of 20 words for a procedural sentence and 25 for a descriptive sentence. In Korean, split where a thought ends and keep a cause, condition, or simultaneous action with what it explains. If the facts are clear, keep the sentence. [ASD-STE100 Issue 9](https://www.asd-ste100.org/assets/files/ASD-STE100_ISSUE9.pdf)

In Korean, replace phrases such as "~에 있어서" or "~하는 것이 중요하다" with the verb that says what happens. Keep a necessary technical term and explain it once. Keep related sentences in the same paragraph.

### Hidden actors and actions

Find who acts and what happens, then compare them with the grammatical subject and verb. "File comparison was performed by the installer" makes the reader recover the action from "comparison". "The installer compared the files" states it directly.

Nominalizations and passive voice can still be useful. "The configuration file was deleted. We have not identified who deleted it" accurately reports an unknown actor. Do not replace it with an invented person or component.

### Vague names and relationships

Check what each reference names and how adjacent nouns relate. Replace unusual labels with the names the reader or code uses. Keep a precise domain term when it carries meaning that an everyday substitute would lose.

"The database readiness check sits at the user session entry boundary before authentication handler execution" obscures a supported sequence. "Database readiness is checked before the authentication handler runs" states that sequence. The original does not define the boundary or name the checking component. If that boundary matters, ask what it means or inspect the code before explaining it.

### Empty sentences and misplaced emphasis

Find the result, decision, request, or next step the reader needs. Put it before the explanation. Check whether each remaining sentence adds information the reader can use.

"Since nothing else depends on anything but the result, I will report as soon as polling returns" buries the promise. "When polling finishes, I will report the result" makes it clear. Add "No other work is waiting" only if the reader needs that fact. Do not drop a condition or uncertainty that changes the promise.

### Review with sentence evidence

- For each defect, identify the sentence, the hidden actor or unclear relationship or buried point, and a rewrite that preserves the supported meaning.
- Accept a sentence that already communicates clearly. When context is missing or reviewers disagree, record the uncertainty instead of forcing a defect label.
- If asked to quantify defects, report defective sentences divided by inspected sentences. Count each sentence once even when it has several defect types, and keep the evidence with the count.
- Treat that rate as a judgment about the reviewed sample. It does not identify AI authorship, measure a model's overall quality, or reproduce a study's result. Punctuation lint checks syntax, not these semantic defects.

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

- **13. Dashes and parentheses.** Banned in prose. End the sentence or use a comma.
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
- **29. Active voice.** When the compiler is the known actor, "Queries are validated" becomes "the compiler validates queries". Passive can be clearer when the actor is unknown or irrelevant, or when the reader needs the existing topic first.
- **30. Cut adverbs.** "significantly improves" becomes the measured delta.
- **31. Prefer the plain word.** "utilize" becomes "use", "facilitate" becomes "help", "in the event that" becomes "if".
- **32. Mannered prose.** Aphorisms, rhetorical fragments, personified code, figurative verbs. Say what you mean.
- **33. Over-compression.** Dropped articles, verbless fragments, and arrows that make the reader decode. Write whole sentences.
