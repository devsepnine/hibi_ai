# Claude Code 상태줄

### `statusline-render`: 상태줄 바이너리의 출력과 줄바꿈

- **위치**: Claude Code 입력창 아래 상태줄. 사용자·경로, 브랜치, 모델과 시각을 먼저 표시한다. 입력에 따라 컨텍스트, 사용량 한도, 버전과 todos를 다음 행에 추가하고 폭에 맞춰 줄을 나눈다.
- **별칭**: 상태줄, 상태표시줄, statusline, 토큰 사용량, ctx, 5h, 7d, todos, 줄바꿈, 한글 폭, 상태줄이 안 나옴
- **UI 문구**: `"ctx:"`, `"5h:"`, `"7d:"`, `"todos:"`, `"Unknown"`, `"unknown"`
- **컨트롤**: 없음, Claude Code 가 stdin 으로 JSON 한 줄을 넘기면 실행된다
- **코드 경로**:
  - 상태 / 핸들러: `tools/statusline/src/main.rs` → `main` 이 stdin 첫 줄을 읽어 `StatusInput` 으로 파싱한다
  - 상태 / 핸들러: `tools/statusline/src/main.rs` → `extract_values`, git 정보는 `tools/statusline/src/main.rs` → `get_git_info`
  - UI: `tools/statusline/src/render.rs` → `line1_segments`, `tools/statusline/src/render.rs` → `line2_segments`
  - UI: `tools/statusline/src/render.rs` → `wrap_segments` 가 `tools/statusline/src/render.rs` → `terminal_width` 폭에 맞춰 조각 단위로 줄을 나눈다
  - 데이터: 배포 바이너리는 `src/statusline/statusline.exe`, `src/statusline/statusline_macos`, `src/statusline/statusline_linux` 이고 `tools/statusline/build.sh` 가 만든다. 설치와 settings.json 등록은 `default-style-statusline`
- **테스트**: `tools/statusline/src/render.rs`, `tools/statusline/src/main.rs`
- **함정**: 상태줄이 통째로 빈칸 → stdin 이 비었거나 JSON 파싱에 실패하면 아무것도 찍지 않고 끝낸다, 근거: `tools/statusline/src/main.rs` → `main`
- **함정**: Windows에서 상태줄이 안 나옴 → 빌드가 오래되면 입력 종료를 기다릴 수 있다. 현재 바이너리는 첫 줄만 읽으며 기본 설정은 Git Bash에서 stdin을 파이프로 전달한다, 근거: `5c584c7`, `tools/installer/src/fs/installer/settings.rs` → `statusline_command`
- **함정**: 창이 좁아도 줄바꿈이 안 됨 → `COLUMNS`가 없거나 유효한 숫자가 아니거나 2열 이하이면 줄바꿈을 건너뛴다, 근거: `tools/statusline/src/render.rs` → `terminal_width`
- **함정**: 긴 경로나 브랜치가 화면 밖으로 넘침 → 폭보다 긴 조각은 자르지 않고 한 줄에 그대로 둔다, 근거: `tools/statusline/src/render.rs` → `wrap_segments`
- **함정**: todos 숫자가 줄지 않고 계속 커짐 → transcript 전체에서 `"type":"todo"` 등장 횟수를 센다, 근거: `tools/statusline/src/main.rs` → `count_todos`
- **함정**: 한글 경로에서 줄바꿈 위치가 어긋남 → 폭을 ANSI 를 뺀 유니코드 표시 폭으로 잰다, 이 계산이 바뀌면 깨진다, 근거: `tools/statusline/src/render.rs` → `visible_width`
- **함정**: 상태줄 소스를 고쳐도 릴리즈에 반영 안 됨 → 릴리즈 워크플로는 인스톨러만 빌드하고 상태줄은 커밋된 `src/statusline` 바이너리를 그대로 싣는다. 소스 테스트는 릴리스의 `verify`에서 실행한다, 근거: `.github/workflows/release.yml` → `build`
- **공유 의존**: unicode-width, chrono 크레이트
- **관련**: `default-style-statusline`, `packaging`

소스 테스트는 `cargo test --locked --manifest-path tools/statusline/Cargo.toml`로 실행한다. 실제 출력의 줄바꿈을 확인하려면 저장소 루트에서 샘플 JSON을 전달한다.

```bash
printf '%s\n' '{"cwd":"/tmp/한글 프로젝트","model":{"display_name":"Claude"}}' | COLUMNS=24 cargo run --locked --quiet --manifest-path tools/statusline/Cargo.toml
```

배포 파일을 바꿀 때는 세 플랫폼 바이너리와 소스 변경을 함께 확인한다. [운영 가이드](../RUNBOOK.md)
