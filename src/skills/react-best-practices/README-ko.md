# React Best Practices

에이전트와 LLM에 최적화된 React 베스트 프랙티스를 작성·관리하기 위한 구조화된 저장소이다.

## 구조

- `rules/` - 개별 규칙 파일 (규칙당 한 파일)
  - `_sections.md` - 섹션 메타데이터 (제목, 영향도, 설명)
  - `_template.md` - 새 규칙 작성용 템플릿
  - `area-description.md` - 개별 규칙 파일
- `src/` - 빌드 스크립트 및 유틸리티
- `metadata.json` - 문서 메타데이터 (버전, 조직, 요약)
- __`AGENTS.md`__ - 컴파일된 출력물 (생성됨)
- __`test-cases.json`__ - LLM 평가용 테스트 케이스 (생성됨)

## 시작하기

1. 의존성 설치한다.
   ```bash
   pnpm install
   ```

2. rules로부터 AGENTS.md를 빌드한다.
   ```bash
   pnpm build
   ```

3. 규칙 파일을 검증한다.
   ```bash
   pnpm validate
   ```

4. 테스트 케이스를 추출한다.
   ```bash
   pnpm extract-tests
   ```

## 새 규칙 만들기

1. `rules/_template.md`를 `rules/area-description.md`로 복사한다.
2. 적절한 영역 prefix를 선택한다.
   - `async-` - Eliminating Waterfalls (Section 1)
   - `bundle-` - Bundle Size Optimization (Section 2)
   - `server-` - Server-Side Performance (Section 3)
   - `client-` - Client-Side Data Fetching (Section 4)
   - `rerender-` - Re-render Optimization (Section 5)
   - `rendering-` - Rendering Performance (Section 6)
   - `js-` - JavaScript Performance (Section 7)
   - `advanced-` - Advanced Patterns (Section 8)
3. frontmatter와 본문을 채운다.
4. 명확한 예시와 설명을 포함한다.
5. `pnpm build`를 실행해 AGENTS.md와 test-cases.json을 재생성한다.

## 규칙 파일 구조

각 규칙 파일은 다음 구조를 따라야 한다.

```markdown
---
title: Rule Title Here
impact: MEDIUM
impactDescription: Optional description
tags: tag1, tag2, tag3
---

## Rule Title Here

Brief explanation of the rule and why it matters.

**Incorrect (description of what's wrong):**

```typescript
// Bad code example
```

**Correct (description of what's right):**

```typescript
// Good code example
```

Optional explanatory text after examples.

Reference: [Link](https://example.com)

## 파일 명명 규칙

- `_`로 시작하는 파일은 특수 파일이다 (빌드에서 제외).
- 규칙 파일: `area-description.md` (예: `async-parallel.md`)
- 섹션은 파일명 prefix로 자동 추론된다.
- 규칙은 각 섹션 내에서 제목 기준 알파벳순으로 정렬된다.
- ID(예: 1.1, 1.2)는 빌드 시 자동 생성된다.

## 영향도 레벨

- `CRITICAL` - 최상위 우선순위, 큰 성능 향상
- `HIGH` - 상당한 성능 개선
- `MEDIUM-HIGH` - 중간-높은 향상
- `MEDIUM` - 중간 정도의 성능 개선
- `LOW-MEDIUM` - 낮음-중간 향상
- `LOW` - 점진적 개선

## 스크립트

- `pnpm build` - rules를 AGENTS.md로 컴파일한다
- `pnpm validate` - 모든 규칙 파일을 검증한다
- `pnpm extract-tests` - LLM 평가용 테스트 케이스를 추출한다
- `pnpm dev` - 빌드 후 검증한다

## 기여하기

규칙을 추가하거나 수정할 때:

1. 섹션에 맞는 파일명 prefix를 사용한다.
2. `_template.md` 구조를 따른다.
3. 명확한 bad/good 예시와 설명을 포함한다.
4. 적절한 태그를 추가한다.
5. `pnpm build`로 AGENTS.md와 test-cases.json을 재생성한다.
6. 규칙은 자동으로 제목 기준 정렬되므로 번호를 직접 관리할 필요가 없다.

## 감사의 글

원작자: [@shuding](https://x.com/shuding) at [Vercel](https://vercel.com).
