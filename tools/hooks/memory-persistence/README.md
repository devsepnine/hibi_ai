# Memory Persistence Hooks

세션 간 컨텍스트 연속성을 유지하기 위한 Rust 기반 크로스 플랫폼 hooks입니다.

## 기능

### 3가지 핵심 Hook

1. **session-start** - 세션 시작 시 이전 컨텍스트 로드
   - 최근 7일 이내 세션 파일 검색
   - 학습된 스킬 확인
   - 사용 가능한 컨텍스트 알림

2. **pre-compact** - 컨텍스트 압축 전 상태 보존
   - 압축 이벤트 타임스탬프 기록
   - 활성 세션 파일에 압축 발생 마킹
   - 요약 과정에서 손실될 수 있는 정보 보호

3. **session-end** - 세션 종료 시 학습 내용 영속화
   - 날짜별 세션 파일 생성/업데이트
   - 세션 시작/종료 시간 기록
   - 다음 세션을 위한 컨텍스트 템플릿 제공

## 지원 플랫폼

- ✅ **macOS** (Intel & Apple Silicon)
- ✅ **Windows** (x64)
- 🔜 **Linux** (추가 예정)

## 사용 방법

### 빌드된 바이너리 사용 (권장)

바이너리가 이미 `bin/` 디렉토리에 포함되어 있습니다:

```bash
bin/
├── macos/
│   ├── session-start
│   ├── pre-compact
│   └── session-end
└── windows/
    ├── session-start.exe
    ├── pre-compact.exe
    └── session-end.exe
```

wrapper 스크립트가 자동으로 플랫폼을 감지하여 적절한 바이너리를 실행합니다.

### 직접 빌드

Rust가 설치되어 있다면 직접 빌드할 수 있습니다:

```bash
# 간편한 빌드 (macOS + Windows)
./build-all.sh

# 또는 수동 빌드
cargo build --release                                    # macOS
cargo build --release --target x86_64-pc-windows-gnu   # Windows
```

#### 사전 요구사항

```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows 크로스 컴파일 (macOS)
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
```

## 파일 저장 위치

- **세션 파일**: `~/.claude/sessions/YYYY-MM-DD-session.tmp`
- **압축 로그**: `~/.claude/sessions/compaction-log.txt`
- **학습 스킬**: `~/.claude/skills/learned/`

## 세션 파일 구조

```markdown
# Session: 2026-01-23
**Date:** 2026-01-23
**Started:** 11:30
**Last Updated:** 15:45

---

## Current State

[Session context goes here]

### Completed
- [ ]

### In Progress
- [ ]

### Notes for Next Session
-

### Context to Load
```
[relevant files]
```
```

## 왜 Rust인가?

1. **크로스 플랫폼**: 단일 코드베이스로 Windows/macOS/Linux 지원
2. **의존성 없음**: 런타임 설치 불필요 (bash, PowerShell 등)
3. **빠른 실행**: 네이티브 바이너리로 즉시 실행
4. **메모리 안전성**: Rust의 안전성 보장

## 개발

### 프로젝트 구조

```
hooks/memory-persistence/
├── src/
│   ├── lib.rs              # 공통 유틸리티
│   ├── session_start.rs    # 세션 시작 로직
│   ├── pre_compact.rs      # 압축 전 로직
│   └── session_end.rs      # 세션 종료 로직
├── bin/                    # 빌드된 바이너리
├── Cargo.toml             # Rust 프로젝트 설정
├── build-all.sh           # 전체 빌드 스크립트
└── *-wrapper.sh           # 플랫폼 감지 wrapper
```

### 코드 수정 후 빌드

```bash
cd hooks/memory-persistence
./build-all.sh
```

## 라이센스

MIT
