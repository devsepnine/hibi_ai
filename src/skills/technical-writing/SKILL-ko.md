---
name: technical-writing
description: "Technical writing standard: one Diátaxis mode per doc, plain direct sentences, an AI-slop cut list, and no dashes or parentheses in prose. Use when writing or reviewing docs, READMEs, guides, or skill and agent text. 기술 문서 작성, 문서 리뷰, AI 문체 제거, 글 다듬기."
---

# Technical Writing

목표는 지친 엔지니어가 한 번 읽고 이해하는 글이다. 이 skill은 글이 어떻게 읽히는지를 소유한다. 문서에 무엇이 들어가야 하는지는 다른 skill이 소유한다. PR 본문은 `pull-request`, 커밋 메시지는 `commit-rules`, 각 템플릿은 `qa-handoff`와 `obsidian-notes`가 맡는다. 이 skill은 그 위에 얹어서 적용한다.

나머지 규칙보다 위에 세 가지 규칙이 있다:

- **일하지 않는 단어는 모두 지운다.** "In order to"는 "to"다. "It is important to note that"는 아무것도 아니다.
- **짧고 일상적인 단어를 쓴다.** "utilize"가 아니라 "use", "facilitate"가 아니라 "help".
- **규칙이 문장을 더 나쁘게 만들면 다른 방법으로 문장을 고친다.** 규칙은 독자를 위한 것이다. 모든 규칙을 지켰는데 기계가 쓴 것처럼 읽히는 문장은 실패한 것이다.

동의어나 설명 대신 실제 심볼, 파일, 플래그, 커맨드 이름을 쓴다.

## 문장부호: dash 금지, 괄호 금지

산문에는 em dash, en dash, dash 대신 쓴 하이픈, 괄호를 쓰지 않는다. 문장을 끝내거나 쉼표로 잇는다. 콜론은 목록이나 예시 앞, 그리고 "**라벨**: 텍스트"처럼 라벨 뒤에서는 괜찮다. dash를 그대로 바꿔 끼우는 용도로는 쓰지 않는다.

| 대신 | 이렇게 쓴다 |
|---|---|
| `X — Y` | `X. Y` 또는 `X, Y` |
| `X (보충) Y` | 보충을 문장에 녹이거나, 별도 문장으로 만들거나, 지운다 |
| `MC/DC (modified condition/decision coverage)` | `MC/DC, 곧 modified condition/decision coverage` |
| `50–100ms` | `50에서 100ms` |

코드, 인라인 코드, 링크 대상, wikilink, frontmatter 키는 문법이므로 그대로 둔다. 규칙을 피하려고 일반 산문을 백틱으로 감싸지 않는다. `: `를 포함한 `description:` 값은 따옴표로 감싸야 한다. 그렇지 않으면 YAML이 거부한다.

이 규칙은 기억이 아니라 강제로 지킨다: 완료를 보고하기 전에 `python tools/lint-prose.py [paths]`를 실행한다. 위반마다 `file:line`을 출력하고 exit 1로 끝난다.

## 먼저 모드를 고른다: Diátaxis

문서 하나에 모드 하나. 두 질문으로 고른다. 내용이 행동을 돕는가, 이해를 돕는가? 학습을 위한 것인가, 일을 위한 것인가?

- **Tutorial**, 행동과 학습. 모든 단계가 눈에 보이는 결과를 낸다. 독자가 무엇을 보게 될지 말해 준다.
- **How-to**, 행동과 일. 독자가 가진 문제를 푼다. 역량을 전제하고, 가르치지 않으며, 분기를 허용한다: "x를 원하면 y를 한다."
- **Reference**, 이해와 일. 기술하고, 기술만 한다. 기술하는 대상의 구조를 그대로 따른다.
- **Explanation**, 이해와 학습. 범위가 정해진 주제 하나를 실제 why 질문에 고정한다. 의견은 여기에만 쓴다.

모드를 섞지 않는다. 나누고 링크한다.

## 독자에게 말하듯 쓴다

- 독자를 "you"로, 현재 시제로 부른다.
- 누가 무엇을 하는지 말한다: "is checked"가 아니라 "the compiler checks".
- 조건을 지시 앞에 둔다: "문서를 삭제하려면 Delete를 누른다."
- 흔한 경우를 먼저, 예외를 뒤에 둔다.
- 문장 하나에 지시 하나. 약 20단어를 넘는 지시는 나눈다.
- "only"와 "not"은 수식하는 단어 옆에 둔다.
- "it", "they", "this"가 분명한 하나를 가리키게 한다. 애매하면 명사를 반복한다.
- 한 대상은 어디서나 한 이름으로 부른다. 바뀌지 않은 문장은 다시 쓰지 않는다.
- 제목은 주제가 아니라 요점을 담는다: "Modes"가 아니라 "Pick the mode first".
- 절차에 "simply", "easy", "quickly"를 쓰지 않는다.

## AI 문체를 걷어낸다

예시가 있는 전체 목록은 `references/slop-patterns.md`에 있다. 가장 자주 나오는 것:

- **AI 어휘**: delve, crucial, pivotal, leverage, showcase, underscore, landscape, tapestry, robust, seamless. 평이한 단어를 쓴다.
- **화려한 "is"**: "serves as", "stands as", "boasts". "is"나 "has"로 쓴다.
- **추상 은유 명사**: substrate, vector, surface, north star, flywheel. 구체적인 대상을 말한다.
- **메커니즘 대신 느낌**: "types that follow your schema"는 아무것도 말하지 않는다. "컬럼 이름을 바꾸면 빌드가 실패한다"는 말한다. 다른 프로젝트 문서에 그대로 들어가도 되는 문장이라면 지운다.
- **과압축**: "bad date → exit 2, no write"는 독자가 해독해야 한다. "파서는 잘못된 날짜를 거부하고, 코드 2로 종료하며, 아무것도 쓰지 않는다"라고 쓴다.
- **챗봇·군더더기 표현**: "도움이 되었길 바랍니다", "좋은 질문입니다", "주목할 만한 점은". 지운다.

## 한국어 산문

같은 규칙이 한국어에도 적용되며, 다음을 더한다:

- 여기서도 dash나 괄호로 보충하지 않는다. `X, 즉 Y` 또는 `X. Y`로, `MC/DC, 곧 modified condition/decision coverage`로 쓴다.
- 한 개념에는 영어 용어 하나만 쓰고 두 번 병기하지 않는다. 한 문서 안에서는 `결합도`든 `coupling`이든 하나를 골라 유지한다.
- 번역투를 걷어낸다: 평범한 동사로 충분할 때의 "~하는 것이 중요하다", "~에 있어서", "~를 통해".

## 넘기기 전에

- [ ] 문서가 Diátaxis 모드 하나다
- [ ] 모든 문장이 메커니즘, 사실, 지시 중 하나를 말한다
- [ ] `references/slop-patterns.md`의 패턴이 없다
- [ ] `python tools/lint-prose.py <changed paths>`가 아무것도 출력하지 않는다
- [ ] EN과 `-ko.md` 미러가 여전히 같은 말을 한다
