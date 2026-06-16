---
name: deploy-to-vercel
description: Deploy apps/sites to Vercel — interactive (vercel login) or token-based (VERCEL_TOKEN, CI/CD) auth; preview by default, production only when explicit. "deploy my app", "preview deployment". 버셀 배포, Vercel 배포, 토큰 배포, CI 배포.
keywords: [vercel, deploy, 배포, 버셀배포, preview, 프리뷰, token, 토큰, ci-cd, vercel-token]
metadata:
  author: vercel
  version: "3.1.0"
---

# Vercel 배포

어떤 프로젝트든 Vercel에 배포한다. **사용자가 production을 명시적으로 요청하지 않는 한 항상 preview로 배포한다** (production이 아님).

목표는 사용자를 최선의 장기 설정 상태로 이끄는 것이다: 프로젝트가 Vercel에 연결되어 git push 배포가 가능한 상태. 아래 모든 방법은 사용자를 그 상태에 더 가깝게 만든다.

## Step 1: 프로젝트 상태 수집

어떤 방법을 쓸지 결정하기 전에 네 가지 체크를 모두 실행한다:

```bash
# 1. Check for a git remote
git remote get-url origin 2>/dev/null

# 2. Check if locally linked to a Vercel project (either file means linked)
cat .vercel/project.json 2>/dev/null || cat .vercel/repo.json 2>/dev/null

# 3. Check if the Vercel CLI is installed and authenticated
vercel whoami 2>/dev/null

# 4. List available teams (if authenticated)
vercel teams list --format json 2>/dev/null
```

### 팀 선택

사용자가 여러 팀에 속해 있으면, 사용 가능한 팀 슬러그를 글머리 기호 목록으로 제시하고 어느 팀에 배포할지 묻는다. 사용자가 팀을 선택하면 즉시 다음 단계로 진행한다 — 추가 확인을 묻지 않는다.

이후의 모든 CLI 명령어 (`vercel deploy`, `vercel link`, `vercel inspect` 등) 에 `--scope`로 팀 슬러그를 전달한다:

```bash
vercel deploy [path] -y --no-wait --scope <team-slug>
```

프로젝트가 이미 연결되어 있으면 (`.vercel/project.json` 또는 `.vercel/repo.json` 존재), 해당 파일의 `orgId`가 팀을 결정한다 — 다시 묻지 않는다. 팀이 하나 (또는 개인 계정뿐) 이면, 프롬프트를 건너뛰고 직접 사용한다.

**`.vercel/` 디렉토리에 대해:** 연결된 프로젝트는 다음 중 하나를 갖는다:
- `.vercel/project.json` — `vercel link` (단일 프로젝트 연결) 가 생성. `projectId`와 `orgId` 포함.
- `.vercel/repo.json` — `vercel link --repo` (repo 기반 연결) 가 생성. `orgId`, `remoteName`, 그리고 디렉토리를 Vercel 프로젝트 ID에 매핑하는 `projects` 배열 포함.

둘 중 하나라도 있으면 프로젝트가 연결된 것이다. 둘 다 확인한다.

**금지:** 연결되지 않은 디렉토리에서 상태 감지를 위해 `vercel project inspect`, `vercel ls`, `vercel link`를 사용하지 말 것 — `.vercel/` config가 없으면, 인터랙티브하게 프롬프트를 띄우거나 (`--yes` 사용 시) 부수 효과로 조용히 링크해 버린다. 어디서나 안전하게 실행할 수 있는 것은 `vercel whoami`뿐이다.

## Step 2: 배포 방법 선택

### 연결됨 (`.vercel/` 존재) + git remote 있음 → Git Push

이상적인 상태. 프로젝트가 연결되어 있고 git 통합이 있다.

1. **푸시하기 전에 사용자에게 묻는다.** 명시적 승인 없이는 절대 푸시하지 말 것:
   ```
   This project is connected to Vercel via git. I can commit and push to
   trigger a deployment. Want me to proceed?
   ```

2. **Commit and push:**
   ```bash
   git add .
   git commit -m "deploy: <description of changes>"
   git push
   ```
   Vercel은 푸시로부터 자동 빌드한다. non-production 브랜치는 preview 배포를, production 브랜치 (보통 `main`) 는 production 배포를 받는다.

3. **Preview URL 가져오기.** CLI가 인증되어 있으면:
   ```bash
   sleep 5
   vercel ls --format json
   ```
   JSON 출력에는 `deployments` 배열이 있다. 가장 최신 항목의 `url` 필드가 preview URL이다.

   CLI가 인증되어 있지 않으면, Vercel 대시보드 또는 git provider의 commit status 체크에서 preview URL을 확인하라고 사용자에게 알린다.

---

### 연결됨 (`.vercel/` 존재) + git remote 없음 → `vercel deploy`

프로젝트는 연결되어 있지만 git repo가 없다. CLI로 직접 배포한다.

```bash
vercel deploy [path] -y --no-wait
```

`--no-wait`을 사용해 빌드 완료까지 차단하지 않고 즉시 배포 URL을 반환받는다 (빌드는 시간이 걸릴 수 있다). 그 후 배포 상태를 확인한다:

```bash
vercel inspect <deployment-url>
```

production 배포 (사용자가 명시적으로 요청한 경우만):
```bash
vercel deploy [path] --prod -y --no-wait
```

---

### 연결 안 됨 + CLI 인증됨 → 먼저 link, 그 다음 배포

CLI는 동작하지만 프로젝트가 아직 연결되지 않았다. 사용자를 최선의 상태로 이끌 기회다.

1. **사용자에게 어느 팀에 배포할지 묻는다.** Step 1의 팀 슬러그를 글머리 기호 목록으로 제시한다. 팀이 하나 (또는 개인 계정뿐) 이면 이 단계를 건너뛴다.

2. **팀이 선택되면 즉시 link로 진행한다.** 무엇이 일어날지 사용자에게 알리지만 별도의 확인은 묻지 않는다:
   ```
   Linking this project to <team name> on Vercel. This will create a Vercel
   project to deploy to and enable automatic deployments on future git pushes.
   ```

3. **git remote가 있으면**, 선택한 팀 스코프로 repo 기반 링크를 사용한다:
   ```bash
   vercel link --repo --scope <team-slug>
   ```
   git remote URL을 읽어 그 repo에서 배포하는 기존 Vercel 프로젝트와 매칭한다. `.vercel/repo.json`을 만든다. 이는 디렉토리명으로 매칭을 시도해 로컬 폴더와 Vercel 프로젝트명이 다를 때 자주 실패하는 `vercel link` (without `--repo`) 보다 훨씬 신뢰할 수 있다.

   **git remote가 없으면**, 표준 link로 폴백한다:
   ```bash
   vercel link --scope <team-slug>
   ```
   사용자에게 프로젝트 선택 또는 생성을 프롬프트한다. `.vercel/project.json`을 만든다.

4. **그 다음 사용 가능한 최선의 방법으로 배포한다:**
   - git remote가 있으면 → commit + push (위 git push 방법 참조)
   - git remote가 없으면 → `vercel deploy [path] -y --no-wait --scope <team-slug>`, 그 후 `vercel inspect <url>`로 상태 확인

---

### 연결 안 됨 + CLI 인증 안 됨 → 설치, 인증, link, 배포

Vercel CLI가 전혀 설정되지 않은 상태.

1. **CLI 설치 (아직 설치 안 된 경우):**
   ```bash
   npm install -g vercel
   ```

2. **인증:**
   ```bash
   vercel login
   ```
   사용자가 브라우저에서 인증을 완료한다. 로그인이 불가능한 비인터랙티브 환경이라면, 아래 **no-auth fallback**으로 건너뛴다.

3. **어느 팀에 배포할지 묻는다** — `vercel teams list --format json`의 팀 슬러그를 글머리 기호 목록으로 제시. 팀이 하나/개인 계정이면 건너뛴다. 선택되면 즉시 진행.

4. **선택한 팀 스코프로 프로젝트를 link한다** (git remote가 있으면 `--repo` 사용, 없으면 plain `vercel link`):
   ```bash
   vercel link --repo --scope <team-slug>   # if git remote exists
   vercel link --scope <team-slug>          # if no git remote
   ```

5. **배포** 사용 가능한 최선의 방법으로 (remote가 있으면 git push, 그렇지 않으면 `vercel deploy -y --no-wait --scope <team-slug>` 후 `vercel inspect <url>`로 상태 확인).

---

### No-Auth Fallback — claude.ai sandbox

**사용 시점:** claude.ai sandbox에서 CLI를 설치하거나 인증할 수 없을 때의 최후의 수단. 인증이 필요하지 않다 — **Preview URL** (라이브 사이트) 과 **Claim URL** (Vercel 계정으로 이전) 을 반환한다.

```bash
bash /mnt/skills/user/deploy-to-vercel/resources/deploy.sh [path]
```

**Arguments:**
- `path` - 배포할 디렉토리 또는 `.tgz` 파일 (기본값: 현재 디렉토리)

**Examples:**
```bash
# Deploy current directory
bash /mnt/skills/user/deploy-to-vercel/resources/deploy.sh

# Deploy specific project
bash /mnt/skills/user/deploy-to-vercel/resources/deploy.sh /path/to/project

# Deploy existing tarball
bash /mnt/skills/user/deploy-to-vercel/resources/deploy.sh /path/to/project.tgz
```

스크립트는 `package.json`에서 프레임워크를 자동 감지하고, 프로젝트를 패키징하고 (`node_modules`, `.git`, `.env` 제외), 업로드하고, 빌드 완료를 기다린다.

**사용자에게 알린다:** "Your deployment is ready at [previewUrl]. Claim it at [claimUrl] to manage your deployment."

---

### No-Auth Fallback — Codex sandbox

**사용 시점:** CLI가 인증되지 않을 수 있는 Codex sandbox에서. Codex는 기본적으로 sandboxed 환경에서 실행된다 — 먼저 CLI를 시도하고, 인증 실패 시 deploy 스크립트로 폴백한다.

1. **Vercel CLI가 설치되어 있는지 확인** (이 체크는 권한 상승 불필요):
   ```bash
   command -v vercel
   ```

2. **`vercel`이 설치되어 있으면**, CLI로 배포 시도:
   ```bash
   vercel deploy [path] -y --no-wait
   ```

3. **`vercel`이 설치되지 않았거나 CLI가 "No existing credentials found"로 실패하면**, fallback 스크립트 사용:
   ```bash
   skill_dir="<path-to-skill>"

   # Deploy current directory
   bash "$skill_dir/resources/deploy-codex.sh"

   # Deploy specific project
   bash "$skill_dir/resources/deploy-codex.sh" /path/to/project

   # Deploy existing tarball
   bash "$skill_dir/resources/deploy-codex.sh" /path/to/project.tgz
   ```

스크립트가 프레임워크 감지, 패키징, 배포를 처리한다. 빌드 완료를 기다리고 `previewUrl`과 `claimUrl`이 포함된 JSON을 반환한다.

**사용자에게 알린다:** "Your deployment is ready at [previewUrl]. Claim it at [claimUrl] to manage your deployment."

**상승된 네트워크 접근:** sandboxing이 네트워크 호출을 차단하는 경우 (`sandbox_permissions=require_escalated`) 에만 실제 deploy 명령을 escalate한다. `command -v vercel` 체크는 escalate **하지 말 것**.

---

## 에이전트별 노트

### Claude Code / 터미널 기반 에이전트

전체 셸 접근 권한이 있다. `/mnt/skills/` 경로를 사용하지 말 것. CLI를 직접 사용해 위 결정 흐름을 따른다.

no-auth fallback은 skill의 설치 위치에서 deploy 스크립트를 실행한다:
```bash
bash ~/.claude/skills/deploy-to-vercel/resources/deploy.sh [path]
```
경로는 사용자가 skill을 어디에 설치했는지에 따라 달라질 수 있다.

### Sandboxed 환경 (claude.ai)

`vercel login`이나 `git push`를 실행할 수 없을 가능성이 높다. **no-auth fallback — claude.ai sandbox**로 직접 간다.

### Codex

Codex는 sandbox에서 실행된다. CLI가 사용 가능한지 먼저 확인하고, 그 후 deploy 스크립트로 폴백한다. **no-auth fallback — Codex sandbox**로 간다.

---

## 출력

항상 사용자에게 배포 URL을 보여준다.

- **Git push:** `vercel ls --format json`을 사용해 preview URL을 찾는다. CLI가 인증되지 않았으면 사용자에게 Vercel 대시보드 또는 commit status 체크를 확인하라고 알린다.
- **CLI deploy:** `vercel deploy --no-wait`이 반환한 URL을 보여준다. `vercel inspect <url>`로 빌드 상태를 확인하고 사용자에게 보고한다.
- **No-auth fallback:** preview URL과 claim URL을 모두 보여준다:
  ```
  Deployment successful!

  Preview URL: https://my-app-abc123.vercel.app
  Claim URL:   https://vercel.com/claim-deployment?code=...

  View your site at the Preview URL.
  To transfer this deployment to your Vercel account, visit the Claim URL.
  ```

배포된 URL을 동작 검증을 위해 curl이나 fetch **하지 말 것**. 그냥 링크를 반환한다.

---

## 트러블슈팅

### Network Egress Error (claude.ai)

claude.ai의 네트워크 제한으로 배포가 실패하면, 사용자에게 알린다:

```
Deployment failed due to network restrictions. To fix this:

1. Go to https://claude.ai/settings/capabilities
2. Add *.vercel.com to the allowed domains
3. Try deploying again
```

### Escalated Network Access (Codex / sandboxed 환경)

sandboxed 환경에서 네트워크 이슈 (timeout, DNS 에러, connection reset) 로 배포가 실패하면, 상승된 권한 (`sandbox_permissions=require_escalated`) 으로 재실행한다. 초기 CLI 가용성 체크는 escalate 하지 말 것 — 실제 deploy 명령만 escalate한다.

사용자에게 안내 예시:
```
The deploy needs escalated network access to deploy to Vercel. I can rerun
the command with escalated permissions — want me to proceed?
```

### CLI Auth Failure

`vercel login` 또는 `vercel deploy`가 인증 에러로 실패하면, no-auth deploy 스크립트로 폴백한다 (환경에 따라 claude.ai 또는 Codex 변형).

---

## Token 기반 인증 (비인터랙티브)

`vercel login`을 사용할 수 없는 CI/CD 또는 비인터랙티브 환경용. 브라우저 인증 대신 `VERCEL_TOKEN` 환경 변수를 사용한다.

### Vercel Token 위치

다음 시나리오를 순서대로 진행한다:

**A) `VERCEL_TOKEN`이 이미 환경에 있음**
```bash
printenv VERCEL_TOKEN
```

**B) `.env` 파일의 `VERCEL_TOKEN`에 있음**
```bash
grep '^VERCEL_TOKEN=' .env 2>/dev/null
export VERCEL_TOKEN=$(grep '^VERCEL_TOKEN=' .env | cut -d= -f2-)
```

**C) `.env`에서 다른 이름의 토큰** (Vercel 토큰은 `vca_`로 시작)
```bash
grep -i 'vercel' .env 2>/dev/null
export VERCEL_TOKEN=$(grep '^<VARIABLE_NAME>=' .env | cut -d= -f2-)
```

**D) 토큰을 찾을 수 없음** — 사용자에게 vercel.com/account/tokens에서 토큰을 만들도록 요청

### Critical: `--token` 플래그 사용 금지

```bash
# Bad — token visible in shell history and process listings
vercel deploy --token "vca_abc123"

# Good — CLI reads VERCEL_TOKEN from environment
export VERCEL_TOKEN="vca_abc123"
vercel deploy
```

### 프로젝트 및 팀 위치 (`.vercel/` 디렉토리 건너뛰기)

```bash
printenv VERCEL_PROJECT_ID
printenv VERCEL_ORG_ID
```

둘 다 설정되어 있으면 CLI가 자동으로 사용하고 모든 `.vercel/` 디렉토리를 건너뛴다:
```bash
export VERCEL_ORG_ID="<org-id>"
export VERCEL_PROJECT_ID="<project-id>"
```

**반드시 함께 설정** — 하나만 설정하면 에러가 발생한다.

프로젝트 URL에서 팀 슬러그 추출:
```bash
# e.g. "my-team" from "https://vercel.com/my-team/my-project"
echo "$PROJECT_URL" | sed 's|https://vercel.com/||' | cut -d/ -f1
```

### 빠른 토큰 배포 (link 불필요)

```bash
vercel deploy -y --no-wait --scope <team-slug>
```

### 환경 변수 관리 (Token 사용)

```bash
echo "value" | vercel env add VAR_NAME --scope <team-slug>
echo "value" | vercel env add VAR_NAME production --scope <team-slug>
vercel env ls --scope <team-slug>
vercel env pull --scope <team-slug>
vercel env rm VAR_NAME --scope <team-slug> -y
```

### Token 인증 트러블슈팅

**Token을 찾을 수 없음:**
```bash
printenv | grep -i vercel
grep -i vercel .env 2>/dev/null
```

**인증 에러** (`Authentication required`):
- 토큰이 만료되었거나 유효하지 않을 수 있음
- 검증: `vercel whoami` (env에서 `VERCEL_TOKEN` 사용)
- 사용자에게 새 토큰 요청

**잘못된 팀:**
```bash
vercel whoami --scope <team-slug>
```
