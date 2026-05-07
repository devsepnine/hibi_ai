---
title: Use React 19 Native Document Metadata, Not Manual Head Mutation
impact: MEDIUM
impactDescription: removes side-effect hooks; metadata streams with SSR and hoists automatically
tags: react19, metadata, seo, ssr, streaming
---

## 수동 head 조작 대신 React 19의 네이티브 Document Metadata를 사용한다

React 19는 트리 어디에 두어도 `<title>`, `<link>`, `<meta>` 태그를 자동으로 `<head>`로 호이스트한다. 이는 CSR, streaming SSR, 서버 컴포넌트 모두에서 동작한다. 페이지별 메타데이터 처리를 위해 직접 작성하던 `useEffect(() => { document.title = ... })`나 외부 헬퍼(react-helmet, app router에서의 next/head 등)는 제거한다 — 네이티브 호이스트는 더 작고, 스트리밍에 적합하며, 렌더 간 중복도 자동으로 제거된다.

### 잘못된 예 — 명령형 head 조작

```tsx
function BlogPost({ post }: { post: Post }) {
  useEffect(() => {
    document.title = post.title
    // separate effect for meta tags...
  }, [post.title])

  return <article>{post.body}</article>
}
```

문제점: 클라이언트에서만 실행됨(SSR 없음), paint 이후에 실행됨, 복원을 위한 cleanup 필요, 두 컴포넌트가 같은 태그를 설정해도 중복 제거되지 않음.

### 올바른 예 — 컴포넌트 내부에 metadata 요소를 인라인으로 둔다

```tsx
function BlogPost({ post }: { post: Post }) {
  return (
    <article>
      <title>{post.title}</title>
      <meta name="author" content={post.author} />
      <meta name="description" content={post.excerpt} />
      <link rel="canonical" href={`https://example.com/posts/${post.slug}`} />
      {post.body}
    </article>
  )
}
```

React는 렌더링 중에 이 요소들을 `<head>`로 호이스트한다. streaming SSR에서는 만나는 즉시 flush되며 — 전체 트리를 기다리지 않는다.

### 가이드라인

- 메타데이터는 데이터가 있는 곳에 둔다. layout까지 끌어올리지 않는다.
- Next.js App Router에서는 정적으로 알려진 메타데이터에 대해 빌트인 `generateMetadata`/`metadata` export를 우선 사용한다. 라우트 설정이 알 수 없는 렌더별 동적 값에 대해서만 인라인 태그를 사용한다.
- `<link rel="stylesheet">`도 같은 패턴으로 호이스트되고 중복 제거된다.
- `react-helmet` / `react-helmet-async`와 섞어 쓰지 않는다. 네이티브 호이스트와 충돌해 태그가 두 번 렌더될 수 있다.

참고: [React 19 — Document Metadata](https://react.dev/blog/2024/12/05/react-19#support-for-metadata-tags)
