---
name: pull-request
description: Create pull requests following project conventions including title format, PR template, pre-PR checklist, security checks, and review guidelines. Use when creating, updating, or reviewing GitHub pull requests.
keywords: [pull-request, PR, github, gh, 풀리퀘스트]
---

## PR 제목 형식

```
[TICKET-ID] <One-line Summary>
```

예시:
- `[PP-XXXX] Add user authentication system`
- `[PP-XXXX] Fix payment module bug`

## PR 설명 템플릿

```markdown
#### Issue Type
- [ ] feat (feature add) / [ ] feat (feature remove)
- [ ] fix (bug fix)
- [ ] refactor / [ ] perf / [ ] chore / [ ] style / [ ] docs / [ ] test

#### Priority
> Per JIRA Priority criteria
- [ ] Blocker / [ ] Urgent / [ ] Critical / [ ] Major / [ ] Trivial

#### Background
> What this PR does and why.

#### Changes
> Major modifications. Add reviewer notes for non-obvious parts.

**API Changes:**
- [ ] No Breaking / [ ] Breaking (affects backward compat)

**Database Changes:**
- [ ] No schema / [ ] Schema (migration required) / [ ] Data migration

**Major Files:** `path/file.ext` — summary

#### Testing
**Automated:**
- [ ] Unit / Integration / E2E pass
- [ ] New tests for new code · Regression for bug fixes

**Manual:**
- [ ] Local + dev env confirmed
- [ ] Browser / mobile (if UI)

**Performance:**
- [ ] No impact / Improvement / Degradation (reason: …)

#### Screenshots (UI changes only)
**Before / After**

#### Links
- JIRA: [PP-XXXX](https://ggnetwork.atlassian.net/browse/PP-XXXX)
- Docs / Design / Related PR

#### Checklist
- [ ] Self-review done
- [ ] Commits follow `commit-rules` skill
- [ ] No console.logs / debug code
- [ ] No secrets or sensitive data
- [ ] Docs updated · package-lock if deps changed · CHANGELOG for breaking
```

## 필수 체크 (다른 skill/rule에 위임)

| Check | Reference |
|-------|-----------|
| 코드 임계값 (file/function/complexity) | `coding-standards` skill → `references/code-thresholds.md` |
| 보안 (시크릿, 인젝션, XSS, authn) | `security-review` skill |
| 테스팅 (커버리지, 회귀, E2E 경로) | `tdd-workflow` skill |
| 빌드 / 타입 / lint 검증 | `verification-loop` skill |
| 커밋 메시지 형식 | `commit-rules` skill |

**PR 사이즈 원칙**: 작업 / 커밋 / PR을 작게 유지한다. 논리 단위로 분할한다. 각 커밋은 독립적으로 빌드 및 테스트 가능해야 한다.

## PR 전 체크리스트

1. **코드 품질**: `verification-loop` 실행 (lint, type-check, tests).
2. **브랜치 확인**: feature 브랜치 확인.
3. **업데이트**: 타깃에 rebase (`upstream/develop` 또는 `origin/develop`).
4. **커밋 정리**: 노이즈 squash, `commit-rules` 따름.
5. **충돌**: 해결.
6. **테스트**: 모두 통과.
7. **문서**: 동작 또는 API가 변경되면 업데이트.
8. **빌드**: 성공.
9. **보안**: 시크릿/PII/디버그 코드/console.log 없음.

## 리뷰 가이드라인

**리뷰어:**
- [ ] 기능성 — 요구사항 충족
- [ ] 코드 품질 — 가독성, 유지보수성
- [ ] 설계 — 적절한 아키텍처/패턴
- [ ] 보안 — 취약점 없음 (깊이는 `security-review` skill에 위임)
- [ ] 성능 — 부정적 영향 없음
- [ ] 테스팅 — 커버리지 적절
- [ ] 문서 — 필요한 곳에 업데이트

**작성자:**
- [ ] PR 열기 전 셀프 리뷰
- [ ] 컨텍스트 제공 (무엇만이 아닌 왜)
- [ ] 24h 내에 피드백 응답
- [ ] 요청된 변경 즉시 적용
- [ ] CI/CD 통과

## gh CLI 명령어

```bash
# View
gh pr list
gh pr view <PR-number>
gh pr checkout <PR-number>

# Update
gh pr edit <PR-number> --title "..." --body "..."
gh pr ready <PR-number>
gh pr merge <PR-number> --squash
```

## PR 언어 가이드라인

- 기술 용어는 영어 (API, database, migration, refactoring 등)
- 언어 지정이 없으면 기본은 영어
- 예: "Add caching logic to improve API response speed"
