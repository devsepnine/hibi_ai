---
description: Iteratively fix TypeScript and build errors. Parses error output, applies minimal fixes one at a time, verifies after each. Stops on regression.
allowed-tools: Bash, Read, Edit, Grep
model: haiku
effort: low
---

# Build and Fix

**build-error-resolver** 에이전트를 디스패치하여 TypeScript / 빌드 오류를 점진적으로 수정한다.

## Invoke

빌드를 실행하고(`npm run build` / `pnpm build`) 오류 출력을 `build-error-resolver`에 전달한다. 에이전트는 한 번에 하나의 오류만 minimal diff로 수정하고 매번 검증한다.

## Command-specific stop gates

- 수정이 새로운 오류를 유발하면(regression) 중단한다.
- 동일한 오류가 3회 시도 후에도 지속되면 중단한다.
- 사용자가 일시 중지를 요청하면 중단한다.

전체 diagnostic command, 오류 패턴 표, minimal-diff 전략, safety guard, report format은 `build-error-resolver` 에이전트(`src/agents/build-error-resolver.md`)에 있다. 이를 source of truth로 따른다.
