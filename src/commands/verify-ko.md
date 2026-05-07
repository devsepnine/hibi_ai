---
description: Run comprehensive verification (build, type check, tests, lint). Reports failures with file:line context.
argument-hint: "[--quick|--full]"
allowed-tools: Bash, Read, Grep
model: haiku
effort: low
---

# Verification Command

현재 코드베이스 상태에 대해 종합 검증을 실행한다.

## Instructions

다음 순서대로 검증을 실행한다:

1. **Build Check**
   - 이 프로젝트의 빌드 명령을 실행한다
   - 실패 시 오류를 보고하고 중단한다

2. **Type Check**
   - TypeScript/타입 체커를 실행한다
   - file:line과 함께 모든 오류를 보고한다

3. **Lint Check**
   - 린터를 실행한다
   - 경고 및 오류를 보고한다

4. **Test Suite**
   - 모든 테스트를 실행한다
   - pass/fail 개수를 보고한다
   - 커버리지 비율을 보고한다

5. **Console.log Audit**
   - 소스 파일에서 console.log를 검색한다
   - 위치를 보고한다

6. **Git Status**
   - 커밋되지 않은 변경 사항을 표시한다
   - 마지막 커밋 이후 수정된 파일을 표시한다

## Output

간결한 검증 보고서를 생성한다:

```
VERIFICATION: [PASS/FAIL]

Build:    [OK/FAIL]
Types:    [OK/X errors]
Lint:     [OK/X issues]
Tests:    [X/Y passed, Z% coverage]
Secrets:  [OK/X found]
Logs:     [OK/X console.logs]

Ready for PR: [YES/NO]
```

중요한 이슈가 있을 경우 수정 제안과 함께 나열한다.

## Arguments

$ARGUMENTS는 다음 중 하나일 수 있다:
- `quick` - 빌드 + 타입만
- `full` - 모든 검사 (기본값)
- `pre-commit` - 커밋과 관련된 검사
- `pre-pr` - 전체 검사 + 보안 스캔
