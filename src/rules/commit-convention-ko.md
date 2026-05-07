# Commit Convention

## 커밋 메시지 형식

```
<type>: [<ticket-number>] <title>

<body content>
- Specific changes
- Key logic explanation
```

## 커밋 타입

- feat: 신규 기능 추가
- fix: 버그 수정
- refactor: 코드 리팩토링 (기능 변경 없음)
- style: 코드 포맷팅, 세미콜론 누락 등 (로직 변경 없음)
- docs: 문서 업데이트
- test: 테스트 코드 추가/수정
- chore: 빌드 스크립트, 패키지 매니저 등 기타 작업

## 티켓 번호 형식

- `[PP-XXXX]`: 프로젝트 티켓 번호 (예: PP-6050)
- 티켓 번호는 브랜치 이름에서 확인할 수 있다
- feature 브랜치 이름과 일치해야 한다

## 필수 규칙

**CRITICAL: 사용자가 명시적으로 요청한 경우에만 커밋을 생성한다. 작업 완료 후 자동으로 커밋하지 말 것 — 사용자가 명시적으로 요청해야 한다.**

### 커밋 전 체크리스트

- 작업, 커밋, PR을 작게 유지한다.
- 파일 전체를 꼼꼼히 읽고 영향 범위를 이해한다.
- 테스트가 통과하는지 확인한다 (신규 코드에는 신규 테스트 포함).
- 가정사항은 Issue/PR/ADR에 기록한다.

### 절대적 보안 점검

- 절대 금지: 시크릿(비밀번호/API 키/토큰)을 코드/로그/환경 변수/.env 파일에 커밋하지 않는다.
- 절대 금지: 민감 데이터(PII/카드 정보/SSN)를 커밋하지 않는다.
- 시크릿 발견 시 즉시 커밋을 중단하고 위치를 명시한다.

### 커밋 메시지 규칙

- 제목은 50자 이내로 간결하게 작성
- 본문은 변경 내용과 이유를 구체적으로 설명
- 영어로 작성
- **절대 금지: 이모지(🤖, ✅ 등) 사용 금지**
- **절대 금지: 생성 마커(Co-Authored-By, Generated with Claude Code 등) 금지**
- 의도를 드러내는 명료한 설명을 작성한다

### 커밋 프로세스

- 커밋을 논리 단위로 분리한다 (≤ 300 LOC 파일 한계 준수).
- 커밋 계획을 설명하고 승인 후 진행한다.
- 각 커밋은 독립적으로 빌드 가능하고 테스트 가능해야 한다.

## 금지 패턴

**커밋 메시지에 절대 추가 금지:**

```
❌ 🤖 Generated with [Claude Code](https://claude.com/claude-code)
❌ Co-Authored-By: Claude <noreply@anthropic.com>
❌ Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
❌ Any emojis (🎉, 🐛, ✨, etc.)
❌ Any generation markers or AI attribution
```

**올바른 형식:**

```
✅ chore: update installer binary

- Remove debug logs from installer.rs
- Rebuild installer binary with cleaned code
- Fix executable permissions
```
