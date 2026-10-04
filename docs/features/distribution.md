# 배포: 설치 스크립트, 패키징, 릴리즈

운영자와 설치 사용자가 보는 면이다. TUI 가 없고 출력은 셸 메시지다.

### `install-script`: 리눅스용 원라인 설치

- **위치**: 루트 `install.sh`, `curl ... | sh` 로 실행
- **별칭**: 설치 스크립트, curl, install.sh, 리눅스 설치, PATH
- **UI 문구**: `"hibi-install: error:"`, `"on macOS use Homebrew instead"`, `"on Windows use Scoop instead"`, `"unsupported OS"`, `"unsupported architecture"`, `"invalid or unresolvable version"`, `"Verifying checksum..."`, `"unexpected archive layout"`, `"no SHA-256 tool found"`, `"is not in your PATH. Add it with:"`
- **컨트롤**: 환경변수 `HIBI_VERSION` 과 `HIBI_PREFIX`
- **코드 경로**:
  - 상태 / 핸들러: `install.sh` → `main` 이 OS · 아키텍처 확인, 버전 결정, 다운로드, `verify_checksum`, 설치 순으로 간다
  - 데이터: 기본 위치는 ~/.local, 실제 파일은 lib/hibi-ai 아래, bin/hibi 는 그 심볼릭 링크
- **테스트**: none
- **함정**: macOS 나 Windows 에서 install.sh 가 바로 실패 → 리눅스만 받고 나머지는 brew · scoop 으로 보낸다, 근거: `install.sh` → `main`
- **함정**: ARM 리눅스에서 `unsupported architecture` → 릴리즈가 x86_64 만 싣는다, 근거: `install.sh` → `main`
- **함정**: bin/hibi 를 복사본으로 바꾸면 `Cannot find source directory` → 번들 설정을 실제 실행 파일 위치 기준으로 찾으므로 링크여야 한다, 근거: `tools/installer/src/source/mod.rs` → `find_package_source_dir`
- **공유 의존**: GitHub 릴리즈의 `checksums.txt`
- **관련**: `release-workflow`, `startup-loading`

### `packaging`: 빌드와 패키징

- **위치**: `tools/installer/build.sh` 로 바이너리를 만든 뒤 루트 `package.sh`, 리눅스 패키지는 `nfpm.yaml`
- **별칭**: 패키징, dist, deb, rpm, apk, nfpm, 빌드, 체크섬
- **UI 문구**: `"Installer binaries not found in dist/"`, `"Stripping -ko reference files..."`, `"nfpm not found"`, `"Generating checksums..."`, `"Missing macOS targets:"`
- **컨트롤**: 없음
- **코드 경로**:
  - 상태 / 핸들러: `tools/installer/build.sh` 가 macOS 유니버설 · Windows · 리눅스 musl 을 크로스 빌드한다
  - 상태 / 핸들러: `package.sh` 가 `src/` 를 dist 로 복사하고 `-ko.md` 를 지운 뒤 플랫폼별 tar.gz · zip 과 nfpm 패키지, 체크섬을 만든다
  - 데이터: 리눅스 패키지 배치는 `nfpm.yaml` → `contents`, 버전은 `package.sh` → `VERSION`
- **테스트**: none
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
  - 상태 / 핸들러: `.github/workflows/release.yml` 의 Resolve and verify version 단계가 태그와 `package.sh` · `tools/installer/Cargo.toml` 버전을 비교한다
  - 상태 / 핸들러: 이어서 `tools/installer/build.sh` 와 `package.sh` 를 돌리고 `gh release create` 로 올린다
  - 데이터: 바이너리에 박힌 버전은 사용자 install.json 의 `version` 이 된다, `tools/installer/src/fs/manifest.rs` → `build`
- **테스트**: none
- **함정**: 태그를 밀었는데 워크플로가 버전 오류로 멈춤 → 태그와 `package.sh` 와 `Cargo.toml` 세 버전이 모두 같아야 한다, 근거: `.github/workflows/release.yml` → `PKG_VERSION`
- **함정**: 수동 실행 입력값이 셸에 그대로 끼어들 수 있었음 → 입력을 run 식에 직접 넣던 것을 env 로 넘기도록 고쳤다, 근거: `b22fb9a`
- **함정**: 상태줄 수정이 릴리즈에 안 들어감 → 워크플로는 인스톨러만 빌드한다, 근거: `.github/workflows/release.yml` → `build`
- **공유 의존**: macOS 러너의 brew 로 받는 mingw-w64 · musl-cross · nfpm
- **관련**: `packaging`, `install-manifest`, `statusline-render`
