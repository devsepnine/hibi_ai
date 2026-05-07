---
type: release
status: draft | published
version: <major.minor.patch>
release_date: YYYY-MM-DD
changelog_link: <https://github.com/org/repo/releases/tag/v...>
created: YYYY-MM-DD
tags: [type/release, project/<slug>, topic/<area>]
project: <slug>
related: []
---

# v<version> Release Notes

> [!info] Release summary
> 한 단락 피치: 무엇이 달라졌는지, 누가 신경 써야 하는지, 업그레이드가
> 필수인지. 2~3문장 — 사용자는 훑어 읽는다.

## Highlights

- 불릿 1 — 줄당 사용자에게 보이는 변경 하나, 커밋 덤프가 아님.
- 불릿 2 — 상세 섹션이나 ADR로 링크.

## Breaking changes

> [!warning] Breaking
> 소스 호환성을 깨는 모든 변경을 나열한다. **before → after**와
> 마이그레이션 경로를 설명한다. 없으면 이 섹션을 삭제한다.

- `<API/flag name>`: before … → after …. 마이그레이션: …

## Bug fixes

- 짧은 설명 — [GitHub PR](https://github.com/org/repo/pull/123).
  맥락: 있다면 [[<ADR or debug note>]].

## Improvements

- 설명. 변경 뒤에 설계 결정이 있을 때 [[ADR-NNNN]]로 링크.

## Internal / refactor

- 사용자에게 영향이 없지만 미래 메인테이너가 신경 쓸 변경. 짧게
  유지하고 상세는 ADR이나 개발 로그로 미룬다.

## Upgrade guide

1. 이전 마이너 버전에서 단계별로.
2. 데이터 마이그레이션, 설정 파일 이름 변경, CLI 플래그 변경을 명시한다.
3. 사용자가 구버전에 머무를 수 있다면, 그래도 옮길 만한 이유를 적는다.

## Related

- [[<prior release note>]]
- [[ADR-NNNN …]] 이 버전에 들어간 결정들
- [[Retro YYYY-WNN]] 이 릴리스가 스프린트를 마무리했다면

## External references

- [Release on GitHub](<changelog_link>)
- 마이그레이션 문서 / 블로그 포스트
