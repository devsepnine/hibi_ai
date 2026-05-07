---
description: Update README, CHANGELOG, and project documentation to reflect current state. Reads package.json, .env.example, route definitions.
allowed-tools: Read, Write, Edit, Bash, Grep, Glob
model: sonnet
effort: medium
---

# Update Project Documentation

프로젝트의 현재 상태를 반영하도록 프로젝트 문서를 업데이트하는 작업을 수행한다.

## Your Responsibilities

1. **현재 프로젝트 구조 분석**
   - 새로운 디렉토리나 중요한 변경 사항을 기록한다
   - `package.json`을 읽는다 (scripts 섹션 추출)
   - `.env.example`을 읽는다 (환경 변수 추출)

2. **최근 변경 사항 검토**
   - 커밋되지 않은 변경에 대한 git status 확인
   - `git log --oneline -10`을 사용해 최근 커밋(5-10개) 검토
   - 수정/추가/삭제된 파일 식별
   - 최근 변경 내용 확인을 위해 `git diff HEAD~5..HEAD --stat` 사용

3. **`/docs/README.md` 업데이트**
   - 현재 `/docs/README.md`를 읽는다
   - 디렉토리가 변경되었다면 프로젝트 구조 섹션을 업데이트한다
   - 다음 내용으로 "Recent Changes" 섹션을 추가한다:
     - 마지막 업데이트 날짜
     - 최근 수정 사항 요약
     - 추가된 새로운 피처나 컴포넌트
     - 제거되거나 사용 중단된 항목
   - 문서를 간결하고 잘 구조화된 상태로 유지한다

4. 매니페스트 소스에서 **`/docs/CONTRIB.md` 생성**:
   - 사용 가능한 스크립트 (`package.json` scripts, 설명 포함)
   - 환경 설정 (`.env.example`, 목적과 형식 문서화)
   - 개발 워크플로우
   - 테스트 절차

5. 다음 내용으로 **`/docs/RUNBOOK.md` 생성**:
   - 배포 절차
   - 모니터링 및 알람
   - 일반적인 이슈와 수정 방법
   - 롤백 절차

6. **노후화된 문서 식별**
   - 30일 이상 수정되지 않은 문서를 찾는다
   - 수동 검토를 위해 목록화한다

7. **Diff 요약 표시**
   - 변경된 파일을 요약한다
   - 구조적 변경 사항을 강조한다

## Documentation Format

README는 다음 구조를 따라야 한다:
```markdown
# Project Documentation

> Last Updated: YYYY-MM-DD

## Overview
Brief description of the project

## Project Structure
```
config/ai/claude/
├── agents/          # Custom AI agents
├── commands/        # Slash commands
├── contexts/        # Context definitions
├── hooks/           # Lifecycle hooks
├── ...
```

## Components

### Agents
- Description of agents

### Commands
- List of available commands

### Hooks
- Installed hooks and their purposes

[... other sections ...]

## Recent Changes

### YYYY-MM-DD
- Added: ...
- Modified: ...
- Removed: ...
```

## Important Guidelines

- 변경 전 항상 현재 README를 먼저 읽는다
- 노후화되지 않은 한 기존 콘텐츠를 보존한다
- 명확하고 간결한 언어를 사용한다
- 관련 변경사항을 그룹화한다
- 실제 변경이 있을 때만 "Recent Changes" 섹션을 업데이트한다
- 플레이스홀더나 예시 콘텐츠는 추가하지 않는다
- 설명은 한국어로 작성하고, 코드/경로는 영어로 유지한다
- CONTRIB의 단일 진실 원천: `package.json`과 `.env.example`

## Execution Steps

1. 현재 `/docs/README.md`를 읽는다 (없으면 생성)
2. `package.json`과 `.env.example`을 읽는다
3. `find` 또는 `ls`로 프로젝트 구조를 분석한다
4. git 이력 확인: `git log --oneline -10` 및 `git diff HEAD~5..HEAD --stat`
5. 실제이며 현재의 정보로 README를 업데이트한다
6. `/docs/CONTRIB.md`와 `/docs/RUNBOOK.md`를 생성/업데이트한다
7. 사용자에게 업데이트 내용 요약을 표시한다
