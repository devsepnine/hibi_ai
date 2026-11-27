---
name: verification-loop
description: Build, type, test, and security verification loop. Use before reporting completion or when a build or type check fails. 빌드 검증, 타입 체크, 테스트 실행, 검증 루프, 완료 전 검증.
---

# Verification Loop Skill

Claude Code 세션을 위한 종합 검증 시스템.

Phase 1~6은 대리 지표다: build·타입 체크·테스트가 green이면 코드가 일관적이라는 뜻이지, 변경이 요청대로 동작한다는 뜻이 아니다. Phase 7이 실제 결과물을 확인한다. Phase 7을 포함한 실행만 `Overall: READY`를 보고할 수 있다. `/verify quick`, `pre-commit`, 지속 모드 같은 부분 범위는 `Overall: PARTIAL`을 보고하고, Phase 7은 완료 시점에 한 번 실행한다.

## 사용 시점

다음 상황에 이 skill을 호출한다:
- 기능 완료 또는 의미 있는 코드 변경 직후
- PR 생성 전
- 품질 게이트 통과를 보장하고 싶을 때
- 리팩토링 후

## 실행 준비

코드, 빌드, 테스트, 개발 서버를 실행하기 전에:

1. `rundebugconfig` MCP 도구를 찾는다. 제공된다면 `list_configurations`로 연결을 확인하고 현재 프로젝트의 설정을 찾는다.
2. 연결되어 있으면 후보 설정을 `get_configuration`으로 읽어 디렉터리와 명령이 작업에 맞는지 확인한다. 새로 실행하기 전에 `list_sessions`를 확인한다. 로그 조회나 실행 중인 서버 사용에는 관련된 기존 세션을 활용한다.
3. 작업에 맞는 저장된 설정이 있고 도구가 제공되면 `run_configuration`을 우선 사용한다. `read_session_output`과 `list_sessions`로 출력, 최종 상태, 종료 코드를 확인한다. 세션 ID 반환은 세션이 생겼다는 증거일 뿐이다. 서버는 준비 상태와 요청한 동작을 확인한다.
4. 도구가 없는 이유는 앱의 권한 등급일 수 있다. 실행 도구가 없어도 제공되는 조회 도구를 활용한다. 명시적으로 거부된 권한을 셸 실행으로 우회하지 않는다.
5. MCP를 사용할 수 없거나, 맞는 설정이 없거나, 실행 도구가 없으면 한계를 밝히고 별도로 허용된 셸 명령을 사용한다. 검증 한 번을 위해 저장된 설정을 만들거나 덮어쓰지 않는다. 시작 요청의 결과가 불확실하면 재시도나 대체 실행 전에 세션을 확인해 프로세스 중복을 피한다.

검사 명령은 프로젝트 문서, 매니페스트, CI에서 선택하고 위 절차로 실행한다. 해당 단계가 없는 프로젝트는 이유와 함께 `N/A`로 표시하고 적용되는 검사를 계속한다.

## 검증 단계

### Phase 1: 빌드

배포에 빌드가 필요하면 프로젝트의 빌드 명령을 실행한다. 빌드에 의존하는 검사는 빌드 실패를 해결한 뒤 실행한다. 소스에서 바로 실행하는 Python CLI는 실제 CLI 진입점을 검증한다.

### Phase 2: 타입 검사

프로젝트가 구성한 검사기와 설정을 사용한다. 예를 들어 TypeScript의 타입 검사 스크립트나 Rust의 `cargo check`를 실행한다.

### Phase 3: 린트

프로젝트에서 해당 파일에 적용하는 린터와 포매터를 실행한다.

### Phase 4: 테스트

저장소의 테스트 러너로 변경된 동작을 확인한다. 통과, 실패, 건너뛴 테스트 수를 기록한다. 커버리지 기준이 있으면 프로젝트가 설정한 한도와 측정 결과를 비교한다.

### Phase 5: 보안 검사

변경된 데이터와 권한 경계를 따라 입력 처리, 자격 증명 노출, 로그를 확인한다. 저장소 스캐너가 있으면 대상 파일 형식에 맞춰 실행한다. 시크릿 후보는 경로만 보고한다. 셸에서 검색할 때는 그 이름이 실행하는 도구와 해당 도구의 파일 필터가 검사 대상 경로를 포함하는지 확인한다. 일반적인 `rg` 또는 `grep` 검색에서 종료 코드 1은 일치 없음이고 1보다 크면 검색 오류다.

### Phase 6: Diff Review
```bash
# Show what changed
git status --short
git diff --stat
git diff --cached --stat
```

변경된 각 파일에 대해 검토:
- 의도하지 않은 변경
- 누락된 에러 처리
- 잠재적 엣지 케이스

`coding-standards`의 `references/review-checklist.md` 중 셀프리뷰와 PR 리뷰의 실패 경로를 적용한다. 관련 실패 시나리오, 근거, 검증하지 못한 한계를 정상 동작 검사와 구분해서 기록한다. 테스트 사례 설계에는 `tdd-workflow`를 쓴다.

### Phase 7: 실제 결과물 관찰

변경된 동작을 직접 실행하고 결과를 읽는다:

| 변경 | 확인 |
|------|------|
| CLI | 실제 입력으로 실제 커맨드 실행 |
| UI | 실행 중인 앱에서 변경된 플로우를 직접 조작한다. `/e2e`, `run` skill, 브라우저 자동화를 쓴다 |
| 파서 / 마이그레이션 | 저장해 둔 실제 입력을 재생하고 출력을 diff |
| 저장소 / 설정 | 쓴 값을 다시 읽음 |
| 성능 | 같은 하네스에서 전후 측정값 비교 |
| 문서만 | 독자의 입장에서 diff를 다시 읽음; 동작을 좌우하는 텍스트, 즉 skill·agent·command 프롬프트가 바뀐 경우에만 eval 실행 |

- 캐시되거나 파생된 표현, 예를 들어 파일 mtime, 에이전트의 자기 보고, 오래된 스크린샷이 아니라 실제 값을 읽는다.
- 변경한 흐름에 도달 가능한 실패가 있으면 정상 동작과 대표 실패를 함께 실행하고 약속한 실패 후 상태를 관찰한다. 격리된 데이터와 제어할 수 있는 의존성을 쓴다. 실행할 수 없으면 `INCONCLUSIVE`와 부족한 근거를 보고하고, 적용할 실패 시나리오가 없으면 이유와 함께 `N/A`로 표시한다.
- UI 흐름을 변경했다면 정상 흐름과 변경의 영향을 받는 서로 다른 실패 상태를 각각 실행한다. 화면에 표시된 문구, 동작 버튼의 활성 여부, 요청이나 상태 변경이 예상대로 일어났는지 확인한다. UI가 복구 수단을 제공하면 복구도 실행한다. 프로젝트의 브라우저 테스트 환경을 사용한다. 실제 외부 호출이 검증 범위를 벗어나면 로컬 Mock을 사용한다. 실제 연동이 검증 범위에 포함되면 승인된 테스트 환경에서 확인한다. 어떤 의존성을 Mock으로 대체했는지 보고한다.
- 정상 화면과 각 오류 화면은 해당 화면의 검증 조건을 충족하는 즉시 캡처한다. 입력 수정, 재시도, 닫기, 화면 이동 전에 오류 문구와 관련 컨트롤이 함께 보이는 상태를 저장한다. 테스트용 데이터를 사용하고 파일명에 시나리오와 브라우저를 표시한다. 이미지를 열어 의도한 상태가 보이는지 확인하고 테스트 결과에 링크한다. 실행하거나 캡처하지 못한 UI 상태도 기록한다.
- 확인이 실패하면 시스템보다 관찰 방법을 먼저 의심한다.
- 한 번 눈으로 보기보다 비교를 다시 돌릴 수 있는 스크립트를 선호하고, 그 출력을 증거로 남긴다.
- 직접 돌릴 수 있는 확인을 사용자에게 넘기지 않는다. 기기나 자격증명이 없어 여기서 정말 실행할 수 없다면 `INCONCLUSIVE`로 표시하고 무엇이 판정할지 적는다. `INCONCLUSIVE`이거나 변경된 곳과 다른 환경에서 실행한 확인은 `NOT READY`다.

## 출력 포맷

여러 항목을 전수 검토할 때는 목록 확인, 연관된 계약 대조, 실제 실행의 범위를 각각 기록한다. 예를 들어 상세 예제의 호출자를 진입 문서에 선언된 인터페이스와 함께 컴파일한다. 반복 리뷰에서는 이전 회차에 빠진 관련 경로를 확인하고, 결함이 0건이면 그 판정이 적용되는 범위를 적는다.

모든 phase를 실행한 후 검증 보고서를 산출한다:

```
VERIFICATION REPORT
==================

Build:     [PASS/FAIL/N/A]
Types:     [PASS/FAIL/N/A] (X errors)
Lint:      [PASS/FAIL/N/A] (X warnings)
Tests:     [PASS/FAIL/N/A] (passed, failed, skipped; coverage if measured)
Security:  [PASS/FAIL/N/A] (X issues)
Diff:      [X files changed]
Artifact:  [PASS/FAIL/INCONCLUSIVE] (what was run, what was observed)
Failure paths: [PASS/FAIL/INCONCLUSIVE/N/A] (scenarios, state after failure, evidence or limits)
UI evidence: (links to success and failure screenshots when applicable)

Overall:   [READY/NOT READY/PARTIAL] for PR

Issues to Fix:
1. ...
2. ...
```

보고서와 완료 메시지의 모든 주장에 **measured**, 즉 직접 실행해 출력을 본 것, **inferred**, 즉 측정된 것에서 따라 나오는 것, **guess** 중 하나를 붙인다. inferred 주장은 inferred로 보고하고, 판정은 measured 주장만 근거로 삼는다.

## 지속 모드

긴 세션에서는 15분마다 또는 주요 변경 후 검증을 실행한다:

```markdown
멘탈 체크포인트 설정:
- 함수 완료 후
- 컴포넌트 완료 후
- 다음 작업으로 넘어가기 전

실행: /verify
```

## Hooks와의 통합

이 skill은 PostToolUse 훅을 보완하면서 더 깊은 검증을 제공한다.
훅은 이슈를 즉시 잡고, 이 skill은 종합 리뷰를 제공한다.
