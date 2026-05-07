---
name: PR Creation Guidelines
description: PR title format, description template, and checklist rules
keywords: [PR, pull request, review, ticket, testing, checklist, merge]
---

## PR 작성 가이드라인

### PR 제목 형식

```
[TICKET-ID] <One-line Summary>
```

예시:
- `[PP-XXXX] Add user authentication system`
- `[PP-XXXX] Fix payment module bug`

### PR 설명 템플릿

```markdown
#### Issue Type
- [ ] Feature addition (feat)
- [ ] Feature removal (feat)
- [ ] Bug fix (fix)
- [ ] Refactoring (refactor)
- [ ] Performance improvement (perf)
- [ ] Dependencies, environment variables, configuration file updates (chore)
- [ ] Styling (style)
- [ ] Documentation (docs)
- [ ] Test code (test)

#### Priority
> Mark issue priority based on current JIRA `Priority` criteria
- [ ] Blocker
- [ ] Urgent
- [ ] Critical
- [ ] Major
- [ ] Trivial

#### Background
> Summarize what work is included in this PR and why this work was done

#### Changes
> List major modifications. Add additional comments for parts that might be difficult for reviewers to understand

**API Changes:**
- [ ] No Breaking Changes
- [ ] Breaking Changes (affects backward compatibility)

**Database Changes:**
- [ ] No schema changes
- [ ] Schema changes (migration required)
- [ ] Data migration required

**Major Changed Files:**
- `path/filename.ext` - Summary of changes

#### Testing
> Testing performed before PR submission. List test cases briefly

**Automated Testing:**
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] New tests added (for new code)
- [ ] Regression tests added (for bug fixes)

**Manual Testing:**
- [ ] Normal operation confirmed in local environment
- [ ] Normal operation confirmed in development environment
- [ ] Browser compatibility confirmed (if applicable)
- [ ] Mobile responsiveness confirmed (if applicable)

**Performance Testing:**
- [ ] No performance impact
- [ ] Performance improvement confirmed
- [ ] Performance degradation (reason: )

#### Screenshots
> Attach Before/After screenshots for UI changes

**Before:**
<!-- Screenshot before changes -->

**After:**
<!-- Screenshot after changes -->

#### Links
> Add links to related documents, work tickets, design guide documents
- [ ] JIRA Ticket: [PP-XXXX](https://ggnetwork.atlassian.net/browse/PP-XXXX)
- [ ] Related Documentation: [Document Name](link)
- [ ] Design Guide: [Figma](link)
- [ ] Related PR: #number

#### Checklist
> Required items to check before creating PR
- [ ] Self-review completed
- [ ] Commit messages follow conventions
- [ ] Code conventions followed
- [ ] Unnecessary console logs/comments removed
- [ ] No secrets or sensitive information included
- [ ] Documentation updated (if necessary)
- [ ] package-lock.json included when dependencies updated
- [ ] CHANGELOG updated for Breaking Changes
```

### 기본 설정
**필수 점검 항목:**

**Code Quality** (Soft target / Hard block — `code-thresholds.md` 참조):
- 파일 크기: ≤ 300 LOC soft, ≤ 500 LOC hard
- 함수 크기: ≤ 50 LOC soft, ≤ 80 LOC hard
- Parameters: ≤ 5 soft, ≤ 7 hard
- Cyclomatic complexity: ≤ 10 soft, ≤ 15 hard
- Nesting depth: ≤ 4 soft, ≤ 6 hard
- Soft: 리뷰에서 논의. Hard: 머지 전에 리팩토링하거나 예외를 문서화한다

**보안 점검:**
- 절대 금지: 시크릿 포함 (비밀번호/API 키/토큰)
- 절대 금지: 민감 데이터 포함 (PII/카드 정보/SSN)
- 절대 금지: SQL injection, XSS, CSRF 취약점 생성
- 항상: 모든 입력 검증, 정규화, 인코딩
- 항상: 파라미터화된 쿼리 사용
- 항상: 인증/인가 적용

**테스트 요구사항:**
- 신규 코드 → 신규 테스트 필수
- 버그 수정 → 회귀 테스트 필수
- 테스트는 먼저 실패하도록 작성한 뒤 수정한다
- E2E 테스트: 성공 경로 ≥1개와 실패 경로 ≥1개 각각 작성

**PR 크기 원칙:**
- 작업, 커밋, PR을 작게 유지한다
- 논리 단위로 분리한다
- 독립적으로 빌드/테스트 가능해야 한다

### PR 작성 전 체크리스트

**1. 코드 품질 점검**
```bash
# Lint check
npm run lint
# or
yarn lint

# Type check
npm run type-check
# or
yarn type-check

# Run tests
npm test
# or
yarn test
```

**2. 필수 점검 항목**
- [ ] **브랜치 확인**: 현재 브랜치가 feature 브랜치인지 확인
- [ ] **업데이트**: 타깃 브랜치(upstream/develop 또는 origin/develop)의 최신 변경 반영
- [ ] **커밋 정리**: 불필요한 커밋을 squash로 정리
- [ ] **충돌 해결**: 머지 충돌이 없는지 확인
- [ ] **테스트 실행**: 모든 테스트가 통과하는지 확인
- [ ] **문서 업데이트**: 필요한 경우 관련 문서 수정
- [ ] **빌드 점검**: 빌드가 성공하는지 확인

**3. 보안 점검**
- [ ] 시크릿, API 키, 토큰, 민감 정보가 포함되지 않음
- [ ] 개발 디버그 코드 제거
- [ ] console 로그 정리

### 리뷰 가이드라인

**리뷰어 관점:**
- [ ] **Functionality**: 요구사항을 정확히 구현했는가?
- [ ] **Code Quality**: 가독성과 유지보수성이 있는가?
- [ ] **Design**: 적절한 아키텍처와 패턴을 사용했는가?
- [ ] **Security**: 보안 취약점은 없는가?
- [ ] **Performance**: 부정적인 성능 영향이 있는가?
- [ ] **Testing**: 적절한 테스트 커버리지가 있는가?
- [ ] **Documentation**: 필요한 문서가 제공되었는가?

**작성자 관점:**
- [ ] **Self Review**: PR 생성 전에 셀프 리뷰 완료
- [ ] **컨텍스트 제공**: 변경 사유와 의도를 명확히 설명
- [ ] **리뷰 응답**: 24시간 내에 피드백에 응답
- [ ] **변경 적용**: 요청된 수정 사항을 신속히 반영
- [ ] **CI/CD**: 모든 자동 점검 통과

### 자주 사용하는 명령어

**PR 상태 확인:**
```bash
# List PRs
gh pr list

# View specific PR details
gh pr view <PR-number>

# Checkout PR (for review)
gh pr checkout <PR-number>
```

**PR 업데이트:**
```bash
# Edit PR title/description
gh pr edit <PR-number> --title "New Title" --body "New Description"

# Change Draft PR to Ready
gh pr ready <PR-number>

# Merge PR
gh pr merge <PR-number> --squash
```

### PR 언어 가이드라인

**기술 용어는 영어로, 설명은 요청된 언어로, 별도 지정이 없으면 영어로 기본 작성**
- 기술 용어: 영어 유지 (API, database, migration, refactoring 등)
- 예시: "Add caching logic to improve API response speed"
