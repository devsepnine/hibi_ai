# Epistemics

`why` 답변의 모든 주장은 아래 티어 중 하나에 속한다. 티어가 그 주장이 어느 출력 섹션에 들어가는지와 어떻게 표현하는지를 정한다. pstack의 `why` epistemics를 가져와 다듬었다.

| Tier | 해당 조건 | 표현 | Section |
|---|---|---|---|
| **Direct** | 누군가 이유를 적어 두었다: PR 본문, 티켓, 코드 주석, 설계 문서, 작성자의 채팅 메시지 | "이것은 X 때문에 존재한다", 출처 인용과 함께 | What we found |
| **Supported** | 간접 신호 여럿이 일치하지만 어느 하나도 직접 말하지 않는다 | "증거는 X를 가리킨다: A, B, C", 각각 인용 | What we found |
| **Inferred** | 맥락에 대한 합리적인 해석이지만 어디에도 적혀 있지 않다 | "그런 듯하다", "아마", 추론 사슬과 함께: "A와 B를 고려하면 D 때문에 C일 가능성이 높아 보인다" | What we can infer |
| **Speculative** | 그럴듯하지만 다른 설명도 들어맞는다 | "한 가지 가능성은 X이지만 증거는 찾지 못했다" | Competing readings |
| **Unknown** | 검색했지만 아무것도 찾지 못했다 | "A와 B를 검색어로 X를 검색했으나 근거를 찾지 못했다" | What we don't know |

## 규칙

- 요약하면서 티어를 올리지 않는다. 일치하는 추론 두 개도 그 뒤의 신호가 독립적이지 않으면 여전히 추론이다.
- "코드가 X를 하니 작성자가 X를 원했다"는 Direct 주장이 아니라 추론이다.
- null을 구체적으로 적는다. "알아내지 못했다"보다 "2023년 이후 이 파일을 건드린 PR 6개를 읽고 상수 값으로 Jira를 검색했으나 이유를 준 것은 없었다"가 더 도움이 된다.
- 어떤 출처가 이유를 밝히고 더 새로운 출처가 그것을 되풀이할 뿐이면 오래된 쪽을 택한다.
- 이 티어는 `CLAUDE.md`의 measured, inferred, guess 라벨을 세분한 것이다: Direct 주장은 measured, Supported와 Inferred는 inferred, Speculative는 guess에 해당한다.
