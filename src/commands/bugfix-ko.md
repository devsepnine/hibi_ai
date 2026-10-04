---
description: Fix a bug scientifically. Reproduce it on the reported surface, bisect the cause with runtime evidence, ship the smallest fix the evidence justifies, and prove it on the same surface.
argument-hint: "[bug report | issue | failing command]"
allowed-tools: Agent, Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: high
---

# Bug Fix

배포되는 모든 줄은 런타임 증거로 거슬러 올라간다. "도움이 될지도 모르는" 변경은 수정이 아니라 가설이며, 배포하지 않는다. 증거가 가설을 반박하면 그 가설 때문에 넣은 것을 되돌린다. `$ARGUMENTS`가 보고 내용이다. 없으면 증상과 어디서 봤는지를 묻는다.

## 단계

1. **보고된 표면에서 직접 재현한다.** 실제 커맨드를 실행하거나, `run` skill이나 브라우저 자동화로 UI를 조작하거나, 요청을 재생한다. 사용자에게 재현을 부탁하는 것은 이 세션이 대상에 닿을 수 없는 구체적인 이유가 있을 때, 그리고 갈 수 있는 데까지 직접 가 본 뒤에만 한다. 재현되지 않으면 트리거를 합성하거나, 조건을 좁히거나, 재현될 때까지 계측을 넣는다.
2. **원인을 이분 탐색한다.** 후보 가설을 나열하고 하나씩 제거한다. 매번 남은 공간을 가장 많이 없애는 분기를 골라 런타임 증거를 얻는다: 로그 한 줄, probe, 디버거 정지, 회귀라면 `git bisect`. 프로그램 상태가 불분명하면 계측하고 코드가 실행되는 동안 읽는다. 추측하지 않는다. 같은 가정을 공유한 수정이 두 번 실패하면 멈추고 그 가정을 검증한다.
3. **수정을 쓰기 전에 메커니즘을 런타임 증거로 확인한다.** 수정의 일부가 아닌 계측은 제거한다.
4. **수정을 계획한다.** 함수 경계를 넘으면 먼저 `architect`를 실행한다. 버그에 싼 로컬 테스트 경로가 있으면 `tdd-workflow`에 따라 실패하는 테스트를 먼저 쓴다. 그렇지 않으면 그 skill이 정의한 티어 면제 범위 안에서 가장 가까운 실행 가능한 재현을 유지한다.
5. **증거가 정당화하는 가장 작은 변경을 한다.** 크래시를 침묵시키는 guard는 증상 수정이며, 이것이 아니다.
6. **같은 표면에서 검증한다.** 원래 재현이 이제 통과한다. `INCONCLUSIVE`나 다른 표면에서의 통과는 통과가 아니다. `verification-loop` Phase 7을 본다. unit 테스트는 분기 동작을 보여줄 뿐, 버그가 사라졌다는 것을 보여주지 않는다.
7. **같은 유형의 버그가 다른 곳에 있는지 확인하고**, 수정이 공유 코드를 건드리면 `/blast-radius`를 실행한다.
8. **재현과 수정을 분리 가능하게 둔다.** 그래야 실패하는 테스트가 히스토리에서 수정보다 먼저 들어갈 수 있다. 커밋은 사용자가 요청할 때만 한다.
9. **보고 전에 `CLAUDE.md`의 작업 후 `code-reviewer` 게이트를 실행한다.** 엄격도는 변경의 티어가 정한다.

## 응답

무엇이 깨졌는지, 근본 원인, 수정, 어떻게 검증했는지. 실패-전과 통과-후 출력을 그대로 붙이고, 각 주장에 measured, inferred, guess를 표시한다.
