# Slop patterns

이 패턴을 찾아서 다시 쓴다. 의미와 의도한 어조는 보존한다. pstack의 `unslop`에서 가져왔고, 두 목록을 비교할 수 있도록 규칙 번호를 그대로 유지했다.

## Content

- **3. Superficial -ing phrases.** "highlighting...", "ensuring...", "reflecting...", "fostering...". 지우거나 실제 출처로 풀어 쓴다.
- **5. Vague attributions.** "Experts believe", "Industry reports suggest". 출처를 밝히거나 그 주장을 지운다.

## Language

- **7. AI vocabulary.** Additionally, crucial, delve, enduring, enhance, fostering, garner, interplay, intricate, landscape, pivotal, robust, seamless, showcase, tapestry, testament, underscore, vibrant. 평이한 단어로 바꾼다.
- **8. Fancy ways to say "is".** "serves as", "stands as", "boasts", "features". "is"나 "has"로 쓴다.
- **9. "Not just X, but Y."** 요점을 곧바로 말한다.
- **10. Rule of three.** 아이디어를 억지로 셋씩 묶는 것. 자연스러운 개수를 쓴다.
- **11. Synonym cycling.** 한 문단에서 한 대상을 이름 넷으로 부르는 것. 하나를 골라 반복한다.
- **12. False ranges.** X와 Y가 실제 척도 위에 있지 않은 "from X to Y". 항목을 나열한다.

## Style

- **13. Dashes and parentheses.** 산문에서는 금지다. 문장을 끝내거나 쉼표를 쓴다. `tools/lint-prose.py`가 강제한다.
- **14. Colon overuse.** 콜론은 목록이나 예시 앞, 라벨 뒤에서는 괜찮다. 문장 중간의 연결어로는 쓰지 않는다.
- **15. Boldface overuse.** 고유명사나 약어마다 굵게 쓰지 않는다.
- **16. Inline-header lists.** 굵은 라벨 뒤의 줄이 라벨을 그대로 되풀이하는 경우. 예를 들어 "**Performance**: performance improved". 산문으로 바꾼다. 라벨 뒤에 정말 새로운 내용이 오면 괜찮다.
- **17. Title case headings.** 문장형 대소문자를 쓴다.
- **18. Decorative emojis.** 지운다.
- **19. Curly quotes.** 곧은 따옴표를 쓴다.

## Communication artifacts

- **20. Chatbot phrases.** "I hope this helps!", "Let me know if...", "Certainly!". 지운다.
- **22. Sycophantic tone.** "Great question! You're absolutely right!" 바로 답한다.

## Filler

- **23. Filler phrases.** "In order to"는 "to"로 바꾼다. "Due to the fact that"은 "because"로 바꾼다. "It is important to note that"은 지운다.
- **24. Excessive hedging.** "could potentially possibly be argued that it might"는 "may"로 바꾼다.
- **25. Generic conclusions.** "The future looks bright." 구체적인 계획이나 사실을 쓴다.

## Jargon

- **26. Abstract metaphor nouns.** Substrate, wedge, vector, locus, nexus, 명사로 쓴 primitive, "API surface"처럼 쓴 surface, bedrock, 은유로 쓴 scaffolding, paradigm, gold-plating, 은유로 쓴 ratchet, endgame, north star, flywheel. "Substrate"는 "base"로 바꾼다. "Vector"는 "way"로 바꾼다. "Gold-plating"은 "more than the job needs"로 바꾼다. "Ratchet"은 "a limit that only tightens"로 바꾼다.

## Plain speech

- **27. Say what it does, not how it feels.** "SQL you can read"는 느낌을 말한다. "`.toSQL()` returns the exact string sent to the database"는 메커니즘을 말한다. 다른 프로젝트 문서에 그대로 들어갈 수 있는 문장이라면 이 프로젝트에 대해 아무것도 말하지 않는 것이다.
- **28. Shorten or split dense sentences.** 독자가 되돌아가 읽어야 한다면 문장을 둘로 나눈다.
- **29. Active voice.** "Queries are validated"는 "the compiler validates queries"로 바꾼다. 수동태는 행위자를 모르거나 중요하지 않을 때만 괜찮다.
- **30. Cut adverbs.** "significantly improves"는 측정한 변화량으로 바꾼다.
- **31. Prefer the plain word.** "utilize"는 "use"로, "facilitate"는 "help"로, "in the event that"은 "if"로 바꾼다.
- **32. Mannered prose.** 격언, 수사적 단편, 의인화한 코드, 비유적 동사. 하고 싶은 말을 그대로 쓴다.
- **33. Over-compression.** 관사 생략, 동사 없는 단편, 독자가 풀어 읽어야 하는 화살표. 완전한 문장으로 쓴다.
