# 배포 설정의 동작 영역

`src/` 는 인스톨러가 ~/.claude 와 ~/.codex 로 복사하는 산문 설정이다. 제보는 파일 이름이 아니라 모델의 행동으로 들어온다. 그래서 항목은 동작 영역 단위로 나누고 그 동작을 정하는 파일을 가리킨다. 스킬 28개 · 커맨드 26개 · 에이전트 8개를 하나씩 찾을 때는 `component-lookup` 의 규칙을 쓴다. 이 영역에는 화면 문구가 없으므로 UI 문구 줄에는 모델 출력이나 설정에 실제로 박힌 문자열을 적는다.

### `commit-push-gating`: 커밋 · 푸시 · PR 은 명시 요청 때만

- **위치**: Claude Code · Codex 대화, git 작업 직후
- **별칭**: 커밋, 푸시, PR, 묻지 않고 커밋, 자동 커밋, Co-Authored-By, 티켓 번호, /commit 이 커밋함
- **UI 문구**: `"Co-Authored-By"`, `"Generated with Claude Code"`, `"<type>: [<ticket>] <title>"`
- **컨트롤**: `/commit` → `src/commands/commit.md`, `/pull-request` → `src/commands/pull-request.md`, `/upstream-pr` → `src/commands/upstream-pr.md`
- **코드 경로**:
  - 상시 규칙: `src/CLAUDE.md` 의 Absolute commit and push rules 와 Git and change safety, Codex 는 `src/AGENTS.md` 의 Git and change safety
  - 상세 규칙: `src/skills/commit-rules/SKILL.md` 의 Ticket Number 와 Mandatory Rules, PR 은 `src/skills/pull-request/SKILL.md` 의 Open it, only when asked
  - 데이터: 커밋 · PR 서명 끄기는 `src/settings.json` → `attribution`
- **테스트**: `src/skills/pull-request/evals/evals.json`
- **함정**: `/commit` 을 쳤더니 묻지 않고 커밋함 → 방금 친 슬래시 커맨드가 그 커밋 하나에 대한 명시 요청이다, push 와 PR 은 따로 승인받는다, 근거: `src/CLAUDE.md` → `authorizes`
- **함정**: 커밋 제목에 남의 티켓 번호가 붙음 → 브랜치에 없는 번호를 git log 에서 빌려 썼다, 지금은 브랜치가 정하고 없으면 접두사를 뺀다, 근거: `5fb1c5d`
- **함정**: 티켓이 없는 저장소에서도 `[PP-XXXX]` 같은 자리표시자를 요구함 → 형식이 고정 프로젝트 키를 요구했다, 근거: `b399c3a`
- **함정**: 커밋 메시지나 PR 에 Co-Authored-By 가 붙음 → 서명 끄기는 settings.json 의 `attribution` 빈 문자열에 있어서 Config 탭 settings.json 을 설치하지 않으면 적용되지 않는다, 근거: `src/settings.json` → `attribution`
- **공유 의존**: `settings-merge`
- **관련**: `review-gate`, `install-manifest`

### `response-language`: 응답 언어와 출력 서식

- **위치**: 모든 대화 응답
- **별칭**: 영어로 답함, 한국어 응답, 응답 언어, 출력 스타일, 서식
- **UI 문구**: `"Default response language: Korean"`, `"respond in Korean, user readability first"`
- **컨트롤**: Styles 탭에서 `hibi_default` 를 s 로 기본 지정, `default-style-statusline` 참고
- **코드 경로**:
  - 상시 규칙: `src/CLAUDE.md` 의 Thinking and response language policy
  - 출력 스타일: `src/output-styles/hibi_default.md` 가 언어와 마크다운 서식을 정한다, settings.json 의 `outputStyle` 로 켠다
  - Codex: `src/AGENTS.md` 의 Language settings 와 Output formatting, Codex 에는 이 파일이 유일한 사본이다
- **테스트**: none
- **함정**: Codex 가 영어로 답함 → Codex 대상은 skills 와 AGENTS.md 만 설치되므로 Config 탭에서 AGENTS.md 를 설치하지 않으면 언어 규칙이 없다, 근거: `tools/installer/src/fs/scanner/components.rs` → `add_config_files`
- **함정**: hibi_default 를 설치했는데 출력 스타일이 안 바뀜 → 이미 다른 `outputStyle` 이 있으면 자동 등록하지 않는다, 근거: `tools/installer/src/fs/installer/settings.rs` → `register_output_style_in_settings`
- **함정**: 응답은 한국어인데 코드 주석은 영어 → 주석 언어는 응답 언어가 아니라 대상 파일의 기존 주석 언어를 따른다, 근거: `src/CLAUDE.md` → `Language`
- **공유 의존**: `settings-merge`
- **관련**: `comment-rules`, `default-style-statusline`

### `comment-rules`: 코드 주석 규칙

- **위치**: 모델이 쓰거나 고친 코드의 주석
- **별칭**: 주석, 주석이 너무 많음, 주석이 없음, 주석 언어, 오래된 주석
- **UI 문구**: `"Code is the spec."`, `"No over-commenting"`
- **컨트롤**: 없음
- **코드 경로**:
  - 상시 규칙: `src/CLAUDE.md` 의 Absolute comment rules
  - 상세 규칙: `src/skills/coding-standards/SKILL.md` 의 Comments 와 Comment maintenance
  - 리뷰 쪽: `src/agents/code-reviewer.md` 가 diff 로 무효가 된 주석을 이 변경의 결과로 본다
- **테스트**: none
- **함정**: 모델이 주석을 거의 달지 않음 → 기본값이 주석 없음이고 이유만 적게 한다, 근거: `src/CLAUDE.md` → `default`
- **함정**: 이름을 바꿨는데 다른 파일의 주석이 옛 이름 그대로 → 규칙은 옛 이름을 트리 전체에서 grep 하라고 요구한다, 지켜지지 않으면 리뷰 게이트가 잡아야 한다, 근거: `src/CLAUDE.md` → `grep`
- **공유 의존**: `src/skills/coding-standards/SKILL.md`
- **관련**: `review-gate`, `response-language`

### `review-gate`: 작업 후 리뷰 게이트와 code-reviewer 필터

- **위치**: 코드 변경 뒤 완료 보고 전, `code-reviewer` 에이전트나 `/code-review`
- **별칭**: 리뷰, 리뷰어, 잔소리, 닛픽, nit, 지적이 너무 많음, 리뷰가 안 돌아옴, code-reviewer
- **UI 문구**: `"[APPROVE]"`, `"Dismissed"`, `"Filter Before Reporting"`
- **컨트롤**: `/code-review` → `src/commands/code-review.md`
- **코드 경로**:
  - 상시 규칙: `src/CLAUDE.md` 의 Verify before completion 안 Mandatory post-work review
  - 에이전트: `src/agents/code-reviewer.md` 의 Filter Before Reporting 과 Approval Criteria
  - 기준표: `src/skills/coding-standards/references/review-checklist.md`
- **테스트**: none
- **함정**: 리뷰가 사소한 스타일 지적을 경고로 쏟아냄 → 필터는 LOW 지적만 남으면 APPROVE 하고 제안으로만 나열하게 한다, 이 필터가 없는 옛 설치본이면 Agents 탭에 modified 로 보인다, 근거: `src/agents/code-reviewer.md` → `Nits`
- **함정**: 내가 건드리지 않은 문제를 리뷰가 지적함 → 기존 스타일 문제는 빼지만 건드린 파일의 보안 · 정확성 결함은 그대로 보고한다, 근거: `src/agents/code-reviewer.md` → `Caused`
- **함정**: 팀으로 띄운 리뷰어가 결과를 못 돌려줌 → 도구 허용 목록에 SendMessage 가 없었다, 근거: `11b8d52`
- **공유 의존**: `src/skills/coding-standards/SKILL.md`
- **관련**: `verification-phase7`, `comment-rules`

### `verification-phase7`: 검증 루프와 실제 산출물 확인

- **위치**: 완료 보고 직전, `/verify` 나 verification-loop 스킬
- **별칭**: 검증, verify, READY 안 나옴, PARTIAL, INCONCLUSIVE, 빌드 확인
- **UI 문구**: `"Overall: READY"`, `"Overall: PARTIAL"`, `"Observe the Real Artifact"`
- **컨트롤**: `/verify` → `src/commands/verify.md`, `/build-fix` → `src/commands/build-fix.md`
- **코드 경로**:
  - 스킬: `src/skills/verification-loop/SKILL.md` 의 Phase 1 부터 7
  - 커맨드: `src/commands/verify.md` 는 Phase 1 부터 6 만 돈다
- **테스트**: none
- **함정**: `/verify` 가 항상 PARTIAL 로 끝남 → 커맨드는 프록시 단계 1 부터 6 만 돌고 READY 는 Phase 7 실제 산출물 확인이 있어야 한다, 근거: `src/commands/verify.md` → `PARTIAL`
- **함정**: 확인을 못 한 항목이 NOT READY 로 보고됨 → 실행할 수 없는 확인은 INCONCLUSIVE 이고 INCONCLUSIVE 는 NOT READY 다, 근거: `src/skills/verification-loop/SKILL.md` → `INCONCLUSIVE`
- **공유 의존**: 없음
- **관련**: `review-gate`

### `skill-triggering`: 스킬 자동 트리거와 description 예산

- **위치**: 대화 중 스킬이 자동으로 로드되는지 여부, 측정은 `src/skills/eval-harness/scripts` 의 두 스크립트
- **별칭**: 스킬이 안 뜸, 스킬 트리거, description, 예산, 트리거 평가, /learn
- **UI 문구**: `"within budget, {budget - total} chars spare"`, `"OVER BUDGET by {total - budget} chars."`, `"Truncation is all-or-nothing per skill"`
- **컨트롤**: `python src/skills/eval-harness/scripts/skill_budget.py src/skills`, `/eval` → `src/commands/eval.md`, `/learn` → `src/commands/learn.md`
- **코드 경로**:
  - 데이터: 각 스킬의 frontmatter `description`, 위치는 `component-lookup`
  - 측정: `src/skills/eval-harness/scripts/skill_budget.py` → `BUDGET_FRACTION`, 트리거 측정은 `src/skills/eval-harness/scripts/trigger_eval.py` 가 중첩 `claude -p` 를 띄워 Skill 호출을 본다
  - 평가셋: 스킬별 evals 디렉터리의 trigger-eval.json, 예 `src/skills/feature-map/evals/trigger-eval.json`
- **테스트**: `src/skills/feature-map/evals/trigger-eval.json`, `src/skills/technical-writing/evals/trigger-eval.json`, `src/skills/pull-request/evals/trigger-eval.json`
- **함정**: 특정 스킬이 전혀 트리거되지 않음 → 목록 전체가 문자 예산을 넘으면 넘친 스킬은 이름만 남고 description 이 통째로 빠진다, 근거: `src/skills/eval-harness/scripts/skill_budget.py` → `BUDGET_FRACTION`
- **함정**: description 을 고쳤는데 트리거 평가 결과가 그대로 → 평가는 ~/.claude/skills 의 설치본을 읽으므로 먼저 설치해야 한다, 근거: `src/skills/eval-harness/SKILL.md` → `installed`
- **함정**: 하네스가 죽었는데 FAIL 로 집계됨 → 처리 안 된 예외가 exit 1 로 끝났다, 지금은 exit 4 다, 근거: `582b99b`
- **함정**: `/learn` 으로 저장한 패턴이 한 번도 로드되지 않음 → 예전에는 로드되지 않는 평면 파일에 썼다, 근거: `b22fb9a`
- **공유 의존**: `component-lookup`
- **관련**: `prose-lint`

### `prose-lint`: 산문 구두점 린트

- **위치**: `python tools/lint-prose.py` 실행, 기본 대상은 src · docs · README.md
- **별칭**: 대시, 괄호, 줄표, lint-prose, 문체 린트, 산문 규칙
- **UI 문구**: `"em dash"`, `"en dash"`, `"hyphen as dash"`, `"parenthesis"`
- **컨트롤**: 없음, 위반이 있으면 exit 1
- **코드 경로**:
  - 검사: `tools/lint-prose.py` → `CHECKS`, 대상 루트는 `tools/lint-prose.py` → `DEFAULT_ROOTS`
  - 규칙 원문: `src/skills/technical-writing/SKILL.md` 의 Punctuation
- **테스트**: none
- **함정**: 문장 중간의 ` - ` 가 걸리는데 목록 맨 앞의 `-` 는 통과 → 줄 첫 목록 마커만 지운 뒤 나머지에 hyphen as dash 검사를 한다, 근거: `tools/lint-prose.py` → `prose_of`
- **함정**: 괄호를 백틱 안에 넣으면 통과 → 인라인 코드 · 코드 펜스 · 링크 대상 · frontmatter 키는 산문이 아니라 검사에서 뺀다, 근거: `tools/lint-prose.py` → `INLINE_CODE`
- **공유 의존**: 없음
- **관련**: `skill-triggering`

### `component-lookup`: 커맨드 · 스킬 · 에이전트 파일을 찾는 규칙

- **위치**: `src/` 트리, 설치 후에는 ~/.claude 와 ~/.codex 아래 같은 상대 경로
- **별칭**: 커맨드 파일 어디, 스킬 파일 위치, 에이전트 정의, -ko 파일, 한국어 미러
- **UI 문구**: `"-ko"`
- **컨트롤**: 없음
- **코드 경로**:
  - 규칙: `/NAME` 커맨드는 src/commands/NAME.md, 스킬은 src/skills/NAME/SKILL.md, 에이전트는 src/agents/NAME.md 에 있다. 각각 `-ko.md` 접미사의 한국어 미러가 옆에 있지만 설치되지 않는다
  - 목록: `docs/README.md` 의 에이전트 · 커맨드 · 스킬 표, 위치 표는 `docs/INDEX.md`
  - 스캔: `tools/installer/src/fs/scanner/components.rs` → `scan_components`
- **테스트**: `tools/installer/src/fs/scanner/components.rs`
- **함정**: `-ko.md` 를 고쳤는데 동작이 안 바뀜 → 스캐너가 `-ko` 파일을 건너뛰고 `package.sh` 도 릴리즈에서 지운다, 영어 원본을 같이 고쳐야 한다, 근거: `tools/installer/src/fs/scanner/components.rs` → `scan_directory`
- **함정**: Codex 에서 커맨드나 에이전트를 쓰라고 안내함 → Codex 대상은 skills 와 AGENTS.md 만 받는다, AGENTS.md 의 그런 안내는 고쳤다, 근거: `b22fb9a`
- **함정**: `src/hooks` 의 훅을 설치할 수 없음 → 모든 hook.yaml 이 deprecated 라 인스톨러가 설치를 거부하고 기존 설치본은 지운다, 근거: `tools/installer/src/fs/installer/mod.rs` → `install_component`
- **공유 의존**: `tools/installer/src/fs/scanner/mod.rs`
- **관련**: `component-list`, `skill-triggering`
