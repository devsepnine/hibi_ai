# Development Workflow

## 핵심 루프

**Define Problem** → **Small Safe Change** → **Review Change** → **Refactor** — 반복.

## Problem 1-Pager

코딩을 시작하기 전, 문제가 복잡하거나 불명확하다면 다음 항목을 포함한 **Problem 1-Pager**를 작성한다. 항목이 모호하면 인터뷰를 요청해 명확히 한다.

* **Background:** 변경의 맥락과 동기.
* **Problem:** 어떤 구체적인 이슈를 해결하려 하는가?
* **Goal:** 성공의 정의는 무엇인가 ("성공 상태")?
* **Non-goals:** 명시적으로 범위 밖에 있는 것은 무엇인가?
* **Constraints:** 필수 기술적/비즈니스 제약 사항.

## 기능 구현 워크플로우

1. **Plan First**
   - **planner** 에이전트로 구현 계획 작성
   - 의존성과 리스크 식별
   - 단계별로 분해

2. **TDD Approach**
   - **tdd-guide** 에이전트 사용
   - 테스트 먼저 작성 (RED)
   - 테스트를 통과시키기 위한 구현 (GREEN)
   - 리팩토링 (IMPROVE)
   - 80%+ 커버리지 검증

3. **Code Review**
   - 코드 작성 직후 **code-reviewer** 에이전트 사용
   - CRITICAL 및 HIGH 이슈 처리
   - 가능하면 MEDIUM 이슈도 수정

4. **Commit & Push**
   - 커밋 메시지는 `commit-convention.md`를 따른다
   - PR은 `pull-request-rules.md`에 따라 생성한다
