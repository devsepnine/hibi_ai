# Hooks System

## Hook 타입

- **PreToolUse**: 도구 실행 전 (검증, 파라미터 수정)
- **PostToolUse**: 도구 실행 후 (자동 포맷, 점검)
- **Stop**: 세션 종료 시 (최종 검증)

## 현재 등록된 Hook (in ~/.claude/settings.json)

### PreToolUse
- **tmux reminder**: 장기 실행 명령어(npm, pnpm, yarn, cargo 등)에 tmux 사용을 제안
- **git push review**: push 전에 Zed로 리뷰 화면을 연다
- **doc blocker**: 불필요한 .md/.txt 파일 생성을 차단

### PostToolUse
- **PR creation**: PR URL과 GitHub Actions 상태를 로깅
- **Prettier**: edit 후 JS/TS 파일을 자동 포맷
- **TypeScript check**: .ts/.tsx 파일 편집 후 tsc 실행
- **console.log warning**: 편집된 파일 안의 console.log에 대해 경고

### Stop
- **console.log audit**: 세션 종료 전에 수정된 모든 파일을 console.log 기준으로 점검

## 자동 수락 권한

신중하게 사용한다:
- 신뢰할 수 있고 잘 정의된 계획에만 활성화
- 탐색적 작업에서는 비활성화
- dangerously-skip-permissions 플래그는 절대 사용하지 않는다
- 대신 `~/.claude.json`의 `allowedTools`를 설정한다

## TodoWrite 모범 사례

TodoWrite 도구를 사용해:
- 다단계 작업의 진행 상황을 추적한다
- 지시 사항에 대한 이해를 검증한다
- 실시간 방향 전환을 가능하게 한다
- 세분화된 구현 단계를 보여준다

Todo 목록이 드러내는 것:
- 순서가 어긋난 단계
- 누락된 항목
- 불필요한 추가 항목
- 잘못된 단위(granularity)
- 잘못 해석된 요구사항
