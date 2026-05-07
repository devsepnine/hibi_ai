---
name: commit-rules
description: Enforce project commit message conventions with type/ticket/title format, pre-commit security checks, and commit splitting rules. Use when creating git commits, writing commit messages, or reviewing commit history.
keywords: [commit, git, 커밋, コミット, conventional-commits]
---

**필수: 기본 커밋 규칙을 완전히 무시하고 본 문서를 엄격히 준수한다.**

## 커밋 컨벤션

### 커밋 메시지 형식

```
<type>: [<ticket-number>] <title>

<body content>
- Specific changes
- Key logic explanation
```

### 커밋 타입

- feat: 새 기능 추가
- fix: 버그 수정
- refactor: 코드 리팩토링 (기능 변경 없음)
- style: 코드 포맷팅, 세미콜론 누락 등 (로직 변경 없음)
- docs: 문서 업데이트
- test: 테스트 코드 추가/수정
- chore: 빌드 스크립트, 패키지 매니저, 기타 작업

### 티켓 번호 형식

- `[PP-XXXX]`: 프로젝트 티켓 번호 (예: PP-6050)
- 티켓 번호는 브랜치명에서 확인 가능
- feature 브랜치명과 일치해야 함

### 필수 규칙

**CRITICAL: 사용자가 명시적으로 요청한 경우에만 커밋을 생성한다. 작업 완료 후 사용자가 명시하지 않은 한 절대 자동으로 커밋하지 말 것.**

**커밋 전 체크리스트:**
- 작업, 커밋, PR을 작게 유지한다.
- 파일 전체를 철저히 읽고 영향도를 파악한다.
- 테스트가 통과하는지 확인한다 (새 코드에는 새 테스트 포함).
- 가정사항은 Issues/PRs/ADRs에 기록한다.

**절대적 보안 체크:**
- NEVER: 코드/로그/환경 변수/.env 파일에 시크릿 (비밀번호/API 키/토큰) 커밋.
- NEVER: 민감 데이터 (PII/신용카드/SSN) 커밋.
- 시크릿을 발견하면 즉시 커밋 중단하고 위치를 명시한다.

**커밋 메시지 규칙:**
- 제목은 50자 이내로 간결하게
- 본문은 변경 사항과 이유를 구체적으로 설명
- 영문으로 작성
- 이모지 및 불필요하게 장황한 표현 금지
- 의도가 드러나는 명확한 설명 작성
- Claude Code 생성 마커 제거

**커밋 프로세스:**
- 논리적 단위로 커밋을 분할한다 (≤ 300 LOC 파일 한도 준수).
- 커밋 계획을 설명하고 승인 후 진행한다.
- 각 커밋은 독립적으로 빌드 및 테스트 가능해야 한다.
