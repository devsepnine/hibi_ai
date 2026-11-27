# 배포와 앱 업데이트

설치 스크립트, 릴리스 빌드와 앱 업데이트를 찾는 문서다. 실행 순서는 [운영 가이드](../RUNBOOK.md)를 참고한다.

### `install-script`: 리눅스용 원라인 설치

- **위치**: 루트 `install.sh`, `curl ... | sh` 로 실행
- **별칭**: 설치 스크립트, curl, install.sh, 리눅스 설치, PATH
- **UI 문구**: `"hibi-install: error:"`, `"on macOS use Homebrew instead"`, `"on Windows use Scoop instead"`, `"unsupported OS"`, `"unsupported architecture"`, `"invalid or unresolvable version"`, `"Verifying checksum..."`, `"unexpected archive layout"`, `"no SHA-256 tool found"`, `"is not in your PATH. Add it with:"`
- **컨트롤**: 환경변수 `HIBI_VERSION` 과 `HIBI_PREFIX`
- **코드 경로**:
  - 상태 / 핸들러: `install.sh` → `main` 이 OS · 아키텍처 확인, 버전 결정, 다운로드, `verify_checksum`, 설치 순으로 간다
  - 데이터: 기본 위치는 ~/.local, 실제 파일은 lib/hibi-ai 아래, bin/hibi 는 그 심볼릭 링크
  - 데이터: 설치 스크립트는 `flock`으로 같은 경로의 설치와 업데이트를 조정한다. 기존 번들은 새 번들과 실행 링크가 준비될 때까지 별도 디렉터리에 보관한다.
- **테스트**: `tools/installer/tests/install_smoke.py`가 상대 설치 경로, 교체 실패 후 이전 실행 파일, 재시도를 확인한다.
- **함정**: macOS 나 Windows 에서 install.sh 가 바로 실패 → 리눅스만 받고 나머지는 brew · scoop 으로 보낸다, 근거: `install.sh` → `main`
- **함정**: ARM 리눅스에서 `unsupported architecture` → 릴리즈가 x86_64 만 싣는다, 근거: `install.sh` → `main`
- **함정**: Alpine에서 GNU 아카이브가 실행되지 않음 → Alpine은 musl을 사용한다. 설치 스크립트는 체크섬에 musl 아카이브가 있으면 그 파일을 사용하고 구형 릴리스는 기존 Linux 아카이브로 되돌아간다, 근거: `install.sh` → `main`
- **함정**: bin/hibi 를 복사본으로 바꾸면 `Cannot find source directory` → 번들 설정을 실제 실행 파일 위치 기준으로 찾으므로 링크여야 한다, 근거: `tools/installer/src/source/mod.rs` → `find_package_source_dir`
- **공유 의존**: GitHub 릴리즈의 `checksums.txt`
- **관련**: `release-workflow`, `startup-loading`

### `packaging`: 빌드와 패키징

- **위치**: `tools/installer/build.sh` 로 바이너리를 만든 뒤 루트 `package.sh`, 리눅스 패키지는 `nfpm.yaml`
- **별칭**: 패키징, dist, deb, rpm, apk, nfpm, 빌드, 체크섬
- **UI 문구**: `"Installer binaries not found in dist/"`, `"Stripping -ko reference files..."`, `"nfpm not found"`, `"Generating checksums..."`, `"Missing $mac_app.$ext. Build it on macOS with bash package.sh --macos-app."`
- **컨트롤**: 없음
- **코드 경로**:
  - 상태 / 핸들러: `tools/installer/build.sh`는 플랫폼별 네이티브 러너에서 macOS 유니버설, Windows, Linux GNU, Alpine musl을 빌드한다. 인수는 `native`, `macos`, `windows`, `linux`, `linux-musl`이다.
  - 상태 / 핸들러: `package.sh`는 설정을 `dist/`로 모으고 `-ko.md`를 제외한다. 아카이브, macOS DMG·앱 ZIP, Windows MSI, Linux 패키지와 체크섬을 `release/`에 둔다.
  - 데이터: 리눅스 패키지 배치는 `nfpm.yaml` → `contents`, 버전은 `package.sh` → `VERSION`
  - 데이터: deb와 rpm은 GNU 바이너리, apk는 동적 musl 바이너리를 싣는다. Linux 패키지는 X11, Wayland의 실행 의존성과 데스크톱 실행 항목을 선언한다.
- **호환성**: GNU 릴리즈는 AlmaLinux 9에서 빌드하며 glibc 2.34 이상을 사용한다. 릴리즈 워크플로는 Fedora, AlmaLinux, Debian, Alpine에서 완성된 패키지의 설치와 업그레이드를 검사한다.
- **Windows 검사**: 릴리즈 워크플로는 Windows Installer에서 MSI 설치와 업그레이드를 실행한다. 업그레이드 실패 뒤 이전 설치가 복구되는지도 확인한다.
- **테스트**: `tools/installer/tests/package_smoke.sh`는 CI에서 Linux 패키지 설치와 업그레이드를 실행한다. `tools/installer/tests/windows_msi_smoke.ps1`는 설치와 성공 및 실패한 업그레이드를 검사한다.
- **함정**: src 에서 지운 파일이 릴리즈 압축에 남아 있음 → `cp -r` 은 덮기만 하고 지우지 않아서 dist 를 먼저 비우도록 고쳤다, 근거: `13e146d`
- **함정**: 새 최상위 컴포넌트 디렉터리를 만들었는데 deb · rpm 에 없음 → `package.sh` 의 두 복사 목록과 `nfpm.yaml` 의 contents 가 디렉터리를 하나씩 나열한다, 근거: `nfpm.yaml` → `contents`
- **함정**: 로컬에서 deb · rpm 이 안 생김 → nfpm 이 없으면 경고만 하고 건너뛴다, 근거: `package.sh` → `nfpm`
- **공유 의존**: `src/` 전체
- **관련**: `release-workflow`, `component-lookup`

### `release-workflow`: 태그 푸시로 도는 릴리즈

- **위치**: GitHub Actions 의 Release 워크플로, `v*.*.*` 태그 푸시나 수동 실행
- **별칭**: 릴리즈, 배포, 태그, GitHub Actions, 버전 불일치
- **UI 문구**: `"does not match package.sh VERSION"`, `"does not match tools/installer/Cargo.toml version"`, `"Version to release"`
- **컨트롤**: 수동 실행 입력 `version`
- **코드 경로**:
  - 상태 / 핸들러: `.github/workflows/release.yml`의 Resolve and verify version 단계가 태그, `package.sh`와 `tools/installer/Cargo.toml`의 버전을 비교하고 기존 태그가 현재 커밋을 가리키는지 확인한다
  - 상태 / 핸들러: 검증 뒤 네이티브 빌드를 모아 `package.sh`로 패키징한다. Windows MSI 검증을 마치면 SHA-256을 대조하고 minisign 서명을 생성·검증한 뒤 `gh release create`로 게시한다.
  - 데이터: 바이너리에 박힌 버전은 사용자 install.json 의 `version` 이 된다, `tools/installer/src/fs/manifest.rs` → `build`
- **테스트**: `tools/installer/tests/install_smoke.py`, `tools/installer/tests/cli_smoke.py`, `tools/installer/tests/gui_smoke.py`, `tools/installer/tests/update_smoke.py` 및 플랫폼별 패키지 검증
- **함정**: 태그를 밀었는데 워크플로가 버전 오류로 멈춤 → 태그와 `package.sh` 와 `Cargo.toml` 세 버전이 모두 같아야 한다, 근거: `.github/workflows/release.yml` → `PKG_VERSION`
- **함정**: 수동 실행 입력값이 셸에 그대로 끼어들 수 있었음 → 입력을 run 식에 직접 넣던 것을 env 로 넘기도록 고쳤다, 근거: `b22fb9a`
- **함정**: 상태줄 수정이 릴리즈에 안 들어감 → 워크플로는 인스톨러만 빌드한다, 근거: `.github/workflows/release.yml` → `build`
- **공유 의존**: 플랫폼별 Rust 툴체인과 Linux 화면 라이브러리. Alpine은 Docker의 Rust Alpine 이미지에서 빌드한다. 패키징은 nfpm을 사용한다.
- **관련**: `packaging`, `install-manifest`, `statusline-render`

### `app-update`: GUI 버전 버튼과 업데이트

- **위치**: 앱 오른쪽 아래 버전과 업데이트 대화상자
- **별칭**: 앱 업데이트, 버전 확인, self-update, 업데이트 대기, 다시 실행, 서명 오류
- **UI 문구**: `"Checking for updates…"`, `"Up to date"`, `"Waiting for the package update"`, `"Open release page"`, `"Update failed"`
- **컨트롤**: 버전 버튼으로 확인하고 새 버전이 있으면 업데이트를 승인한다. 설치 방식에 따라 패키지 공개를 기다리거나 릴리스 페이지를 연다. 설치·제거 등 진행 중인 작업이 끝난 뒤 승인할 수 있다.
- **코드 경로**:
  - UI: `tools/installer/src/gui/updates.rs` → `handle`, `footer`, `dialog`
  - 상태: `tools/installer/src/fs/update/mod.rs` → `check`와 `apply`. `tools/installer/src/fs/update/releases.rs`가 릴리스를 조회한다. `tools/installer/src/fs/update/origin.rs` → `resolve`가 현재 설치 방식과 패키지 공개 여부를 판정한다.
  - 데이터: `tools/installer/src/fs/update/bundle.rs`에서 내장 공개키로 체크섬 서명을 확인하고 번들을 검증한다. 교체 뒤 재실행이 실패하면 이전 번들을 복구한다. Windows의 파일 교체와 복구는 `tools/installer/src/fs/update/windows-update.ps1`이 수행한다.
- **테스트**: `tools/installer/tests/update_smoke.py`, `tools/installer/src/fs/update/bundle.rs`, `tools/installer/src/fs/update/origin.rs`, `tools/installer/src/fs/update/windows.rs`
- **함정**: 새 버전이 있는데 업데이트 버튼으로 설치할 수 없음 → Homebrew와 Scoop이 같은 버전을 게시할 때까지 대기한다. MSI와 Linux 시스템 패키지는 릴리스 페이지를 연다, 근거: `tools/installer/src/fs/update/origin.rs` → `resolve`
- **함정**: 휴대용 설치를 옮긴 뒤 직접 업데이트 대신 릴리스 페이지가 열림 → 이름이나 번들 마커가 기대값과 다르면 직접 교체 대상으로 분류하지 않는다, 근거: `tools/installer/src/fs/update/origin.rs` → `direct_root`
- **관련**: `release-workflow`, `packaging`, `gui-appearance`
