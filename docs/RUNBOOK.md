# hibi-ai 운영 가이드 (RUNBOOK)

> 마지막 업데이트: 2026-09-18 · 버전 v1.17.1

릴리즈는 ~v1.13부터 GitHub Actions로 자동화됐다. 아래 절차는 태그 푸시 이전의 준비와, 워크플로가 끝난 뒤 남는 수동 작업을 다룬다.

## 릴리즈 절차

### 1. 버전 동기화 (3곳)

릴리즈 워크플로가 세 값의 일치를 검증하고, 어긋나면 빌드 전에 실패한다.

```bash
# 1) tools/installer/Cargo.toml
#    version = "1.17.1"
#    바이너리에 각인되고 사용자의 ~/.hibi/install.json에 provenance로 기록된다
vim tools/installer/Cargo.toml

# 2) package.sh
#    VERSION="1.17.1"
vim package.sh

# 3) Cargo.lock
cargo update -w --manifest-path tools/installer/Cargo.toml
```

푸시할 태그(`v1.17.1`)의 `v` 접두어를 뗀 값이 위 두 버전과 같아야 한다.

### 2. 로컬 검증

```bash
# 테스트
cargo test --manifest-path tools/installer/Cargo.toml   # 133 tests

# 전 플랫폼 빌드 (dist/로 출력)
cd tools/installer && ./build.sh && cd ../..

# 바이너리 확인
file dist/hibi
lipo -info dist/hibi        # macOS Universal: x86_64 + arm64
file dist/hibi-linux dist/hibi.exe

# 실행 테스트
./dist/hibi
```

패키징까지 로컬에서 확인하려면:

```bash
./package.sh
ls -lh release/v1.17.1/
cat release/v1.17.1/checksums.txt
```

`release/`와 `dist/`는 gitignore 대상이므로 커밋되지 않는다.

### 3. 커밋 및 태그 푸시

```bash
git add tools/installer/Cargo.toml tools/installer/Cargo.lock package.sh
git commit -m "chore: bump version to 1.17.1"
git push origin main

git tag v1.17.1
git push origin v1.17.1
```

> **`main` 히스토리를 리라이트하지 말 것.** 메인테이너가 `main`을 rebase하면 모든 사용자 캐시의 태그가 존재하지 않는 커밋을 가리키게 되고, `git pull --ff-only`가 "would clobber existing tag"로 실패한다. v1.9.7 → v1.9.8은 이 때문에 sync 핫픽스를 급히 내보내야 했다. 불가피하게 rebase했다면 같은 릴리즈에 sync 복원 로직을 함께 넣는다.

### 4. GitHub Actions 확인

태그 푸시로 `.github/workflows/release.yml`이 트리거된다. 수동 실행은 Actions 탭의 `workflow_dispatch`(version 입력)로도 가능하다.

워크플로 단계:

1. 버전 검증 (태그 == `package.sh` VERSION == `Cargo.toml` version)
2. Rust 크로스 타겟 설치 + `mingw-w64`·`musl-cross` 설치
3. cargo 레지스트리·빌드 캐시 복원 (`Swatinem/rust-cache@v2`, workspace `tools/installer`)
4. `tools/installer/build.sh` (macOS 러너에서 전 플랫폼 크로스 컴파일)
5. `package.sh` (아카이브 + `checksums.txt`)
6. `gh release create v{VERSION} --generate-notes`

```bash
# 진행 상황
gh run list --workflow=release.yml --limit 5
gh run watch

# 결과 확인
gh release view v1.17.1
```

체크리스트:

- [ ] 워크플로 성공
- [ ] macOS `.tar.gz`, Linux `.tar.gz`, Windows `.zip` 3개 업로드
- [ ] `checksums.txt` 업로드 및 내용 확인

### 5. Homebrew Tap 갱신 (수동)

```bash
cd ../homebrew-brew

# checksums.txt에서 sha256을 가져와 version / URL / sha256 갱신
gh release view v1.17.1 --repo devsepnine/hibi_ai
vim Formula/hibi.rb

git add Formula/hibi.rb
git commit -m "chore: update hibi to v1.17.1"
git push origin main
```

### 6. Scoop Bucket 갱신 (수동)

```bash
cd ../scoop-bucket

# version / URL / hash / extract_dir 갱신
vim hibi-ai.json

git add hibi-ai.json
git commit -m "chore: update hibi-ai to v1.17.1"
git push origin master
```

## 릴리즈 후 검증

### 설치 경로 확인

```bash
# Homebrew
brew update
brew uninstall hibi
brew install --debug --verbose devsepnine/brew/hibi
which hibi && hibi --version
```

```powershell
# Scoop (Windows)
scoop uninstall hibi-ai
scoop update
scoop install hibi-ai
where hibi; hibi --version
```

### 기존 사용자 sync 확인

캐시를 가진 사용자가 새 릴리즈로 넘어오는 경로를 확인한다.

```bash
hibi --sync       # git 소스 업데이트만 수행 (TUI 없이)
ls ~/.hibi/cache/
cat ~/.hibi/install.json   # version이 새 태그를 가리키는지
```

### 다운로드 통계

```bash
gh release view v1.17.1 --json assets \
  --jq '.assets[] | {name: .name, downloads: .downloadCount}'
```

## 일반적인 문제 및 해결

### 릴리즈 워크플로 실패

#### 문제: 버전 불일치로 즉시 실패

```
# 증상
::error::Release version (1.17.1) does not match package.sh VERSION (1.15.0).

# 해결
# 세 곳(Cargo.toml / package.sh / 태그)을 맞춘 뒤 태그를 다시 만든다
git tag -d v1.17.1
git push origin :refs/tags/v1.17.1
# 버전 수정 커밋 후
git tag v1.17.1 && git push origin v1.17.1
```

#### 문제: crates.io 네트워크 타임아웃

워크플로에 `CARGO_NET_RETRY: 10`이 설정돼 있어 대부분 자동 복구된다. 그래도 실패하면 `gh run rerun <run-id>`로 재실행한다.

### 빌드 실패

#### 문제: macOS 타겟 누락

```bash
# 증상
error: can't find crate for `std`

# 해결
rustup target add aarch64-apple-darwin x86_64-apple-darwin \
  x86_64-pc-windows-gnu x86_64-unknown-linux-musl
```

#### 문제: lipo 실패

```bash
# 증상
fatal error: lipo: can't open input file

# 해결 — Xcode Command Line Tools 설치
xcode-select --install
```

#### 문제: musl 타겟 링커 에러

```bash
# 증상
error: linker `x86_64-linux-musl-gcc` not found

# 해결
brew install filosottile/musl-cross/musl-cross
```

#### 문제: mingw 링커 에러 (Windows 타겟)

```bash
# 증상
error: linker `x86_64-w64-mingw32-gcc` not found

# 해결
brew install mingw-w64
```

### 패키징 실패

#### 문제: 바이너리 파일 없음

```bash
# 증상
❌ Error: Installer binaries not found in dist/

# 해결 — 먼저 빌드
cd tools/installer && ./build.sh && cd ../..
./package.sh
```

#### 문제: 체크섬 파일이 비어 있음

```bash
# 해결
cd release/v1.17.1
shasum -a 256 *.tar.gz *.zip > checksums.txt
```

### 설치 문제

#### 문제: Homebrew 설치 실패

```bash
# 증상
Error: No available formula with the name "hibi"

# 해결
brew update
brew tap devsepnine/brew
brew install hibi
```

#### 문제: Scoop 설치 실패

```powershell
# 증상
Couldn't find manifest for 'hibi-ai'

# 해결
scoop bucket add hibi-ai https://github.com/devsepnine/scoop-bucket
scoop update
scoop install hibi-ai
```

#### 문제: macOS Gatekeeper 경고

```bash
# 증상
"hibi" cannot be opened because the developer cannot be verified

# 임시 해결
xattr -d com.apple.quarantine hibi

# 영구 해결 — Apple Developer ID 서명 필요
codesign --sign "Developer ID Application: ..." hibi
```

### 실행 문제

#### 문제: 권한 거부

```bash
# 증상
Permission denied: ./hibi

# 해결
chmod +x hibi
```

#### 문제: 잘못된 아키텍처

```bash
# 증상
Bad CPU type in executable

# 확인
file hibi
lipo -info hibi   # Intel Mac은 x86_64, Apple Silicon은 arm64 포함 필요
```

#### 문제: Windows에서 MCP 서버가 스캔되지 않음

```
# 증상 — npm으로 설치한 CLI가 목록에 없음

# 원인
# CreateProcessW는 .exe만 자동 부착하고 PATHEXT를 따르지 않아
# npm shim의 .cmd 파일이 보이지 않는다

# 해결 — v1.9.7 이상으로 업그레이드
# fs/mod.rs::resolve_cli_program이 PATH를 .exe -> .cmd -> .bat 순으로 직접 탐색한다
```

#### 문제: 사용자 캐시 sync 실패

```
# 증상
would clobber existing tag
refusing to merge unrelated histories

# 원인 — 상류 main 히스토리 리라이트
# 해결 — v1.9.9 이상으로 업그레이드
# fetch --tags --force 후 shallow는 reset --hard FETCH_HEAD,
# full clone은 merge --ff-only @{u}로 분기한다
```

## 롤백 절차

### 1. GitHub 릴리즈 롤백

```bash
gh release delete v1.17.1 --yes

# 태그 삭제 (로컬 + 리모트)
git tag -d v1.17.1
git push origin :refs/tags/v1.17.1
```

`main` 커밋은 되돌리지 않는다 — 태그만 제거하면 배포가 멈춘다. 히스토리 리라이트는 사용자 캐시를 깨뜨린다.

### 2. Homebrew Formula 롤백

```bash
cd ../homebrew-brew
git revert HEAD
git push origin main
```

### 3. Scoop Manifest 롤백

```bash
cd ../scoop-bucket
git revert HEAD
git push origin master
```

### 4. 사용자 안내

- GitHub Discussions 공지
- 필요 시 루트 `README.md`에 경고 추가

## 긴급 대응

### 보안 취약점 발견

1. **즉시 조치** — 문제 릴리즈를 Draft로 전환하거나 삭제, 루트 `README.md`에 경고
2. **수정** — 취약점 수정 커밋 → 패치 버전 릴리즈 (예: v1.17.1 → v1.17.2)
3. **알림** — GitHub Security Advisory 생성, Homebrew/Scoop 갱신

시크릿이 커밋에 들어간 경우는 릴리즈 롤백만으로 끝나지 않는다. 해당 크리덴셜을 먼저 폐기(rotate)하고, 그다음 이력 처리를 판단한다.

### 심각한 버그 발견

1. **영향 평가** — 사용자 영향 범위, 데이터 손실 여부. 인스톨러는 `~/.claude` 트리를 병합 방식으로 다루므로 설정 손상 가능성을 우선 확인한다
2. **핫픽스 릴리즈** — 긴급 수정 후 패치 버전. 사용자 캐시 sync 경로가 깨졌다면 sync 복원 로직을 같은 릴리즈에 포함한다
3. **사용자 안내** — Discussions 공지, 업그레이드 권장

## 유지보수 작업

### 주간

- [ ] GitHub Issues 확인 및 응답
- [ ] Pull Requests 리뷰
- [ ] 다운로드 통계 확인

### 월간

- [ ] 의존성 업데이트 (`cargo update -w --manifest-path tools/installer/Cargo.toml`)
- [ ] Rust 툴체인 업데이트
- [ ] 보안 스캔 (`cargo audit`)
- [ ] `src/mcps/mcps.yaml`·`src/plugins/plugins.yaml`의 상류 패키지명·URL 유효성 확인
- [ ] 문서 현행화 (`/update-docs`)

### 분기별

- [ ] 로드맵 검토
- [ ] 사용자 피드백 분석
- [ ] 아키텍처 리뷰 (`/deps`로 결합도 감사)
- [ ] 스킬 목록 예산 재측정 — `python3 src/skills/eval-harness/scripts/skill_budget.py src/skills`
  (초과면 exit 1. 스킬이 늘면 8,000자를 넘겨 설명이 절삭되고, 절삭된 스킬은
  자동 트리거를 잃는다)

## 연락처 및 리소스

- **GitHub 저장소**: https://github.com/devsepnine/hibi_ai
- **이슈 트래커**: https://github.com/devsepnine/hibi_ai/issues
- **Releases**: https://github.com/devsepnine/hibi_ai/releases
- **Homebrew Tap**: https://github.com/devsepnine/homebrew-brew
- **Scoop Bucket**: https://github.com/devsepnine/scoop-bucket
