# Slop patterns

이 패턴을 찾아서 다시 쓴다. 의미와 의도한 어조는 보존한다. 번호가 있는 목록은 pstack의 `unslop`을 바탕으로 썼고, 두 목록을 비교할 수 있도록 규칙 번호를 그대로 유지했다.

## 독자가 해석해야 하는 부분을 찾는다

문장이 독자에게 어떤 해석을 맡기는지 살핀다. 익숙한 전문 용어나 긴 문장 자체는 결함이 아니다.

### 숨은 행위자와 행동

누가 무엇을 하는지 찾고 문법상의 주어, 동사와 비교한다. "파일 비교의 수행이 설치기에 의해 이루어졌다"는 독자가 "비교"라는 명사에서 행동을 되살려야 한다. "설치기가 파일을 비교했다"는 행동을 바로 보여 준다.

명사형 표현과 수동태가 유용할 때도 있다. "설정 파일이 삭제됐다. 누가 삭제했는지는 아직 확인하지 못했다"는 행위자를 모르는 상황을 정확하게 전한다. 사람이나 컴포넌트를 지어내어 주어에 넣지 않는다.

### 모호한 이름과 관계

각 지칭이 가리키는 대상과 나란히 놓인 명사들의 관계를 확인한다. 낯선 이름은 독자나 코드가 쓰는 이름으로 바꾼다. 일상어로 바꾸면 뜻이 사라지는 정확한 전문 용어는 유지한다.

"데이터베이스 준비 상태 확인은 인증 핸들러 실행 전, 사용자 세션 진입 경계에 위치한다"는 확인할 수 있는 순서를 흐린다. "인증 핸들러가 실행되기 전에 데이터베이스가 준비됐는지 확인한다"는 그 순서를 드러낸다. 원문은 경계의 뜻과 확인 주체를 밝히지 않는다. 그 경계가 중요하면 뜻을 묻거나 코드를 살핀 뒤 설명한다.

### 내용 없는 문장과 잘못된 강조

독자에게 필요한 결과, 결정, 요청, 다음 작업을 찾아 설명보다 먼저 쓴다. 남은 문장마다 독자에게 쓸모 있는 정보를 더하는지 확인한다.

"다른 어떤 것도 결과 외에는 아무것에도 의존하지 않으므로, 폴링이 반환되는 즉시 보고하겠다"는 약속을 뒤에 숨긴다. "폴링이 끝나면 결과를 보고하겠다"는 약속을 바로 전한다. "그 외에 대기 중인 작업은 없다"는 독자에게 필요한 사실일 때만 덧붙인다. 약속의 뜻을 바꾸는 조건이나 불확실성은 지우지 않는다.

### 문장별 근거로 검수한다

- 결함마다 해당 문장, 숨은 행위자나 모호한 관계 또는 뒤에 묻힌 핵심, 확인된 뜻을 보존한 수정안을 제시한다.
- 이미 명확한 문장은 인정한다. 맥락이 부족하거나 검수자 사이에 판단이 다르면 결함으로 단정하지 말고 불확실성을 기록한다.
- 결함 비율을 요청받으면 결함이 있는 문장 수를 검수한 문장 수로 나눠 보고한다. 한 문장에 여러 결함이 있어도 한 번만 세고, 수치와 함께 판단 근거를 남긴다.
- 그 비율은 검수한 표본에 대한 판단이다. AI 작성 여부나 모델 전체의 품질을 판별하거나 연구 결과를 재현한 수치가 아니다. 문장부호 린트는 문법을 검사하며 이 의미상의 결함은 판정하지 못한다.

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
- **29. Active voice.** 컴파일러가 행위자라고 확인됐을 때 "Queries are validated"를 "the compiler validates queries"로 바꾼다. 행위자를 모르거나 중요하지 않을 때, 또는 독자가 기존 화제를 먼저 알아야 할 때는 수동태가 더 명확할 수 있다.
- **30. Cut adverbs.** "significantly improves"는 측정한 변화량으로 바꾼다.
- **31. Prefer the plain word.** "utilize"는 "use"로, "facilitate"는 "help"로, "in the event that"은 "if"로 바꾼다.
- **32. Mannered prose.** 격언, 수사적 단편, 의인화한 코드, 비유적 동사. 하고 싶은 말을 그대로 쓴다.
- **33. Over-compression.** 관사 생략, 동사 없는 단편, 독자가 풀어 읽어야 하는 화살표. 완전한 문장으로 쓴다.
