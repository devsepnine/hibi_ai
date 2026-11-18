---
title: Cache Repeated Function Calls
impact: MEDIUM
impactDescription: avoid redundant computation
tags: javascript, cache, memoization, performance
---

## Cache Repeated Function Calls

렌더링 중 동일한 입력으로 같은 함수가 반복 호출된다면, 모듈 레벨 Map을 사용해 결과를 캐시한다.

**Incorrect, redundant computation:**

```typescript
function ProjectList({ projects }: { projects: Project[] }) {
  return (
    <div>
      {projects.map(project => {
        // slugify() called 100+ times for same project names
        const slug = slugify(project.name)
        
        return <ProjectCard key={project.id} slug={slug} />
      })}
    </div>
  )
}
```

**Correct, cached results:**

```typescript
// Module-level cache
const slugifyCache = new Map<string, string>()

function cachedSlugify(text: string): string {
  if (slugifyCache.has(text)) {
    return slugifyCache.get(text)!
  }
  const result = slugify(text)
  slugifyCache.set(text, result)
  return result
}

function ProjectList({ projects }: { projects: Project[] }) {
  return (
    <div>
      {projects.map(project => {
        // Computed only once per unique project name
        const slug = cachedSlugify(project.name)
        
        return <ProjectCard key={project.id} slug={slug} />
      })}
    </div>
  )
}
```

로그인 표시는 프로젝트의 세션 상태를 구독해 갱신한다. 사용자가 바뀌거나 세션이 만료되면 같은 인자로 호출해도 결과가 달라진다. 모듈 캐시는 입력이 결과를 결정하고 캐시 수명이 데이터의 수명과 맞을 때 사용한다.

hook이 아닌 Map을 사용하므로 React 컴포넌트뿐 아니라 유틸리티, 이벤트 핸들러 등 어디서나 동작한다.

Reference: [How we made the Vercel Dashboard twice as fast](https://vercel.com/blog/how-we-made-the-vercel-dashboard-twice-as-fast)
