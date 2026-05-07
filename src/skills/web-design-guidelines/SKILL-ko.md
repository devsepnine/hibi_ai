---
name: web-design-guidelines
description: Review UI code for Web Interface Guidelines compliance. Use when asked to "review my UI", "check accessibility", "audit design", "review UX", or "check my site against best practices". UI 리뷰, 웹 디자인 검토, 접근성 점검, UX 리뷰, 디자인 감사.
keywords: [ui, ux, 디자인, design, accessibility, 접근성, web-design]
metadata:
  author: vercel
  version: "1.0.0"
  argument-hint: <file-or-pattern>
---

# Web Interface Guidelines

Web Interface Guidelines 준수를 위해 파일을 리뷰한다.

## 동작 방식

1. 아래 source URL에서 최신 가이드라인을 fetch
2. 지정된 파일을 읽음 (없다면 사용자에게 파일/패턴 요청)
3. fetch한 가이드라인의 모든 규칙에 대해 점검
4. 간결한 `file:line` 포맷으로 결과 출력

## 가이드라인 출처

매 리뷰 전 새로운 가이드라인을 fetch:

```
https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md
```

WebFetch로 최신 규칙을 가져온다. fetch한 콘텐츠에는 모든 규칙과 출력 포맷 지시가 들어 있다.

## 사용법

사용자가 파일 또는 패턴 인자를 제공하면:
1. 위 source URL에서 가이드라인 fetch
2. 지정된 파일 읽기
3. fetch한 가이드라인의 모든 규칙 적용
4. 가이드라인이 명시한 포맷으로 결과 출력

파일이 지정되지 않으면 사용자에게 어떤 파일을 리뷰할지 묻는다.
