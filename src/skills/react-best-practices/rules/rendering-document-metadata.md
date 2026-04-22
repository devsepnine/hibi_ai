---
title: Use React 19 Native Document Metadata, Not Manual Head Mutation
impact: MEDIUM
impactDescription: removes side-effect hooks; metadata streams with SSR and hoists automatically
tags: react19, metadata, seo, ssr, streaming
---

## Use React 19 Native Document Metadata, Not Manual Head Mutation

React 19 automatically hoists `<title>`, `<link>`, and `<meta>` tags from
anywhere in the tree into `<head>`. This works across CSR, streaming SSR,
and Server Components. Remove hand-rolled `useEffect(() => { document.title = ... })`
and external helpers (react-helmet, next/head in app router, etc.) for
per-page metadata — the native hoist is smaller, streams properly, and
stays deduped across renders.

### Incorrect — imperative head mutation

```tsx
function BlogPost({ post }: { post: Post }) {
  useEffect(() => {
    document.title = post.title
    // separate effect for meta tags...
  }, [post.title])

  return <article>{post.body}</article>
}
```

Problems: runs only on the client (no SSR), fires after paint, needs
cleanup to restore, doesn't dedupe if two components set the same tag.

### Correct — metadata elements inline with the component

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

React hoists these into `<head>` during render. In streaming SSR they flush
as soon as they're encountered — no blocking-on-whole-tree required.

### Guidelines

- Place metadata where the data lives; don't thread it up to a layout.
- For Next.js App Router, prefer the built-in `generateMetadata`/`metadata`
  export for statically-known metadata; use inline tags only for
  per-render dynamic values the route config can't see.
- `<link rel="stylesheet">` is also hoisted and deduped — same pattern.
- Don't mix with `react-helmet` / `react-helmet-async`; they fight the
  native hoist and can double-render tags.

Reference: [React 19 — Document Metadata](https://react.dev/blog/2024/12/05/react-19#support-for-metadata-tags)
