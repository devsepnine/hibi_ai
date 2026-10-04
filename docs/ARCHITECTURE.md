# hibi-ai Architecture Rules

Verified at `c29d8ce` · `2026-10-04` · Structural view: `docs/INDEX.md` · Feature map: `docs/FEATURES.md`
Premises: Cargo workspace 없음, 크레이트 두 개 `tools/installer`와 `tools/statusline`은 서로 독립 · 경계 lint 도구 없음, 의존 측정은 `use crate::` grep · `src/`는 실행 코드가 아니라 배포되는 마크다운 설정

## How to use these rules

파일을 새 위치에 추가하거나, 새 모듈을 만들거나, unit을 넘는 import를 추가하기 전에:

1. **Units**에서 지금 있는 unit과 import하려는 unit을 찾는다.
2. 두 unit 중 하나라도 이름이 나오는 규칙을 모두 확인한다. `MUST` 위반은 결함이다. `SHOULD` 위반은 PR에 이유를 적는다.
3. **Known violations**에 있는 파일은 선례가 아니다. 그 패턴을 따라 하지 않는다.
4. `python tools/lint-arch.py`가 `Enforced by`에 이 스크립트가 적힌 규칙 9개를 검사하고, Known violations 표를 허용 목록으로 읽는다. description 규칙 2개와 문장부호 규칙은 `python tools/lint-prose.py`가 검사한다. `arch-home-dir-single-source`, `arch-skill-layout`, `arch-statusline-binaries-rebuilt`, `arch-tests-placement`, `arch-policy-in-skills`, `arch-file-size`, `arch-ko-never-installed`는 리뷰나 테스트로만 확인한다.

## Units

| Unit | Path | Responsibility | May depend on |
|---|---|---|---|
| `main` | `tools/installer/src/main.rs` | 터미널 셋업, view별 loading 분기, `--help`와 `--sync` 진입 | `app`, `loading`, `ui`, `cli`, `fs` |
| `app` | `tools/installer/src/app/` | TUI 상태 `App`, 입력, 이동, 소스 위저드 | `component`, `fs`, `mcp`, `plugin`, `source`, `theme`, `tree` |
| `ui` | `tools/installer/src/ui/` | ratatui 렌더링 | `app`, `tree`, `component`, `source`, `mcp`, `plugin`, `theme`, `fs` 상수 |
| `cli` | `tools/installer/src/cli/` | 키 입력 분기와 비대화형 `run_sync` | `app`, `fs`, `loading`, `source`, `ui` |
| `fs` | `tools/installer/src/fs/` | 스캔, 설치·제거, manifest, diff, CLI 프로세스, settings.json 쓰기 | `component`, `mcp`, `plugin`, `source`, `target`, `paths`, `exec` |
| `loading` | `tools/installer/src/loading/` | 백그라운드 스레드 채널과 대기 화면 상태 | `app`, `fs`, `component`, `mcp`, `plugin`, `process_exec` |
| `process_exec` | `tools/installer/src/process_exec.rs` | 백그라운드 설치·제거의 `ProcessData` 조정 | `app`, `component`, `fs`, `mcp`, `plugin` |
| `source` | `tools/installer/src/source/` | 소스 탐색, git clone과 캐시, `sources.yaml` | `paths`, `exec` |
| `tree` | `tools/installer/src/tree/` | 컴포넌트 목록을 접는 트리 모델 | `component` |
| leaf 타입 | `tools/installer/src/component.rs`, `tools/installer/src/mcp.rs`, `tools/installer/src/plugin.rs`, `tools/installer/src/theme.rs` | 컴포넌트·MCP·플러그인 타입과 색 테마 | 없음 |
| `paths` | `tools/installer/src/paths.rs` | 홈 디렉터리를 얻는 유일한 지점 | 없음 |
| `exec` | `tools/installer/src/exec.rs` | 짧은 외부 커맨드의 공용 실행기, 타임아웃과 정리 포함 | 없음 |
| `target` | `tools/installer/src/target.rs` | 대상 CLI `TargetCli`와 그 설정 디렉터리 | `paths` |
| `statusline` | `tools/statusline/` | stdin JSON을 상태줄로 렌더링하는 독립 바이너리 | 외부 크레이트만 |
| 배포 설정 | `src/` | agents, commands, skills, hooks, mcps, plugins, output-styles, 상태줄 바이너리 | 해당 없음 |

## Rules

### `arch-fs-no-app-import`

- **Level**: MUST
- **Rule**: `fs`의 어떤 파일도 `crate::app`을 import하지 않는다.
- **Why**: `fs`→`app` 참조는 `app`↔`fs` 순환을 만든다. 유일한 참조 대상이던 `TargetCli`를 `target` 모듈로 옮겨 해소했다. 2026-10-04 결정.
- **Evidence**: 위반 0건 · `grep -rln "crate::app" tools/installer/src/fs`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-fs-no-ui-import`

- **Level**: MUST
- **Rule**: `fs`는 `crate::ui`를 import하지 않는다.
- **Why**: 코드가 증명함, 위반 0건. 설치 로직은 화면과 무관해야 `--sync`처럼 TUI 없이도 돈다.
- **Evidence**: 0건 · `grep -rn "crate::ui" tools/installer/src/fs`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-ui-reads-state-only`

- **Level**: MUST
- **Rule**: `ui`는 `App` 상태를 읽어 그리기만 하고, `fs`의 함수를 호출하거나 파일 I/O를 하지 않는다.
- **Why**: 코드가 증명함, 위반 0건. 렌더링이 부수 효과를 가지면 다시 그릴 때마다 동작이 바뀐다.
- **Evidence**: `fs` 참조는 상수 `fs::VERSION` 하나뿐, 파일 I/O 0건 · `grep -rn "crate::fs" tools/installer/src/ui | grep -v VERSION` · `grep -rn "std::fs::" tools/installer/src/ui`
- **Exceptions**: `fs::VERSION` 상수 읽기, 버전 표시용
- **Enforced by**: `tools/lint-arch.py`

### `arch-leaf-modules-no-crate-import`

- **Level**: MUST
- **Rule**: `component`, `mcp`, `plugin`, `theme`, `source`는 크레이트 내부 모듈 가운데 `paths`와 `exec`만 import할 수 있다. `paths`와 `exec`는 아무것도 import하지 않는다.
- **Why**: 코드가 증명함, 위반 0건. 이 모듈들은 모든 unit이 가져다 쓰는 바닥이라, 위를 참조하는 순간 순환이 생긴다.
- **Evidence**: 0 edges · `grep -rnE "crate::(app|ui|cli|fs|loading|tree|process_exec)" tools/installer/src/component.rs tools/installer/src/mcp.rs tools/installer/src/plugin.rs tools/installer/src/theme.rs tools/installer/src/source`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-process-spawn-in-fs`

- **Level**: MUST
- **Rule**: 외부 프로세스 실행 호출 `.spawn()`, `.status()`, `.output()`은 `tools/installer/src/exec.rs`와 `tools/installer/src/fs/installer/process.rs`에만 둔다. `Command`를 만드는 것은 어디서든 된다.
- **Why**: 실행 지점을 모아 타임아웃, stdin 차단, 자식 프로세스 정리를 통일한다. 2026-10-04 결정은 `fs`로 통일이었지만 `source`가 내부 import를 하지 않는 leaf라서, 공용 실행기는 내부 의존이 없는 leaf 모듈 `exec`에 두었다. 설치·제거처럼 취소와 정리 콜백이 필요한 프로세스는 `process.rs`가 맡는다.
- **Evidence**: 위 두 파일 밖 실행 호출 0건 · `grep -rnE "\.spawn\(\)|\.status\(\)|\.output\(\)" tools/installer/src`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-home-dir-single-source`

- **Level**: MUST
- **Rule**: 홈 디렉터리 경로는 `tools/installer/src/paths.rs`에서만 얻는다.
- **Why**: 경로 계산이 흩어져 있으면 테스트에서 HOME을 바꾸기 어렵고 규칙이 어긋난다. 2026-10-04 결정. 접근 지점은 어떤 unit이든 순환 없이 쓸 수 있도록 내부 의존이 없는 leaf 모듈 `paths`로 정했다.
- **Evidence**: `paths.rs` 밖 호출 0건 · `grep -rn "dirs::home_dir(" tools/installer/src`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-config-writes-in-fs-or-source`

- **Level**: MUST
- **Rule**: JSON·YAML 설정 파일 쓰기는 `tools/installer/src/fs/installer/`, `tools/installer/src/fs/manifest.rs`, `tools/installer/src/source/`에서만 한다.
- **Why**: 코드가 증명함, 위반 0건. 사용자 설정을 쓰는 곳이 적어야 덮어쓰기 사고를 추적할 수 있다.
- **Evidence**: 비테스트 쓰기 지점 5곳, 모두 해당 위치 · `grep -rnE "fs::write" tools/installer/src | grep -v unwrap`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-tests-placement`

- **Level**: SHOULD
- **Rule**: 단위 테스트는 대상 파일 안의 `#[cfg(test)] mod tests`에 두고, 모듈 전체를 엮는 테스트만 `<module>/tests.rs`로 분리한다.
- **Why**: Rust의 일반 관례. 2026-10-04 결정. 렌더링과 키 라우팅 테스트는 파일 하나에 속하지 않아 분리돼 있다.
- **Evidence**: 인라인 21개 파일, 분리 2개 · `grep -rln "mod tests {" tools/installer/src | wc -l` · `grep -rn "mod tests;" tools/installer/src`
- **Exceptions**: `tools/installer/src/ui/tests.rs`, `tools/installer/src/cli/tests.rs`, 모듈 전체 테스트. 공용 픽스처는 `tools/installer/src/app/test_support.rs`
- **Enforced by**: review

### `arch-file-size`

- **Level**: SHOULD
- **Rule**: Rust 파일의 프로덕션 코드는 빈 줄, 주석, 인라인 `mod tests` 블록을 뺀 300줄 이하로 유지한다. 500줄을 넘으면 결함이다.
- **Why**: `coding-standards`의 soft 300, hard 500 한도. 테스트는 픽스처처럼 길이에서 빠진다. 2026-10-04 rustfmt 도입으로 줄이 늘어 측정 기준을 프로덕션 코드로 맞췄다.
- **Evidence**: 300줄 초과 1개, 500줄 초과 0개 · `for f in $(find tools -name '*.rs' -not -path '*/target/*' ! -name tests.rs); do echo "$(awk '/^\s*mod tests\s*\{/{exit} !/^\s*$/ && !/^\s*\/\//{n++} END{print n+0}' $f) $f"; done | sort -n | tail -3`
- **Exceptions**: 테스트 파일 `tools/installer/src/ui/tests.rs`, `tools/installer/src/cli/tests.rs`
- **Enforced by**: review

### `arch-statusline-standalone`

- **Level**: MUST
- **Rule**: `tools/statusline`은 인스톨러 크레이트를 의존하지 않고, 인스톨러도 상태줄 코드를 의존하지 않는다.
- **Why**: 코드가 증명함, path 의존과 참조 0건. 상태줄은 Claude Code가 매번 실행하는 별도 바이너리다.
- **Evidence**: 0건 · `grep -rn "hibi_ai" tools/statusline` · `grep -n "path" tools/statusline/Cargo.toml`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-statusline-binaries-rebuilt`

- **Level**: MUST
- **Rule**: `tools/statusline`의 소스를 바꾼 커밋은 `src/statusline`의 바이너리 3개도 `tools/statusline/build.sh`로 다시 빌드해 함께 갱신한다.
- **Why**: 릴리즈 워크플로는 상태줄을 빌드하지 않고 커밋된 바이너리를 그대로 싣는다. 2026-10-04 결정, 수동 빌드 유지.
- **Evidence**: 커밋된 바이너리 3개, 워크플로의 statusline 언급 0건 · `git ls-files src/statusline` · `grep -ci statusline .github/workflows/release.yml`
- **Exceptions**: none
- **Enforced by**: review

### `arch-ko-mirror`

- **Level**: MUST
- **Rule**: `src/` 아래 모든 마크다운에는 같은 위치에 `-ko.md` 한국어 미러가 있다.
- **Why**: 코드가 증명함, 위반 0건. 미러는 개발자 가독용이다.
- **Evidence**: 짝 없는 파일 0개 · `for f in $(find src -name '*.md' ! -name '*-ko.md'); do [ -f "${f%.md}-ko.md" ] || echo $f; done`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-ko-never-installed`

- **Level**: MUST
- **Rule**: 인스톨러는 이름이 `-ko`로 끝나는 파일을 설치 대상에서 뺀다.
- **Why**: 코드가 증명함. 미러가 설치되면 같은 skill이 두 번 로드된다. 판정은 한 함수에만 둬서 두 스캐너가 갈라지지 않게 한다.
- **Evidence**: 판정은 `tools/installer/src/fs/scanner/mod.rs`의 `is_korean_mirror` 한 곳 · `grep -rn 'ends_with("-ko")' tools/installer/src`
- **Exceptions**: none
- **Enforced by**: `tools/installer/src/fs/scanner/mod.rs`와 두 스캐너의 테스트

### `arch-policy-in-skills`

- **Level**: MUST
- **Rule**: 정책과 규범은 skill에 두고, 커맨드는 skill이나 agent에 위임하거나 다른 skill을 참조하는 절차만 담는다.
- **Why**: 정책의 단일 원천을 skill로 유지한다. 2026-10-04 결정, 절차형 커맨드 허용.
- **Evidence**: 위임형 20개, 절차형 7개 · `grep -Li "source of truth" $(ls src/commands/*.md | grep -v -- -ko.md)`
- **Exceptions**: 절차형 `src/commands/bugfix.md`, `src/commands/checkpoint.md`, `src/commands/learn.md`, `src/commands/orchestrate.md`, `src/commands/perf.md`, `src/commands/plan.md`, `src/commands/refactor.md`
- **Enforced by**: review

### `arch-skill-layout`

- **Level**: MUST
- **Rule**: skill 디렉터리에는 `SKILL.md`, 각 마크다운의 `-ko.md` 미러, 하위 폴더 `src/skills/<name>/references/`, `src/skills/<name>/assets/`, `src/skills/<name>/scripts/`, `src/skills/<name>/rules/`, `src/skills/<name>/evals/`만 둔다.
- **Why**: 표준 skill 구조로 고정한다. 2026-10-04 결정, 마이그레이션.
- **Evidence**: 29개 중 24개 준수 · `ls src/skills/*/`
- **Exceptions**: none
- **Enforced by**: review

### `arch-skill-name-kebab`

- **Level**: MUST
- **Rule**: skill 디렉터리 이름은 kebab-case이고 frontmatter `name`과 같다.
- **Why**: 표준 이름 규칙으로 고정한다. 2026-10-04 결정. `iced_rs`와 `ratatui_rs`를 `iced-rs`, `ratatui-rs`로 바꾸고, 사용자 설치본의 옛 디렉터리는 `tools/installer/src/fs/installer/renamed.rs`가 정리한다.
- **Evidence**: 위반 0건 · `ls src/skills | grep _`
- **Exceptions**: none
- **Enforced by**: `tools/lint-arch.py`

### `arch-description-length`

- **Level**: MUST
- **Rule**: skill과 agent의 `description`은 220자 이하다.
- **Why**: skill 목록은 전체 글자 예산을 넘으면 description이 통째로 빠진다. agent는 그 예산에 들지 않지만 같은 한도를 둔다. 2026-10-04 결정.
- **Evidence**: 위반 0건 · `python tools/lint-prose.py src`
- **Exceptions**: none
- **Enforced by**: `tools/lint-prose.py`

### `arch-description-yaml-quoted`

- **Level**: MUST
- **Rule**: `: `가 들어간 `description`은 YAML 따옴표로 감싼다.
- **Why**: 코드가 증명함, 위반 0건. 따옴표 없는 값 안의 `: `는 YAML이 거부한다.
- **Evidence**: 위반 0건 · `python tools/lint-prose.py src`
- **Exceptions**: none
- **Enforced by**: `tools/lint-prose.py`

### `arch-prose-lint`

- **Level**: MUST
- **Rule**: `src/`, `docs/`, `README.md`의 산문은 em dash, en dash, dash 대신 쓴 하이픈, 괄호를 쓰지 않는다.
- **Why**: 2026-10-04 문장부호 전면 금지 결정. `technical-writing` skill이 규칙을 소유한다.
- **Evidence**: 위반 0건 · `python tools/lint-prose.py`
- **Exceptions**: none
- **Enforced by**: `tools/lint-prose.py`

## Known violations

| Path | Rule | Decision |
|---|---|---|
| `tools/installer/src/ui/mod.rs` | `arch-file-size` | accepted, rustfmt로 302줄, hard 500 미만 |
| `src/skills/composition-patterns/README.md` | `arch-skill-layout` | fix, 내용을 `SKILL.md`나 `src/skills/<name>/references/`로 옮긴다 |
| `src/skills/dependency-design/README.md` | `arch-skill-layout` | fix |
| `src/skills/react-best-practices/README.md` | `arch-skill-layout` | fix |
| `src/skills/react-native-skills/README.md` | `arch-skill-layout` | fix |
| `src/skills/deploy-to-vercel/resources` | `arch-skill-layout` | fix, 같은 skill의 assets 폴더로 옮긴다 |

## Decisions

| Date | Question | Answer |
|---|---|---|
| `2026-10-04` | `fs` 8개 파일이 `TargetCli` 때문에 `app`을 import해 순환이 생긴다 | 규칙은 MUST, `TargetCli`를 `target` 모듈로 옮겨 해소, 완료 |
| `2026-10-04` | 커맨드 27개 중 7개가 절차를 직접 담는다 | 정책은 skill, 절차형 커맨드는 허용 |
| `2026-10-04` | description 220자 초과 4개 | MUST로 두고 4개를 바로 수정, `tools/lint-prose.py`가 길이를 검사 |
| `2026-10-04` | git만 프로세스를 `run_with_timeout` 밖에서 직접 실행한다 | 공용 실행기로 통일. leaf 규칙 때문에 위치는 `fs`가 아니라 leaf `exec`, 완료 |
| `2026-10-04` | `home_dir()` 호출이 6개 파일에 흩어져 있다 | 접근 지점 하나로 모은다. 위치는 leaf 모듈 `paths`, 완료 |
| `2026-10-04` | 테스트가 인라인과 `tests.rs` 분리로 섞여 있다 | Rust 일반 관례: 인라인 기본, 모듈 전체 테스트만 분리 |
| `2026-10-04` | skill 하위 구조와 이름에 예외가 있다 | 표준으로 고정: 마이그레이션 |
| `2026-10-04` | 상태줄 바이너리는 수동 빌드 후 커밋된다 | 수동 빌드 유지, 소스 변경 시 바이너리 갱신을 MUST로 |
| `2026-10-04` | rustfmt를 도입할까 | 도입. installer 크레이트만 먼저 포맷하고, statusline은 바이너리를 다시 빌드할 때 함께 포맷한다 |
| `2026-10-04` | rustfmt 이후 테스트를 포함해 300줄을 넘는 파일이 생겼다 | 측정을 프로덕션 코드로 바꿈. 테스트는 길이에서 빠진다 |
| `2026-10-04` | 이름을 바꾼 skill의 옛 디렉터리가 사용자 설치본에 남는다 | 설치 기록에 있는 파일만 지우고 사용자 파일과 그 디렉터리는 남긴다 |
