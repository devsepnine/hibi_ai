---
name: architecture-rules
description: "Write docs/ARCHITECTURE.md for an existing codebase: rules mined from its imports and layout, gray areas settled by Q&A, lint drafts. Use when defining or refreshing architecture rules. 아키텍처 규칙, 아키텍처 문서화, 레이어 규칙."
---

# Architecture Rules

`docs/ARCHITECTURE.md`를 쓴다. AI가 이 프로젝트에 파일, 모듈, import를 추가하기 전에 따라야 할 규칙이다. 대부분은 이미 코드에 있다. 거의 모든 파일이 따르는 패턴으로 존재한다. 나머지는 아무도 적어 두지 않은 결정이고, 코드만으로는 규칙과 우연을 구분할 수 없다.

그래서 방법은 두 갈래다. 코드가 증명하는 규칙을 증거와 함께 모두 캐낸다. 그다음 회색 지대, 즉 코드가 스스로 모순되거나 의도를 드러내지 못하는 곳만 사용자에게 묻는다.

## 왜 이런 형태인가

기억으로 쓴 규칙 문서는 누군가 의도한 아키텍처를 기술한다. 코드만으로 쓴 문서는 모든 우연을 규칙으로 굳힌다. 어느 쪽도 다음 에이전트가 패턴을 깨는 파일을 만났을 때 무엇을 해야 하는지 알려 주지 못한다. 이 문서는 규칙마다 얼마나 강하게 성립하는지, 증거, 알려진 예외를 기록한다. 그래서 새 코드의 위반과 이미 수용된 위반을 구분할 수 있다.

이것은 `dependency-design`이 아니다: 그 skill은 결합이 일반적으로 건강한지 판단한다. `/update-codemaps`의 codemap도 아니다: codemap은 무엇이 허용되는지 말하지 않고 구조만 기술한다. `feature-map`도 아니다: 그 skill은 제보를 코드에 대응시킨다. 서로 링크하되 하나를 다른 하나에 복사하지 않는다.

## 워크플로

### 1. Preflight: 새 문서인가 갱신인가

```bash
test -f docs/ARCHITECTURE.md && grep -m1 -E '^(Verified at|검증 시점)' docs/ARCHITECTURE.md
git rev-parse --short HEAD
ls .dependency-cruiser.* .eslintrc* eslint.config.* importlinter* .importlinter go-arch-lint.* 2>/dev/null
```

- **문서가 있다**: 갱신 모드다. [Update mode](#update-mode)를 본다.
- **문서가 없다**: 전체 작성이고 2단계부터 7단계까지 진행한다.
- 매니페스트에서 스택을 파악한다: `package.json`과 workspaces, `Cargo.toml` workspace, `go.mod`, `pyproject.toml`. 이미 설정된 경계 lint가 있으면 기록한다. 그 규칙은 이미 결정된 규칙이라 질문이 필요 없다.
- 문서는 저장소의 다른 문서가 쓰는 언어로 쓴다. 템플릿의 제목과 필드 라벨, 규칙 ID, `MUST`, `SHOULD`는 영어로 둔다. 아래 검증 블록이 그 문자열을 grep하기 때문이다.
- 모든 경로는 모노레포 패키지 안에서도 저장소 루트 기준 상대 경로로 쓴다.

### 2. 단위를 찾는다

단위란 다른 부분이 통째로 의존하는 코드의 일부다: 패키지, crate, 최상위 모듈, 또는 `routes`, `services`, `repositories` 같은 레이어. 폴더 이름만 보지 말고 매니페스트와 import 그래프에서 가져온다. 아무도 단위로 import하지 않는 폴더는 단위가 아니다.

단위마다 경로와 책임 한 문장을 기록한다. 책임은 그 파일들이 실제로 하는 일에서 쓴다.

### 3. 후보 규칙을 측정한다

모든 후보 규칙은 횟수가 붙은 패턴이다. 다시 실행할 수 있는 명령으로 측정하고 그 명령을 보관한다. 갱신 모드가 그것을 다시 실행하기 때문이다.

| 종류 | 측정할 것 | 방법 |
|---|---|---|
| 의존 방향 | 어느 단위가 어느 단위를 import하는지, 그리고 순환 | 설치되어 있으면 스택의 그래프 도구, 예를 들어 `madge`, `dependency-cruiser`, `cargo modules`, `go list -deps`. 없으면 import 문을 grep |
| 금지된 간선 | 몇 개 파일에만 존재하는 import | 같은 그래프에서 횟수가 적은 간선만 거른다 |
| 배치 | 종류별 파일의 위치: routes, components, tests, migrations | 종류별 glob 횟수 |
| 이름 규칙 | 단위별 파일과 심볼의 이름 규칙 | glob과 grep 횟수 |
| 횡단 접근 | env와 config, 데이터베이스, 로깅, HTTP 클라이언트에 어디서 접근하는지 | 클라이언트나 접근자를 grep하고 단위별로 묶는다 |

후보마다 기록한다: 한 문장의 규칙, `47 of 50 files` 같은 횟수, 명령, 그리고 모든 반례의 경로.

### 4. 후보를 분류한다

- **Proven**: 모든 사례가 따르거나, 반례가 생성된 코드, vendored 코드, 테스트뿐이다. 묻지 않고 규칙으로 초안을 쓴다.
- **Gray**: 코드가 스스로 모순된다. 서비스 3개가 route 모듈을 import하거나, 순환이 있거나, 폴더 하나가 두 가지 일을 하거나, 비슷한 규모의 이름 규칙이 두 가지 있는 경우다. 어느 쪽이 규칙인지는 사용자만 말할 수 있다.
- **코드가 보여 주지 못하는 의도**: 예정된 마이그레이션, 존재해야 하지만 비어 있는 레이어, 아직 쓰이는 deprecated 모듈. 묻는다.

이미 있는 경계 lint는 그 규칙을 proven 규칙으로 만든다.

### 5. Q&A로 회색 지대를 정리한다

`AskUserQuestion`으로 한 번에 최대 네 개씩 묻는다. 질문마다 증거를 싣는다: 횟수, 반례 경로, 각 답이 그 파일들에 의미하는 바. 권장 답을 이유와 함께 맨 앞에 둔다. 답은 보통 다음 중 하나다:

- **규칙으로 만든다**: 반례는 고쳐야 할 알려진 위반이 된다.
- **예외를 허용한다**: 이름 붙은 경로로 범위를 좁히고 이유를 적는다.
- **규칙이 아니다**: 후보를 버린다.
- **마이그레이션 중**: 규칙은 목표 상태이고, 이름 붙은 조건이 충족될 때까지 옛 패턴을 허용한다.

코드가 이미 답하는 것은 묻지 않는다. 모든 답을 날짜와 함께 Decisions 섹션에 기록한다. 다음 사람이 같은 질문을 하기 때문이다.

### 6. 문서를 쓴다

`assets/architecture-template.md`를 쓴다. 규칙마다:

- **ID**: `arch-` 접두사가 붙은 안정적인 kebab-case, 예를 들어 `arch-routes-no-repository-import`.
- **Level**: 위반이 결함이면 `MUST`, 위반에 리뷰에서 이유가 필요하면 `SHOULD`.
- **Rule**: 에이전트가 diff와 대조해 확인할 수 있는 한 문장.
- **Why**: 사용자의 답이나 증거. 지어낸 이유는 안 된다.
- **Evidence**: 횟수와 그것을 측정한 명령.
- **Exceptions**: 이유가 붙은 경로, 또는 `none`.
- **Enforced by**: 그것을 검사하는 lint 규칙, 초안을 제안했다면 `draft: <tool>`, 리뷰어만 검사할 수 있다면 `review`.

알려진 위반은 별도 섹션에 규칙 ID와 결정, 즉 fix 또는 accepted와 함께 둔다. 문서는 약 300줄 아래로 유지한다. 설명이 아니라 경로를 쓰고, 줄 번호는 흔들리므로 쓰지 않는다.

### 7. 강제 수단을 초안으로 쓰고 연결한다

도구로 검사할 수 있는 모든 `MUST`, 예를 들어 import 방향, 금지된 import 경로, glob 기준 배치에 대해, 프로젝트가 이미 쓰는 도구로 설정 초안을 쓴다. 없으면 스택의 일반적인 도구를 쓴다: TypeScript는 `dependency-cruiser` 또는 `eslint-plugin-boundaries`, Python은 `import-linter`, Go는 `go-arch-lint` 또는 `depguard`, Rust는 모듈 가시성이나 workspace crate 경계.

초안은 답변에 보여 준다. 사용자 동의 없이 패키지를 설치하거나, 설정 파일을 추가하거나, 빌드를 바꾸지 않는다. 도구가 이미 설치되어 있으면 초안을 check 모드로 한 번 실행하고 걸린 것을 보고한다. 6단계의 알려진 위반이 정확히 그 목록이어야 한다.

그다음 에이전트가 코드를 쓰기 전에 규칙을 읽게 한다. 프로젝트 루트 `CLAUDE.md`에 한 줄 포인터를 추가하고, 파일이 없으면 그 한 줄로 새로 만든다. `AGENTS.md`가 있으면 같은 줄을 거기에도 넣는다. 그 한 줄만 넣고 파일의 나머지는 그대로 둔다. 이 쓰기는 스킬 산출물의 일부다. 줄이 이미 있으면 문서 위치가 바뀐 경우에만 고친다. 변경 사항은 응답에서 알린다:

```markdown
- Before adding a module, a file in a new place, or an import across units, check `docs/ARCHITECTURE.md`.
```

문서를 쓰는 것은 콘텐츠 변경이므로 `CLAUDE.md`의 작업 후 리뷰 게이트가 적용된다.

### 넘기기 전에 검증한다

```bash
DOC=docs/ARCHITECTURE.md
# every repo-relative path exists; globs and <placeholders> are skipped
grep -ohE '`[^`<> /][^`<> ]*/[^`<> ]*`' $DOC | tr -d '`' | grep -v '[*?{[]' | sort -u \
  | while read -r p; do test -e "$p" || echo "MISSING $p"; done
# rule IDs are unique as headings; the same ID may recur in Known violations
grep -oE '^### `arch-[a-z0-9-]+`' $DOC | sort | uniq -d | sed 's/^/DUPLICATE /'
# every rule carries every field
rules=$(grep -c '^### `arch-' $DOC)
for f in Level Rule Why Evidence Exceptions 'Enforced by'; do
  n=$(grep -c "^- \*\*$f\*\*" $DOC); [ "$n" = "$rules" ] || echo "FIELD $f: $n of $rules rules"
done
```

문서에 문제가 없으면 이 블록은 아무것도 출력하지 않는다. 그다음 각 증거 명령을 다시 실행해 문서의 횟수가 여전히 맞는지 확인한다.

## Update mode

1. 기록된 SHA부터 diff를 본다: `git diff --name-status -M <sha>..HEAD`. SHA에 도달할 수 없으면 그렇게 밝히고 모든 규칙을 다시 측정한다.
2. 이름이 바뀐 파일, 즉 `R`로 표시된 파일은 Units, Exceptions, Known violations의 경로를 새 경로로 고친다. 삭제된 파일, 즉 `D`로 표시된 파일의 예외나 위반은 뺀다.
3. 각 규칙의 증거 명령을 다시 실행한다. 횟수가 움직이지 않은 규칙은 쓰인 그대로 둔다. 대부분의 규칙이 의심스러우면 먼저 문서 전체에 검증 블록을 돌리고, Decision은 그것이 이름을 든 파일이 diff에 나타날 때만 다시 연다.
4. 새 반례는 수정이 아니라 질문이다: 고치거나, 예외로 허용하거나, 규칙을 완화한다. 5단계 형식으로 묻는다.
5. 새 단위나 사라진 단위는 그 단위의 규칙에 한해 2단계부터 5단계까지 거친다.
6. 결정된 규칙을 조용히 다시 쓰지 않는다. 추가, 변경, 삭제된 규칙을 보고한 뒤 헤더의 SHA를 HEAD로 옮긴다.

## 관련

| 필요 | 위치 |
|---|---|
| 결합이 일반적으로 건강한지 | `dependency-design` |
| 새 작업의 설계 결정 | `architect` agent |
| 코드가 지금 어떻게 구성되어 있는지 | `/update-codemaps` |
| 모호한 제보가 가리키는 코드 | `feature-map` |
