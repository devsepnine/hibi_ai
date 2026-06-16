# Review Checklist

> `coding-standards` skill의 참조 문서. SOLID, Clean Code, Functionality,
> Consistency + security/testing/performance를 강조한다.

리뷰 우선순위 (집행 가중치 순): **SOLID → Clean Code → Functionality → Consistency**, 그다음 Security / Testing / Performance / Documentation. 후순위 카테고리에서 좋은 점수를 받아도 4대 우선순위 중 하나라도 실패하는 변경은 수정 요청해야 한다.

## 심각도 범례

작성자가 머지 차단 사유와 후속 작업 가능 사유를 알 수 있도록 모든 발견에 아래 라벨 중 하나를 태그한다.

- **Blocker** — 머지 전 수정 필수. SOLID 위반, 보안 이슈, 깨진 기능, 데이터 race, 리소스 누수, 하드 한도 초과.
- **Major** — 이번 PR에서 수정. Clean Code 위반, Consistency 위반, 신규 코드의 누락된 테스트, 계획 없는 소프트 한도 초과.
- **Minor** — 후속 작업 허용. 문서 다듬기, 명명 취향, 성능 마이크로 최적화, 비필수 리팩토링.

확신이 없으면 위로 (Major → Blocker) 기본값을 잡는다.

## 코드 품질

### 크기 한도 (soft/hard 단계는 `code-thresholds.md` 참고)
- [ ] File size ≤ 300 LOC soft / ≤ 500 LOC hard
- [ ] Function size ≤ 50 LOC soft / ≤ 80 LOC hard
- [ ] Parameters ≤ 5 soft / ≤ 7 hard
- [ ] Cyclomatic complexity ≤ 10 soft / ≤ 15 hard
- [ ] Nesting depth ≤ 4 soft / ≤ 6 hard
- [ ] hard 한도 초과 시 분할/리팩토링; soft 초과 시 논의

### SOLID Principles
- [ ] **S**ingle Responsibility: 각 module/class/function은 정확히 한 가지 이유로 변경된다
- [ ] **O**pen/Closed: 안정된 기존 코드를 수정하지 않고 동작을 확장한다
- [ ] **L**iskov Substitution: 서브타입은 슈퍼타입의 계약을 지킨다 (예상치 못한 오버라이드 없음)
- [ ] **I**nterface Segregation: 호출자는 절대 사용하지 않는 메서드에 의존하지 않는다
- [ ] **D**ependency Inversion: 상위 모듈은 구체 구현이 아닌 추상(trait/interface)에 의존한다

### Clean Code
- [ ] 의도를 드러내는 이름 (`data`, `tmp`, 인덱스 외 단일 문자 루프 회피)
- [ ] 각 함수는 한 가지 일을 한 추상화 수준에서 수행한다
- [ ] Side effect (I/O, 네트워크, 공유 상태 mutation)는 경계 레이어로 격리된다
- [ ] 깊은 중첩보다 guard clause 선호
- [ ] 상수 심볼화 (매직 넘버/문자열 없음, 하드코딩된 경로 없음)
- [ ] Input → Processing → Return으로 구조화된 코드
- [ ] dead code 없음, 주석 처리된 블록 없음, 티켓 없는 `TODO` 없음

## 기능성 리뷰

- [ ] 요구사항을 end-to-end로 정확히 구현
- [ ] Edge case 처리됨 (empty, null, max, concurrent, partial failure)
- [ ] 에러 처리가 구체적이고 실행 가능 (컨텍스트를 삼키는 catch-all 없음)
- [ ] 무관한 모듈에 의도하지 않은 side effect 없음
- [ ] 리팩토링 시 이전 버전 대비 동작 동등성 검증됨

## 동시성 & 리소스 안전성

이 프로젝트는 Rust 스레드 + `mpsc` 채널 + 자식 프로세스를 사용한다. 승인 전에 모든 동시성 접점을 검토한다.

- [ ] 채널 라이프사이클 명시적 — `Sender`/`Receiver` drop 지점 식별, 종료 후 dangling producer 없음
- [ ] `cancel_rx` 협력적 취소 점검이 긴 단계 사이에 배치됨 (`source::sync_all_sources` 패턴 참고)
- [ ] `JoinHandle` 처리 명시적 — `join()`을 호출하거나, 의도적으로 detach하면서 그 이유를 한 줄 주석으로 명시
- [ ] `Arc<Mutex<...>>` 스코프가 필요 이상으로 넓지 않음; 잠금은 critical section 동안만 보유
- [ ] 파일 핸들 / temp dir이 Drop-guard 사용 (정리를 직접 소유하고 프로세스 종료에 의존하지 않음)
- [ ] 자식 프로세스의 라이프타임이 한정됨 — cancel 경로에서 timeout 또는 명시적 kill; 고아 좀비 없음
- [ ] 파이프를 읽는 spawned thread는 자식이 kill되면 자연스럽게 exit (자식 사망 후 blocking read 없음)
- [ ] TUI tick과 background thread 간 공유 상태는 raw mutable reference가 아닌 채널을 통한다

## 에러 처리 (Rust)

- [ ] `unwrap` / `expect`는 invariant가 지역적으로 증명 가능한 곳에서만; panic 메시지는 invariant를 명시
- [ ] 호출 경계에 `anyhow::Context` 부착되어 체인이 stack trace가 아닌 narrative로 읽힘
- [ ] 커스텀 에러 타입은 `thiserror`로 명시적 variant 사용; match granularity를 잃는 `Box<dyn Error>` 회피
- [ ] `Result`가 실제로 처리됨 — 외부 상태를 만지는 fallible call에 silent `let _ =` 없음
- [ ] TUI 코드는 절대 `eprintln!`을 호출하지 않음 (alternate screen 손상); 에러는 `status_message` 또는 typed channel을 통해 표면화
- [ ] 민감 데이터 (토큰, 자격 증명)는 표시 또는 로깅 전 에러 메시지에서 스크럽됨 (`sanitize_stderr` 패턴)

## 일관성 리뷰

- [ ] 프로젝트 코딩 컨벤션 준수 (naming, formatting, error pattern)
- [ ] 인접 코드와 동일한 솔루션 패턴 사용 (한 문제에 두 가지 방식 없음)
- [ ] 로깅 스타일, correlation ID, error 메시지 형태가 주변 모듈과 일치
- [ ] Naming convention (snake_case / camelCase / PascalCase)이 언어 관용구와 프로젝트 표준에 부합
- [ ] API / response 형태가 기존 스키마와 일치 (임시 필드 없음)
- [ ] 의존성 추가가 기존 스택에 부합 (같은 일을 하는 중복 라이브러리 없음)
- [ ] 문서 톤과 구조가 주변 문서와 일치

## 크로스 플랫폼 고려사항

이 프로젝트는 macOS, Linux, Windows에 배포된다 (Homebrew + Scoop + source). 경로, 명령어, 또는 file I/O를 다루는 변경은 세 플랫폼 모두에 대해 검토되어야 한다.

- [ ] 경로 결합은 `Path::join` / `PathBuf` 사용 — 하드코딩된 `/` 또는 `\` 구분자 없음; 문자열 결합 없음
- [ ] `canonicalize()` 사용은 symlink를 포함할 수 있는 경로 비교 시 (예: macOS `/tmp` → `/private/tmp`)
- [ ] MSYS / Cygwin path 형태 (`/c/Users/...`)는 Windows에서 표시 또는 file op 전에 `normalize_git_path`로 정규화
- [ ] Shell 명령은 `Command::new("tool")`로 직접 호출 — **never** `cmd /c ...` (Windows에서 shell injection 위험)
- [ ] 사용자 제공 명령 문자열은 `is_safe_command`를 통과 (Unix `&|><;`와 Windows `%^!` 메타문자 차단)
- [ ] Unix에서 실행 파일 생성 시 파일 권한이 명시적으로 설정됨 (`PermissionsExt::set_mode`); Windows는 안전하게 무시
- [ ] 비교 시 line ending 허용 (`normalize_line_endings`로 `\r` 제거하여 CRLF vs LF가 "Modified" 상태를 뒤집지 않음)
- [ ] Hook/statusline 경로는 `dest_dir.file_name()`에서 파생 — `~/.claude`와 `~/.codex` 타겟 모두 지원

## 보안 리뷰

- [ ] 코드에 시크릿 없음
- [ ] 입력 검증 및 정제됨
- [ ] SQL injection 취약점 없음
- [ ] XSS 취약점 없음
- [ ] 인증/인가 적용됨
- [ ] 전체 체크리스트는 `security-review` skill 참고

## 테스트 리뷰

- [ ] 신규 코드에 테스트 있음
- [ ] 버그 수정에 회귀 테스트 있음
- [ ] 테스트가 결정적임
- [ ] E2E에 success path와 failure path 있음
- [ ] 전체 체크리스트는 `tdd-workflow` skill 참고

## 성능 리뷰

- [ ] 명백한 성능 이슈 없음
- [ ] 데이터베이스 쿼리 최적화됨
- [ ] N+1 query 문제 없음
- [ ] 적절한 캐싱 고려됨

## 문서 리뷰

- [ ] 복잡한 로직이 문서화됨
- [ ] API 변경이 문서화됨
- [ ] 필요시 README 업데이트됨
- [ ] Breaking change 명시됨

## 리뷰어 액션

1. **Read**: 컨텍스트와 목적 이해
2. **Verify**: 요구사항 대비 점검
3. **Test**: 필요시 로컬에서 테스트 실행
4. **Comment**: 건설적 피드백 제공
5. **Approve/Request Changes**: 명확한 결정
