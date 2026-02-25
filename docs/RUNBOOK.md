# hibi-ai 운영 가이드 (RUNBOOK)

> 마지막 업데이트: 2026-02-25

## 배포 절차

### 1. 버전 업데이트

```bash
# 1. Cargo.toml 버전 업데이트
vim tools/installer/Cargo.toml
# version = "0.1.x" 수정

# 2. package.sh 버전 업데이트
vim package.sh
# VERSION="0.1.x" 수정
```

### 2. 빌드 및 테스트

```bash
# Installer 빌드 (dist/로 출력)
cd tools/installer
./build.sh

# Hooks & Statusline 빌드 (필요시만 - src/에서 Git 관리)
cd tools/statusline && ./build.sh
cd tools/hooks/inject_guide && ./build.sh
cd tools/hooks/memory-persistence && ./build.sh
cd tools/hooks/strategic-compact && ./build.sh

# 바이너리 검증
cd ../..
file dist/hibi
lipo -info dist/hibi  # macOS Universal Binary 확인

# 실행 테스트
./dist/hibi
```

### 3. 릴리즈 패키징

```bash
# 릴리즈 패키지 생성
./package.sh

# 생성 확인
ls -lh release/v0.1.x/
cat release/v0.1.x/checksums.txt
```

### 4. GitHub 릴리즈

```bash
# GitHub CLI로 릴리즈 생성
gh release create v0.1.x \
  release/v0.1.x/*.tar.gz \
  release/v0.1.x/*.zip \
  release/v0.1.x/checksums.txt \
  --title "hibi-ai v0.1.x" \
  --notes-file release/v0.1.x/RELEASE_NOTES.md

# 또는 웹 UI 사용:
# https://github.com/devsepnine/hibi_ai/releases/new
```

### 5. Homebrew Tap 업데이트

```bash
# Tap 저장소로 이동
cd ../homebrew-tap

# Formula 업데이트 (버전, URL, SHA256)
vim Formula/hibi.rb

# 커밋 및 푸시
git add Formula/hibi.rb
git commit -m "chore: update hibi to v0.1.x"
git push origin main
```

### 6. Scoop Bucket 업데이트

```bash
# Scoop bucket 저장소로 이동
cd ../scoop-bucket

# Manifest 업데이트 (버전, URL, SHA256)
vim hibi-ai.json

# 커밋 및 푸시
git add hibi-ai.json
git commit -m "chore: update hibi-ai to v0.1.x"
git push origin main
```

## 모니터링 및 알림

### GitHub Actions

릴리즈 후 확인 사항:
- [ ] GitHub Actions 빌드 성공
- [ ] 모든 플랫폼 바이너리 업로드 확인
- [ ] 체크섬 파일 생성 확인

### 다운로드 통계

```bash
# GitHub CLI로 릴리즈 다운로드 통계 확인
gh release view v0.1.x --json assets \
  --jq '.assets[] | {name: .name, downloads: .downloadCount}'
```

### Homebrew 설치 확인

```bash
# 로컬 테스트
brew uninstall hibi
brew install --debug --verbose devsepnine/brew/hibi

# 설치 확인
which hibi
hibi --version
```

### Scoop 설치 확인

```powershell
# Windows에서 테스트
scoop uninstall hibi-ai
scoop install hibi-ai

# 설치 확인
where hibi
hibi --version
```

## 일반적인 문제 및 해결

### 빌드 실패

#### 문제: macOS 타겟 누락

```bash
# 증상
error: can't find crate for `std`

# 해결
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
```

#### 문제: lipo 실패

```bash
# 증상
fatal error: lipo: can't open input file

# 해결
# Xcode Command Line Tools 설치
xcode-select --install
```

#### 문제: musl 타겟 링커 에러

```bash
# 증상
error: linker `x86_64-linux-musl-gcc` not found

# 해결
brew install filosottile/musl-cross/musl-cross
```

### 패키징 실패

#### 문제: 바이너리 파일 없음

```bash
# 증상
cp: hibi-linux: No such file or directory

# 해결
# 먼저 빌드 실행
cd tools/installer
./build.sh
cd ../..
./package.sh
```

#### 문제: 체크섬 생성 실패

```bash
# 증상
checksums.txt 파일이 비어있음

# 해결
cd release/v0.1.x
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

# 영구 해결
# Apple Developer ID로 바이너리 서명 필요
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
lipo -info hibi

# 해결
# Universal Binary가 맞는지 확인
# Intel Mac: x86_64 포함 필요
# Apple Silicon: arm64 포함 필요
```

## 롤백 절차

### 1. GitHub 릴리즈 롤백

```bash
# 문제 있는 릴리즈 삭제
gh release delete v0.1.x --yes

# 태그 삭제 (로컬 + 리모트)
git tag -d v0.1.x
git push origin :refs/tags/v0.1.x
```

### 2. Homebrew Formula 롤백

```bash
cd ../homebrew-tap

# 이전 버전으로 되돌림
git revert HEAD
git push origin main

# 또는 직접 수정
vim Formula/hibi.rb
# 이전 버전으로 수정
git add Formula/hibi.rb
git commit -m "revert: rollback to v0.1.y"
git push origin main
```

### 3. Scoop Manifest 롤백

```bash
cd ../scoop-bucket

# 이전 버전으로 되돌림
git revert HEAD
git push origin main

# 또는 직접 수정
vim hibi-ai.json
# 이전 버전으로 수정
git add hibi-ai.json
git commit -m "revert: rollback to v0.1.y"
git push origin main
```

### 4. 사용자 안내

```bash
# GitHub Discussions에 공지
# 또는 README에 경고 추가
```

## 긴급 대응

### 보안 취약점 발견

1. **즉시 조치**
   - 문제 릴리즈 삭제 또는 Draft로 전환
   - README에 경고 추가

2. **수정**
   - 취약점 수정 커밋
   - 긴급 패치 버전 릴리즈 (예: v0.1.3 → v0.1.4)

3. **알림**
   - GitHub Security Advisory 생성
   - Homebrew/Scoop 업데이트

### 심각한 버그 발견

1. **영향 평가**
   - 사용자 영향 범위 확인
   - 데이터 손실 여부 확인

2. **핫픽스 릴리즈**
   - 긴급 수정
   - 패치 버전 릴리즈

3. **사용자 안내**
   - GitHub Discussions 공지
   - 업그레이드 권장

## 유지보수 작업

### 주간 체크리스트

- [ ] GitHub Issues 확인 및 응답
- [ ] Pull Requests 리뷰
- [ ] 다운로드 통계 확인
- [ ] Homebrew/Scoop 설치 테스트

### 월간 체크리스트

- [ ] 의존성 업데이트 (`cargo update`)
- [ ] Rust 버전 업데이트
- [ ] 보안 취약점 스캔 (`cargo audit`)
- [ ] 문서 업데이트 검토

### 분기별 체크리스트

- [ ] 로드맵 검토
- [ ] 사용자 피드백 분석
- [ ] 성능 개선 검토
- [ ] 아키텍처 리뷰

## 연락처 및 리소스

- **GitHub 저장소**: https://github.com/devsepnine/hibi_ai
- **이슈 트래커**: https://github.com/devsepnine/hibi_ai/issues
- **Homebrew Tap**: https://github.com/devsepnine/homebrew-brew
- **Scoop Bucket**: https://github.com/devsepnine/scoop-bucket
