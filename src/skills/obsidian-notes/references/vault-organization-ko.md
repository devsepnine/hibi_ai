# Vault Organization

일관된 폴더 레이아웃, MOC 전략, capture-to-evergreen 흐름은 vault가 성장하면서 navigable한 상태로 유지되도록 한다. 스키마는 의도적으로 가볍다 — 대부분의 필터링은 frontmatter + tag에 미루고 폴더는 물리적 colocation에만 사용한다.

## Suggested folder layout

```
<vault>/
├── Daily/              # YYYY-MM-DD.md (one per day)
├── Weekly/             # YYYY-Wnn.md
├── Meetings/           # YYYY-MM-DD <slug>.md
├── Release Notes/      # v<semver>.md
├── ADR/                # ADR-NNNN-<slug>.md
├── Retros/             # YYYY-Wnn.md
├── Debug/              # YYYY-MM-DD <slug>.md
├── Library/            # Books/, Zustand/, React/, ...
├── MOC/                # project and topic maps
├── Fleeting/           # YYYY-MM-DD-HHmm.md (captured ideas)
├── Attachments/        # images, PDFs, canvases
└── + Inbox.md          # default landing page for quick captures
```

여기 폴더는 **노트 lifecycle별로** 그룹화한다, 프로젝트별이 아니다. 단일 프로젝트의 작업은 `Daily/`, `Meetings/`, `ADR/` 등에 퍼져있다 — 연결은 `project/<slug>` 태그와 프로젝트의 MOC 노트로 만들어진다, 파일을 물리적으로 colocating하는 것이 아니라.

새 vault를 `Daily/`, `MOC/`, `Fleeting/`만으로 시작한다. 첫 항목을 얻으면 나머지를 추가한다.

## PARA-ish adaptation

PARA (Projects / Areas / Resources / Archive)를 선호한다면, 폴더가 아닌 MOC를 통해 레이어한다:

- **project** MOC는 `MOC/<Project Name>.md`에 산다.
- **area** MOC (지속적인 책임, 끝나는 날짜 없음)는 frontmatter에 `scope: area`와 함께 `MOC/<Area>.md`에 산다.
- **Resources**는 evergreen `Library/` 노트이다.
- **Archive**는 MOC의 `status: archived`일 뿐이다; 구성 노트는 이동하지 않는다 — dataview가 frontmatter로 필터링한다.

이는 노트가 두 카테고리에 맞는 일반적인 PARA 통증을 피한다: 노트는 그대로 있고, MOC는 어느 시점에서든 어느 "bucket"에 속하는지 큐레이션한다.

## Zettelkasten lite

개인 / 연구 vault의 경우, learning-note template (atomic, 하나의 아이디어, 하나의 link out, 하나의 link in)이 Zettelkasten 단위이다. 이를 레이어한다:

- **Fleeting** → `Fleeting/` (raw capture)
- **Literature** → `Library/Books/...` (책 하나 = 노트 하나; key highlights를 block ref로)
- **Permanent / evergreen** → `Library/<Topic>/...` (atomic concepts)

업그레이드 경로 (fleeting → evergreen)는 명시적이다 — 아래 "Fleeting → Evergreen" 섹션 참조.

## MOC strategy

**Map of Content**는 다른 노트를 큐레이션하는 일을 하는 노트이다. 세 가지 flavor:

### Project MOC — `MOC/<Project>.md`

프로젝트의 모든 것에 대한 정문: status, 현재 focus, 핵심 ADR, 최신 릴리스, 열린 디버그 로그, 현재 주의 daily note 대시보드. dataview의 헤비 사용.

Template: [`project-moc`](../assets/project-moc.md).

### Topic / subject MOC — `MOC/<Topic>.md`

프로젝트 전반에 큐레이션. 예: `MOC/Performance.md`는 `topic/perf` 태그된 모든 ADR, 학습 노트, 디버그 로그를 link한다.

프로젝트 MOC와 구별하기 위해 frontmatter에 `scope: topic`을 사용한다.

### Area MOC — `MOC/<Area>.md`

끝나는 날짜 없는 지속적인 책임 (예: `MOC/Health.md`, `MOC/Reading List.md`). `scope: area`를 사용한다.

### When NOT to make a MOC

- 약 10개 미만의 관련 노트 → 그냥 태그와 Obsidian의 태그 pane을 사용한다; MOC는 중복일 것이다.
- 단명한 컨텍스트 → 프로젝트 노트나 핀 고정된 주간 리뷰로 충분하다.

모든 것을 MOC하려는 충동을 거부한다. 가장 좋은 MOC는 적극적으로 navigate하는 것이다 — MOC를 절대 클릭하지 않으면, 삭제한다.

## Fleeting → Evergreen upgrade path

Fleeting note는 만들기 싸고 썩게 두기 쉽다. 좋은 것이 졸업하도록 워크플로우에 업그레이드 패턴을 빌드한다.

### Weekly fleeting review (in your weekly-review note)

`review_on <= today`인 fleeting note를 나열하는 dataview 쿼리를 임베드한다:

````markdown
```dataview
TABLE captured_at, source
FROM #type/fleeting
WHERE status = "new" AND review_on <= date(today)
SORT review_on ASC
```
````

각 due 항목에 대해 세 가지 운명 중 하나를 결정한다:

| Decision | Action | Frontmatter change |
|----------|--------|-------------------|
| **Promote** — 아이디어가 다리가 있다 | 새 learning/ADR/MOC 노트를 만들고; 핵심 콘텐츠를 이동; `related: [[original fleeting]]`로 link back | `status: processed`, `promoted_to: [[new note]]` 추가 |
| **Defer** — 더 많은 시간 필요 | `review_on`을 앞으로 push | (다른 변경 없음) |
| **Discard** — 결과가 없었다 | discarded로 표시; 선택적으로 한 줄짜리 이유 추가 | `status: discarded` |

discarded fleeting을 삭제하지 마라 — "didn't work" 기록은 audit 가치가 있고, 노트는 작다.

### What makes an evergreen note (Zettelkasten-flavored)

승급할 때, 다음을 목표로 한다:

- **노트당 하나의 아이디어** — 두 개로 나누고 싶다면, 그래라.
- **검색 query가 될 수 있는 제목** — "Zustand v5 selector reference equality"가 "Zustand stuff"를 이긴다.
- **적어도 하나의 outgoing link** — link out이 없는 evergreen 노트는 orphan이다; 관련 개념을 찾는다.
- **적어도 하나의 incoming link** — MOC나 부모 topic에서 `[[This Note]]` 참조를 추가한다. 그렇지 않으면 아무도 찾지 못한다.

## Attachments & binary assets

노트 폴더 밖에 둔다:

- `Attachments/` — 인라인 이미지의 기본값
- `Attachments/Excalidraw/` — `.excalidraw.md` 파일
- `Attachments/Canvas/` — 선택적; `.canvas`는 종종 임베드하는 노트 옆에 산다

Obsidian 설정 → **Files & Links → Default location for new attachments** → `In subfolder under current folder` + 폴더 이름 `Attachments/`는 일관된 배치를 제공한다.

큰 바이너리 (비디오, > 10 MB PDF)를 vault의 git repo에 절대 commit하지 마라 — 클라우드 스토리지 link를 대신 사용한다.

## Search patterns that rely on this layout

- "이번 주의 작업" → `#type/daily` + `week`로 필터
- "여전히 열린 hibi-ai의 모든 ADR" → `#type/adr AND #project/hibi-ai` + `status = proposed`
- "Reading list" → `#type/book + status = "reading"` (또는 그냥 `MOC/Reading List.md` 열기)
- "기한 지난 fleeting note" → `#type/fleeting + review_on <= today`

레이아웃은 이러한 쿼리를 손쉽게 만들기 위해 존재한다, 노트 자체를 정리하기 위해서가 아니다 — 그것은 태그 + frontmatter가 이미 한다.
