# hibi-ai 운영 가이드

이 문서는 릴리스 준비, 플랫폼별 검증, 게시, 배포 문제 대응을 다룬다. 사용자 설치 절차는 [루트 README](../README.ko.md), 앱에서 업데이트 방식을 고르는 기준은 [배포 기능맵](features/distribution.md)을 참고한다.

## 릴리스 준비

### 버전과 태그 준비

`tools/installer/Cargo.toml`의 `version`과 `package.sh`의 `VERSION`을 같은 값으로 변경한다. `Cargo.lock`은 아래 명령으로 갱신한다. 바이너리의 버전은 파일 컴포넌트를 설치하거나 제거할 때 `~/.hibi/install.json`에 기록된다.

```bash
cargo update -w --manifest-path tools/installer/Cargo.toml
```

릴리스 워크플로는 이 버전과 `v<version>` 태그를 대조한다. 수동 실행에서도 기존 태그가 가리키는 커밋을 선택하고 `v`를 뺀 버전을 입력한다. `main`을 리라이트하면 기존 Git 소스 캐시가 같은 태그의 다른 커밋을 가리킬 수 있으므로 공개 이력에 새 커밋을 추가한다.

### 릴리스 서명

서명 담당자는 `tools/installer/assets/update.pub`와 짝을 이루는 minisign 비밀 키를 저장소의 Actions Secret `HIBI_UPDATE_SECRET_KEY`에 등록한다. 게시 작업은 완성된 패키지의 SHA-256을 확인하고 `checksums.txt`를 서명한 뒤, 앱에 내장된 공개키로 서명을 검증한다. 키가 없거나 공개키와 일치하지 않으면 게시에 실패한다. 기존 앱이 새 배포를 검증하려면 같은 공개키로 서명이 검증되어야 한다.

정식판은 정식 릴리스를 확인한다. 시험판은 더 높은 시험판과 정식판을 확인한다. 시험판 태그에는 `--prerelease --latest=false`가 적용되고 Homebrew와 Scoop의 정식 배포 버전은 기존 값을 유지한다. [릴리스 워크플로](../.github/workflows/release.yml)

## 빌드 전 검사

저장소 루트에서 실행한다. 릴리스 작업은 상태줄 소스와 바이너리의 변경 이력을 확인하므로 전체 Git 이력을 가져온다.

```bash
python3 tools/lint-prose.py
python3 tools/lint-prose.py README.ko.md README.ja.md
python3 tools/lint-arch.py
cargo fmt --manifest-path tools/installer/Cargo.toml --check
cargo test --locked --manifest-path tools/installer/Cargo.toml
cargo test --locked --manifest-path tools/installer/Cargo.toml --no-default-features
cargo test --locked --manifest-path tools/statusline/Cargo.toml
```

Ubuntu의 검증 환경에서 다음 명령으로 Linux 설치 스크립트, 서명 업데이트와 복구, 실제 터미널 및 GUI 입력을 확인한다. Xvfb, Python Xlib, Pillow, Tesseract 영어 데이터와 minisign 등은 릴리스 워크플로의 `verify` 작업과 같은 설치 목록을 사용한다.

```bash
python3 tools/installer/tests/install_smoke.py
xvfb-run -a python3 tools/installer/tests/update_smoke.py
cargo build --locked --manifest-path tools/installer/Cargo.toml
python3 tools/installer/tests/cli_smoke.py \
  --binary tools/installer/target/debug/hibi --artifacts /tmp/hibi-cli-smoke
xvfb-run -a -s "-screen 0 1280x900x24" \
  python3 tools/installer/tests/gui_smoke.py \
  --binary tools/installer/target/debug/hibi --artifacts /tmp/hibi-gui-smoke
```

GUI와 PTY 스모크는 임시 사용자 디렉터리를 사용한다. CI의 `gui-acceptance` 아티팩트에서 화면과 로그를 확인한다. macOS에서 GUI를 수동 검수할 때는 빌드한 앱을 LaunchServices로 열고 활성화된 창에서 키 입력을 확인한다.

### Claude Code와 Codex 설정 로딩 확인

1. 기존 프로젝트 설정이 없는 테스트 디렉터리에서 시작한다. Claude Code와 Codex에 별도의 임시 OS 사용자 프로필을 사용한다. 설치 전에 hibi가 쓰는 경로와 대상 CLI가 읽는 경로가 같은 임시 프로필에 있는지 확인하고, 그곳에 같은 hibi 번들로 각각 설치한다. CLI 버전과 설치한 항목을 기록하고 지원 여부가 불분명하면 해당 버전의 공식 문서와 대조한다.
2. 파일 위치와 구문을 확인한 뒤 각 CLI에서 변경한 항목을 실제로 사용한다. 스킬은 더미 입력으로 호출한다. MCP 목록으로 등록을 확인하고 연결 동작까지 검증할 때는 테스트용 서버에 연결한다. CLI 시작 여부와 확인한 로딩 동작을 따로 기록한다.
3. 같은 설정 파일의 hibi 관리 범위 밖에 예제 키를 추가한다. 파일 보존이 검증 대상이면 관리되지 않는 예제 파일을 추가한다. 같은 번들을 재설치한 뒤 예제 키의 값과 파일 내용이 그대로이고 설치한 항목이 계속 로드되는지 확인한다. 기록에는 확인한 경로와 결과를 남기고 인증 정보는 가린다.

## 플랫폼별 빌드와 패키징

`tools/installer/build.sh`에 인자 없이 실행하면 현재 호스트에 맞는 빌드 하나를 선택한다. 아래의 명시적 대상은 해당 환경에서 실행한다.

| 환경 | 명령 | 결과 |
|---|---|---|
| macOS, Xcode Command Line Tools | `bash tools/installer/build.sh macos` | Intel과 Apple Silicon용 Universal `dist/hibi` |
| Windows x64, MSVC Rust와 Bash | `bash tools/installer/build.sh windows` | `dist/hibi.exe` |
| GNU Linux x86_64, CI는 AlmaLinux 9 | `bash tools/installer/build.sh linux` | glibc 2.34 이상을 쓰는 `dist/hibi-linux` |
| Alpine x86_64, CI는 Rust Alpine 컨테이너 | `bash tools/installer/build.sh linux-musl` | 동적 musl 기반 `dist/hibi-linux-musl` |

macOS 빌드가 끝나면 같은 호스트에서 `bash package.sh --macos-app`으로 `Hibi.app`을 준비한다. 이 모드는 앱을 로컬 서명하고 설치용 DMG와 업데이트용 ZIP을 `dist/`에 만든다. CI는 네 플랫폼 바이너리와 macOS 패키지를 한곳에 모아 `bash package.sh`를 실행한다.

전체 패키징에는 Windows MSI를 만드는 `wixl` 0.103 이상, `msiinfo`, `exiftool`이 필요하다. Linux 시스템 패키지는 `nfpm`으로 만든다. CI는 패키징 뒤 `package_smoke.sh`로 Linux 설치와 업그레이드를 확인한다. Windows 러너는 `windows_msi_smoke.ps1`로 MSI 설치, 실패한 업그레이드 복구, 성공한 업그레이드를 확인한다.

상태줄은 `tools/statusline/`에서 별도로 빌드한다. 상태줄 소스를 바꾸면 세 바이너리를 다시 만들고 `src/statusline/`과 함께 갱신한다. `tools/statusline/build.sh`는 macOS에서 `lipo`와 macOS 타깃, Windows GNU 및 Linux musl 크로스 도구를 사용한다. Linux 타깃이 없으면 해당 바이너리를 건너뛰므로 세 출력 파일을 모두 확인한다. [상태줄 기능맵](features/statusline.md)

## 게시와 결과 확인

릴리스 대상 커밋을 검토한 뒤 권한 있는 운영자가 `v<version>` 태그를 게시한다. GitHub Actions에서 검사, 빌드, 패키징, Windows MSI 검증, 최종 게시까지 이어지는지 확인한다.

```bash
gh run list --workflow release.yml --limit 5
gh run watch
gh release view "$release_tag"
```

`release_tag`는 확인할 태그이며 `v`를 포함한다. 다음 파일이 모두 제공되는지 확인한다.

- macOS DMG와 앱 ZIP, macOS 터미널 배포용 아카이브
- Windows x64 MSI와 휴대용 ZIP
- GNU Linux와 Alpine 아카이브, Linux `.deb`, `.rpm`, `.apk`
- 설치와 업데이트 파일 열 개의 `checksums.txt` 및 `checksums.txt.minisig`

릴리스 파일을 검증할 때는 파일이 내려받아진 디렉터리에서 아래 명령을 실행한다. `release_public_key`는 체크아웃에 있는 `tools/installer/assets/update.pub`의 절대 경로다. `sha256sum -c`는 해당 체크섬에 적힌 파일들의 존재도 확인한다.

```bash
minisign -Vm checksums.txt -p "$release_public_key"
sha256sum -c checksums.txt
```

### Homebrew와 Scoop 갱신

정식판이 게시되면 Homebrew `Formula/hibi.rb`의 macOS 및 Linux GNU 아카이브 URL, 버전과 체크섬을 갱신한다. Linux Formula가 배포 바이너리의 데스크톱 라이브러리를 찾도록 기존 의존성과 RPATH 설정을 유지한다. 테스트용 Tap에서 설치와 업그레이드를 검증하고 나서 게시한다. [Homebrew Formula](https://github.com/devsepnine/homebrew-brew/blob/main/Formula/hibi.rb)

Scoop의 저장소 루트 `hibi-ai.json`에서 버전, Windows ZIP URL, 해시, 압축 경로를 갱신한다. 테스트용 Bucket에서 이전 버전을 올린 뒤 새 버전을 검증한다. [Scoop Manifest](https://github.com/devsepnine/scoop-bucket/blob/master/hibi-ai.json)

두 설치본에서 `hibi --version`과 `hibi --tui`를 실행하고 GUI에서도 첫 화면과 번들 목록을 확인한다. 앱이 업데이트를 제안해도 해당 패키지의 새 버전이 게시되기 전에는 배포 대기 상태가 표시될 수 있다.

### 배포 후 사용자 경로 검증

1. macOS DMG에서 Applications로 복사한 앱, Windows MSI, Linux 시스템 패키지 및 직접 설치본을 각 환경에서 연다.
2. 이전 버전에서 앱 업데이트를 확인한다. 서명된 직접 설치본이 새 버전으로 재실행되고, MSI와 Linux 시스템 패키지는 릴리스 페이지로 안내되는지 확인한다.
3. GUI의 **소스 관리**에서 기존 소스를 확인한다. `hibi --sync`로 실제로 캐시를 갱신한다.
4. 임시 사용자 설정에 파일 컴포넌트 하나를 설치하고 `~/.hibi/install.json`의 `version`과 설치 파일을 확인한다. 소스 동기화만 실행한 경우에는 설치 기록이 갱신되지 않는다.

다운로드 통계는 릴리스 자산을 조회한다.

```bash
gh release view "$release_tag" --json assets \
  --jq '.assets[] | [.name, .downloadCount] | @tsv'
```

## 문제 해결

| 증상 | 확인할 곳 |
|---|---|
| 태그 실행이 버전 검사에서 실패 | 태그의 커밋과 워크플로가 체크아웃한 커밋, `Cargo.toml`과 `package.sh`의 버전을 비교한다. `Cargo.lock` 변경이 빠졌으면 갱신한다. |
| 의존성 다운로드가 반복해서 실패 | Actions의 다운로드 로그와 crates.io 상태를 확인한다. 코드나 버전을 바꾸기 전에 동일 커밋에서 일시적 네트워크 문제인지 확인한다. |
| macOS에서 타깃이나 `lipo`를 찾지 못함 | `rustup target list --installed`, `xcode-select -p`를 확인한다. 빌드 결과는 `lipo -info dist/hibi`로 확인한다. |
| Linux GNU 또는 Alpine 바이너리가 실행되지 않음 | `file`과 `ldd`로 아키텍처와 로더를 확인한다. glibc 배포판은 GNU, Alpine은 동적 musl 패키지를 사용한다. |
| 데스크톱 실행이 `No graphical display is available`로 실패 | SSH나 헤드리스 환경에서는 `hibi --tui`를 실행한다. 데스크톱 환경이라면 디스플레이와 X11 또는 Wayland 라이브러리를 확인한다. |
| `Cannot find source directory` | 휴대용 설치는 압축을 푼 폴더에서 실행한다. Linux 스크립트 설치의 `bin/hibi`가 원본 번들의 심볼릭 링크인지 확인한다. |
| 패키징에 필요한 실행 파일이 없음 | 플랫폼별 파일이 모두 `dist/`에 있고 macOS 앱 ZIP과 DMG도 생성됐는지 확인한다. Linux 패키지가 빠졌으면 `nfpm` 설치 로그를 확인한다. |
| Homebrew와 Scoop에서 새 버전을 못 찾음 | 패키지 관리자의 인덱스를 갱신하고 Formula 또는 Manifest가 원하는 버전을 가리키는지 확인한다. |
| Git 소스가 오래된 사본으로 표시됨 | Git 설치와 네트워크, URL·브랜치 조합을 확인하고 `hibi --sync`를 실행한다. 실패한 소스가 있으면 0이 아닌 상태로 종료한다. |
| macOS가 앱을 차단함 | Apple 안내와 릴리스의 체크섬을 확인한다. 번들의 로컬 서명과 Apple Developer ID 공증은 다른 배포 절차다. |
| MCP 상태가 `unknown` | Claude는 선택한 스코프의 레지스트리와 프로젝트 경로를 확인한다. Codex는 CLI 경로와 조회 오류를 확인한다. [MCP 기능맵](features/installer-list.md) |

## 배포 실패와 복구

직접 업데이트가 실패하면 표시된 오류를 확인한다. 재실행에 실패하면 교체 코드가 이전 번들을 복구하며, 성공 시 백업은 설치 위치 옆의 `.hibi-update-*`에 남는다. 패키지 설치가 실패하면 패키지 관리자 또는 MSI 로그에서 실패 원인과 롤백 여부를 확인한다.

공개 릴리스에 결함이 있으면 영향받는 버전과 설치 방식을 적고, 확인된 이전 버전으로 돌아가는 방법 또는 수정본을 안내한다. Homebrew Formula와 Scoop Manifest가 결함 버전을 가리키는지 별도로 확인한다. 필요하면 알려진 정상 버전의 URL과 체크섬으로 되돌리고 설치를 검증한다. 공개 태그 삭제가 필요한 경우 사용자 캐시와 패키지 URL에 미치는 영향을 먼저 검토한다.

보안 문제가 의심되면 재현 자료에서 키와 개인 경로를 지우고 저장소의 보안 신고 경로를 확인한다. 수정본을 배포할 때는 영향 범위, 검증 방법, 사용자 조치를 릴리스 노트에 적는다.

## 유지보수

- 실패한 릴리스 작업과 설치·업데이트 이슈를 먼저 확인한다.
- 배포 도구를 바꾼 뒤에는 새 설치와 이전 버전에서의 업그레이드를 모두 검증한다.
- 상태줄을 바꾸면 소스 테스트와 커밋할 플랫폼별 바이너리 세 개를 확인한다.
- 문서의 화면 이름과 검증 명령은 현재 기능맵 및 워크플로와 대조한다.
