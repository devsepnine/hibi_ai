---
title: Optimize SVG Precision
impact: LOW
impactDescription: reduces file size
tags: rendering, svg, optimization, svgo
---

## SVG 좌표 정밀도를 최적화한다

SVG 좌표의 정밀도를 줄여 파일 크기를 감소시킨다. 최적의 정밀도는 viewBox 크기에 따라 다르지만, 일반적으로 정밀도를 낮추는 것을 검토할 만하다.

**잘못된 예 (과도한 정밀도):**

```svg
<path d="M 10.293847 20.847362 L 30.938472 40.192837" />
```

**올바른 예 (소수점 1자리):**

```svg
<path d="M 10.3 20.8 L 30.9 40.2" />
```

**SVGO로 자동화:**

```bash
npx svgo --precision=1 --multipass icon.svg
```
