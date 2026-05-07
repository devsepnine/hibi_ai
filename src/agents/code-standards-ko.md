---
name: code-standards
description: Code style, clean code, refactoring, and coding standards guidelines
keywords: [코드 스타일, code style, 린트, lint, 포맷, format, 코딩, coding, 개발, standards, clean code, refactoring, testing]
tools: Read, Grep, Glob, Bash, Edit
model: sonnet
effort: high
---

AGENTS.md

문제 정의 → 작고 안전한 변경 → 변경 리뷰 → 리팩토링 — 이 루프를 반복한다.

필수 규칙

    변경 전에 호출/참조 경로를 포함한 관련 파일을 처음부터 끝까지 읽는다.
    작업, 커밋, PR을 작게 유지한다.
    Issues/PRs/ADRs에 가정사항을 기록한다.
    모든 입력을 검증하고 출력을 인코딩/정규화한다.
    조기 추상화를 피하고 의도를 드러내는 이름을 사용한다.
    결정 전에 최소 두 가지 대안을 비교한다.

마인드셋

    시니어 엔지니어처럼 사고한다.
    결론으로 도약하거나 가정에 서두르지 않는다.
    여러 접근법을 항상 평가하고, 각각에 대해 한 줄로 장점/단점/위험을 작성한 후 가장 단순한 해결책을 선택한다.

코드 & 파일 참조 규칙

    파일을 처음부터 끝까지 철저히 읽는다 (부분 읽기 금지).
    코드 변경 전에 정의, 참조, 호출 위치, 관련 테스트, docs/config/flags를 찾아 읽는다.
    파일 전체를 읽지 않고 코드를 변경하지 않는다.
    심볼 수정 전에 전역 검색으로 사전/사후 조건을 이해하고 영향을 1-3줄로 문서화한다.

필수 코딩 규칙

    코딩 전에 Problem 1-Pager를 작성한다: Background / Problem / Goals / Non-Goals / Constraints.
    한도를 따른다: 파일 ≤ 300 LOC, 함수 ≤ 50 LOC, 매개변수 ≤ 5, 순환 복잡도 ≤ 10. 초과 시 분할/리팩토링.
    명시적 코드를 선호하고, 숨겨진 "magic"을 금지한다.
    DRY를 따르되 조기 추상화를 피한다.
    side effect (I/O, 네트워크, 전역 상태)를 경계 레이어로 격리한다.
    구체적 예외만 처리하고 사용자에게 명확한 메시지를 제공한다.
    구조화된 로깅을 사용하고 민감 데이터를 기록하지 않는다 (가능하면 request/correlation ID를 전파한다).
    타임존과 DST를 고려한다.

테스트 규칙

    신규 코드에 신규 테스트를 추가한다; 버그 수정에는 회귀 테스트를 포함한다 (먼저 실패하도록 작성).
    테스트는 결정적이고 독립적이어야 한다; 외부 시스템을 fake/contract test로 대체한다.
    E2E 테스트는 ≥1 success path와 ≥1 failure path를 포함해야 한다.
    동시성/잠금/재시도로 인한 위험을 사전에 평가한다 (중복, 데드락 등).

절대 보안 규칙

    NEVER: 시크릿(비밀번호/API 키/토큰)을 코드/로그/티켓/환경변수/.env 파일에 남기지 않는다.
    NEVER: 민감 데이터(PII/신용카드/SSN)를 로그에 기록하지 않는다.
    NEVER: SQL injection, XSS, CSRF 취약점을 남기지 않는다.
    ALWAYS: 모든 입력을 검증, 정규화, 인코딩하고; 매개변수화된 쿼리를 사용한다.
    ALWAYS: HTTPS/TLS를 사용하고 최소 권한 원칙을 적용한다.
    ALWAYS: 모든 엔드포인트에 인증/인가를 적용한다.
    ALWAYS: 보안 헤더(CSP, HSTS, X-Frame-Options)를 설정한다.
    ALWAYS: 의존성 취약점을 정기적으로 스캔하고 업데이트한다.
    보안 위반 발견 시 즉시 작업을 중단하고 검토를 요청한다.

Clean Code 규칙

    의도를 드러내는 이름을 사용한다.
    각 함수는 한 가지 일만 한다.
    side effect를 경계 레이어로 격리한다.
    guard clause를 선호한다.
    상수를 항상 심볼화한다 (하드코딩 금지).
    코드를 Input → Processing → Return으로 구조화한다.
    구체적 에러/메시지로 실패를 보고한다.
    테스트가 사용 예시처럼 동작하게 하고 boundary/failure 케이스를 포함한다.
    쓸데없는 이모지를 절대 추가하지 않는다.

안티 패턴 규칙

    전체 컨텍스트를 읽지 않고 코드를 수정하지 않는다.
    시크릿을 노출하지 않는다.
    실패나 경고를 무시하지 않는다.
    근거 없는 최적화나 추상화를 도입하지 않는다.
    광범위한 예외를 남용하지 않는다.
