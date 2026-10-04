---
description: Change structure without changing behavior. Pin the behavior first, subtract before adding, move in small green steps, and keep the result only if it lowers reader load.
argument-hint: "[target file | module | smell]"
allowed-tools: Agent, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# Refactor

구조는 바뀌고 동작은 바뀌지 않는다. 정리 중에 실제 버그나 빠진 기능이 드러나면 분리한다: 고정한 동작을 기준으로 구조 변경을 먼저 배포하고, 버그는 그다음 `/bugfix`로 고친다. 동작을 바꾸는 재설계는 기능이다. 그렇게 이름 붙이고 기능으로 계획한다.

## 단계

1. **동작을 먼저 고정한다.** 구조를 움직이기 전에 현재 동작을 담는 characterization 테스트, snapshot, 또는 동등성 스크립트를 쓴다. 타입 체크와 lint는 고정이 아니다. 고정이 현실적으로 불가능하면 그렇다고 말하고 멈춘다. 대체 검사를 허용할지는 `tdd-workflow`의 티어 면제가 정한다.
2. **코드에 빠진 구조에 이름을 붙인다**: 서로 맞아야 하는 boolean 대신 union, `else if` 사슬 대신 table, 플래그 대신 상태 머신. 재구성은 간접 계층을 더하는 것이 아니라 분기나 불가능한 상태를 지워야 한다. 이미 명확하고 국소적인 평범한 코드는 그대로 둔다.
3. **목표 형태에 이름을 붙인다**: 오늘 새로 만든다면 택할 모듈 배치, 타입, 호출 그래프. 함수 경계를 넘으면 먼저 `architect`를 실행한다.
4. **더하기 전에 뺀다.** 새 형태를 만들기 전에 죽은 코드를 지우고, 호출자가 하나뿐인 wrapper와 pass-through 메서드를 접고, 중복 검증을 없애고, 고아 참조를 제거한다. 죽은 코드만 정리하는 작업이면 대신 `/refactor-clean`과 그 `refactor-cleaner` 에이전트를 쓴다.
5. **작은 단계로 움직이고, 단계마다 고정을 green으로 유지한다.**
   - API 재구성이면 같은 변경 안에서 모든 호출자를 옮기고 옛 API를 지운다. 외부 소비자가 의존하지 않는 한 호환 shim을 두지 않으며, 둔다면 제거 날짜를 정한다.
   - 이름을 바꾼 뒤에는 문자열, 문서, 주석까지 트리 전체에서 옛 이름을 grep한다. `coding-standards`의 주석 규칙이 적용된다.
6. **실제 결과물에서 동작이 바뀌지 않았음을 증명한다.** 고정을 다시 실행하고, 큰 재구성이면 기록해 둔 입력에 대해 옛 출력과 새 출력을 diff한다.
7. **독자 부담이 줄어들 때만 남긴다**: 분기, wrapper, 개념, 열어야 할 파일이 줄어야 한다. diff가 어디서도 그것을 줄이지 못하면 되돌린다.
8. **단계를 분리 가능하게 둔다.** 빼기, 재구성, 정리 순서로 각각 따로 커밋할 수 있게 한다. 커밋은 사용자가 요청할 때만 한다.
9. **보고 전에 `CLAUDE.md`의 작업 후 `code-reviewer` 게이트를 실행한다.** 엄격도는 변경의 티어가 정한다.

## 응답

바뀐 구조, 기준으로 삼은 고정, 동등성 증명, 독자 부담의 변화, 남긴 것과 되돌린 것. 새 동작은 없다.
