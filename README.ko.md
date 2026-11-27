<p><img src="tools/installer/assets/hibi.png" alt="Hibi 아이콘" width="88"></p>

# hibi-ai

Hibi는 Claude Code와 Codex CLI 설정을 관리하는 데스크톱 및 터미널 앱입니다. 기본 제공 설정, Git 저장소, 로컬 폴더에서 필요한 스킬과 컴포넌트를 골라 설치할 수 있습니다.

[English](README.md) · **한국어** · [日本語](README.ja.md)

[다운로드](https://github.com/devsepnine/hibi_ai/releases/latest) · [릴리스 노트](https://github.com/devsepnine/hibi_ai/releases) · [이슈 등록](https://github.com/devsepnine/hibi_ai/issues)

## 주요 기능

- 데스크톱 앱에서 라이트와 다크 테마를 전환하거나, SSH에서 키보드로 터미널 화면을 조작합니다.
- 컴포넌트를 검색하고 상태별로 거릅니다. 설치된 파일을 원본과 비교하고, 제거 전에 대상 목록을 확인합니다.
- 두 도구의 스킬, 설정, MCP 서버를 관리합니다. Claude Code에서는 에이전트, 슬래시 명령, 플러그인, 훅, 출력 스타일도 관리합니다.
- Git 저장소와 로컬 폴더를 소스로 추가합니다. 데스크톱과 터미널은 소스와 설치 기록을 공유합니다.

## 설치

[GitHub Releases](https://github.com/devsepnine/hibi_ai/releases/latest)에서 운영체제에 맞는 패키지를 받으세요.

| 플랫폼 | 데스크톱 설치 | 다른 설치 방법 |
|---|---|---|
| macOS, Apple Silicon 또는 Intel | DMG를 열고 `Hibi.app`을 `Applications` 폴더로 드래그합니다. 응용 프로그램에서 Hibi를 실행합니다. | Homebrew 또는 `.tar.gz` |
| Linux, x86_64 | Debian이나 Ubuntu에서는 `.deb`, Fedora나 RHEL에서는 `.rpm`, Alpine에서는 `.apk`를 설치합니다. 앱 메뉴에서 Hibi를 실행합니다. | 설치 스크립트, glibc 기반 배포판의 Homebrew, `.tar.gz` |
| Windows, x64 | MSI를 실행한 뒤 시작 메뉴에서 Hibi를 엽니다. | Scoop 또는 휴대용 ZIP |

Linux GNU 빌드에는 glibc 2.34 이상이 필요합니다. Alpine에서는 `.apk` 또는 `linux-musl` 아카이브를 사용하세요. 같은 릴리스의 `checksums.txt`로 다운로드한 파일을 확인하세요. Alpine 패키지는 서명 없이 배포합니다. 체크섬을 확인한 뒤 root 권한으로 `apk add --allow-untrusted ./hibi-ai_*.apk`를 실행하세요.

**Homebrew**

```bash
brew install devsepnine/brew/hibi
```

**Linux 설치 스크립트**

```bash
curl -fsSL https://raw.githubusercontent.com/devsepnine/hibi_ai/main/install.sh | sh
```

스크립트는 릴리스 체크섬을 검증하고 기본으로 `~/.local`에 설치합니다. 필요하면 `~/.local/bin`을 `PATH`에 추가하세요. `HIBI_PREFIX`로 설치 위치를 바꾸고 `HIBI_VERSION`으로 릴리스 버전을 고정할 수 있습니다.

**Scoop**

```powershell
scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket
scoop install hibi-ai
```

휴대용으로 사용하려면 아카이브 전체를 풀고 해당 폴더에서 `./hibi` 또는 `hibi.exe`를 실행하세요. 앱에서 업데이트하려면 압축을 푼 폴더 이름과 번들 구성을 유지하세요.

## 빠른 시작

1. Hibi를 열거나 `hibi`를 실행하면 데스크톱 화면이 열립니다.
2. **Claude Code** 또는 **Codex CLI**를 선택하고 필요한 컴포넌트에 체크합니다.
3. **설치**를 선택해 적용합니다. 설치한 항목을 지우려면 **제거**를 선택한 뒤 대상 목록을 확인합니다.

SSH로 접속했거나 데스크톱을 열 수 없는 환경에서는 다음 명령을 실행하세요.

```bash
hibi --tui
```

터미널의 컴포넌트 목록에서 `?`를 누르면 단축키를 볼 수 있습니다. `hibi --help`와 `hibi --version`은 창을 열지 않고 도움말과 버전을 출력합니다.

## 설정 소스

**소스 관리**에서 HTTPS Git 저장소나 로컬 폴더를 추가하세요. 소스에는 `skills/` 같은 폴더나 `mcps/mcps.yaml` 같은 목록 파일을 둘 수 있습니다. `~/.hibi/sources.yaml`을 직접 편집할 수도 있습니다.

```yaml
sources:
  - type: git
    url: https://github.com/your-org/shared-configs.git
    branch: main
  - type: local
    path: ~/dotfiles/claude-configs
auto_update: true
```

기본 제공 소스의 우선순위가 가장 낮습니다. 두 소스에 같은 컴포넌트가 있으면 목록 뒤쪽의 항목을 사용합니다. `hibi --sync`를 실행하면 화면을 열지 않고 Git 소스를 갱신합니다. 가져오기에 실패하면 오류를 알리고, 캐시가 있으면 캐시를 사용합니다.

Claude Code용 소스에는 사용자가 만든 훅, 규칙, 컨텍스트도 넣을 수 있습니다. 기본 제공 설정은 작업 흐름을 스킬로 안내합니다. 사용 중단된 기본 훅의 처리 방식은 [설정 가이드](docs/features/config-behaviors.md)에서 확인할 수 있습니다.

Hibi는 설치한 컴포넌트와 출처를 `~/.hibi/install.json`에 기록합니다. 새 번들에서 빠진 기존 파일은 기록한 해시와 내용이 일치할 때 정리할 수 있습니다. 직접 수정한 파일은 보존합니다.

## Hibi 업데이트

데스크톱 앱에서 버전을 선택하면 업데이트를 확인합니다. 직접 업데이트를 지원하는 설치본은 서명된 번들을 검증한 뒤 교체하고 앱을 다시 엽니다. Homebrew와 Scoop 설치본은 해당 버전의 패키지가 게시되면 각 패키지 관리자로 업데이트합니다. Windows MSI나 Linux 시스템 패키지는 [GitHub Releases](https://github.com/devsepnine/hibi_ai/releases)에서 새 패키지를 받아 설치하세요.

## 빌드와 기여

안정 버전 Rust 도구 모음과 Python 3가 필요합니다. 저장소 루트에서 다음 명령을 실행하세요.

```bash
cargo run --locked --manifest-path tools/installer/Cargo.toml
cargo run --locked --manifest-path tools/installer/Cargo.toml --no-default-features -- --tui
python3 tools/lint-prose.py
python3 tools/lint-arch.py
cargo fmt --manifest-path tools/installer/Cargo.toml --check
cargo test --locked --manifest-path tools/installer/Cargo.toml
```

Linux 데스크톱 빌드에 필요한 라이브러리는 [릴리스 워크플로](.github/workflows/release.yml)에서 확인하세요. 설치기 동작은 [기능 맵](docs/FEATURES.md), 코드 경계는 [아키텍처 가이드](docs/ARCHITECTURE.md), 패키징은 [배포 가이드](docs/RUNBOOK.md)에 정리되어 있습니다.

버그나 기능 요청은 [GitHub 이슈](https://github.com/devsepnine/hibi_ai/issues)에 남겨 주세요. 버그를 제보할 때는 운영체제, Hibi 버전, 재현 절차를 포함해 주세요. 코드나 문서 수정은 [풀 리퀘스트](https://github.com/devsepnine/hibi_ai/pulls)로 보내 주세요.

## 라이선스

Hibi는 [MIT 라이선스](LICENSE)로 배포합니다.
