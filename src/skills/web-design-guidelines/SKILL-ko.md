---
name: web-design-guidelines
description: "Review UI code against Web Interface Guidelines: accessibility, UX, design audit. Use when asked to review UI/UX or check accessibility. UI 리뷰, 웹 디자인 검토, 접근성 점검, UX 리뷰."
metadata:
  author: vercel
  version: "1.0.0"
  argument-hint: <file-or-pattern>
---

# Web Interface Guidelines

Web Interface Guidelines 준수를 위해 파일을 리뷰한다.

## 동작 방식

1. 아래 source URL에서 최신 가이드라인을 fetch
2. 지정된 파일을 읽음. 없다면 사용자에게 파일/패턴 요청
3. fetch한 가이드라인의 모든 규칙에 대해 점검
4. 간결한 `file:line` 포맷으로 결과 출력

## 가이드라인 출처

매 리뷰 전 새로운 가이드라인을 fetch:

```
https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md
```

현재 환경에서 사용할 수 있는 읽기 도구로 가이드라인을 가져와 요청한 UI 검토에 적용한다. 접근에 실패하면 최신 규칙 확인이 미완료임을 밝히고 아래 항목을 검토한다.

## 사용법

사용자가 파일 또는 패턴 인자를 제공하면:
1. 위 source URL에서 가이드라인 fetch
2. 지정된 파일 읽기
3. fetch한 가이드라인의 모든 규칙 적용
4. 가이드라인이 명시한 포맷으로 결과 출력

파일이 지정되지 않으면 사용자에게 어떤 파일을 리뷰할지 묻는다.

## 화면의 작업에 따른 점검

- **정보 위계**. 사용자가 화면에서 완료해야 하는 핵심 작업을 확인하고, 이에 필요한 정보가 잘 드러나는지 살핀다. 서로 비슷하게 강조된 요소 때문에 다음 행동이나 중요한 데이터를 찾기 어려운 부분을 짚는다.
- **데이터 표**. 정렬, 필터, 행별 동작 중 해당하는 기능을 키보드로 실행한다. 데이터가 갱신되거나 대화상자가 닫힌 뒤 포커스 위치를 확인한다. 로딩, 실패, 데이터 없음, 필터 결과 없음 상태에서 사용자가 취할 행동이 다르면 상태별로 점검한다.
- **상태 표시와 명암비**. 영향을 받는 테마와 상태에서 실제 렌더링된 배경과 텍스트의 명암비를 확인한다. 선택, 성공, 실패 상태를 색에 의존하지 않고 화면의 문구나 아이콘 형태로 구분할 수 있는지 살핀다. 화면에서 확인한 근거와 소스 코드 줄을 제시한다. [WCAG 명암비](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html) · [WCAG 색상 사용](https://www.w3.org/WAI/WCAG22/Understanding/use-of-color)
- **Impeccable 자동 검사**. 사용자가 자동 패턴 검사를 요청했고 CLI가 이미 설치되어 있으면 해당 버전의 옵션을 확인한다. 요청 범위에 있는 파일이나 디렉터리 경로를 `review_path`에 넣어 `impeccable detect --json "$review_path"`을 실행한다. 공식 CLI에서 종료 코드 `2`는 발견 사항을, `1`은 검사 실패를 뜻한다. 발견 사항은 검토 후보로 보고하고 실행에 실패하면 검사가 완료되지 않았다고 적는다. [Impeccable detector](https://github.com/pbakaus/impeccable#three-ways-to-run-the-detector)

점검 항목의 근거가 소스 코드뿐이면 확인한 줄을 인용하고 해당 화면의 표시와 동작은 미검증으로 표시한다.
