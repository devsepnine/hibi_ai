---
name: deploy-to-vercel
description: Deploy to Vercel, interactive or VERCEL_TOKEN auth, preview by default, production only when explicit. Use when deploying a site. 버셀 배포, Vercel 배포, 프리뷰 배포, CI 배포.
metadata:
  author: vercel
  version: "3.1.0"
---

# Vercel 배포

어떤 프로젝트든 Vercel에 배포한다. **사용자가 production을 명시적으로 요청하지 않는 한 항상 preview로 배포한다**, production이 아님.

사용자가 지정한 프로젝트와 배포 환경을 사용한다.

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

사용자가 여러 팀에 속해 있으면, 사용 가능한 팀 슬러그를 글머리 기호 목록으로 제시하고 어느 팀에 배포할지 묻는다. 사용자가 팀을 선택하면 즉시 다음 단계로 진행한다, 추가 확인을 묻지 않는다.

이후의 모든 CLI 명령어, `vercel deploy`, `vercel link`, `vercel inspect` 등, 에 `--scope`로 팀 슬러그를 전달한다:

```bash
vercel deploy [path] --environment=preview -y --no-wait --scope <team-slug>
```

프로젝트가 이미 연결되어 있으면, `.vercel/project.json` 또는 `.vercel/repo.json` 존재, 해당 파일의 `orgId`가 팀을 결정한다, 다시 묻지 않는다. 팀이 하나, 또는 개인 계정뿐, 이면, 프롬프트를 건너뛰고 직접 사용한다.

**`.vercel/` 디렉토리에 대해:** 연결된 프로젝트는 다음 중 하나를 갖는다:
- `.vercel/project.json`, `vercel link`, 단일 프로젝트 연결, 가 생성. `projectId`와 `orgId` 포함.
- `.vercel/repo.json`, `vercel link --repo`, repo 기반 연결, 가 생성. `orgId`, `remoteName`, 그리고 디렉토리를 Vercel 프로젝트 ID에 매핑하는 `projects` 배열 포함.

둘 중 하나라도 있으면 프로젝트가 연결된 것이다. 둘 다 확인한다.

**금지:** 연결되지 않은 디렉토리에서 상태 감지를 위해 `vercel project inspect`, `vercel ls`, `vercel link`를 사용하지 말 것, `.vercel/` config가 없으면, 인터랙티브하게 프롬프트를 띄우거나, `--yes` 사용 시, 부수 효과로 조용히 링크해 버린다. 어디서나 안전하게 실행할 수 있는 것은 `vercel whoami`뿐이다.

## Step 2: 배포 방법 선택

### 연결된 프로젝트

어느 브랜치에서든 현재 작업 내용의 프리뷰를 배포할 때는 CLI를 사용한다. 사용자가 Git 배포를 지정하면 Vercel에서 프로젝트의 Git 연동과 프로덕션 브랜치를 먼저 확인한다. 프로덕션 브랜치에 푸시하려면 프로덕션 배포 요청과 해당 푸시 승인이 필요하다. Git으로 배포한 결과는 프로젝트, 배포 환경, 커밋 SHA가 일치하는 항목으로 확인한다.

```bash
vercel deploy [path] --environment=preview -y --no-wait
```

`--no-wait`을 사용해 빌드 완료까지 차단하지 않고 즉시 배포 URL을 반환받는다, 빌드는 시간이 걸릴 수 있다. 그 후 배포 상태를 확인한다:

```bash
vercel inspect <deployment-url> --wait --timeout=5m
```

production 배포, 사용자가 명시적으로 요청한 경우만:
```bash
vercel deploy [path] --prod -y --no-wait
```

---

### 연결 안 됨 + CLI 인증됨 → 먼저 link, 그 다음 배포

CLI는 동작하지만 프로젝트가 아직 연결되지 않았다. 사용자를 최선의 상태로 이끌 기회다.

1. **사용자에게 어느 팀에 배포할지 묻는다.** Step 1의 팀 슬러그를 글머리 기호 목록으로 제시한다. 팀이 하나, 또는 개인 계정뿐, 이면 이 단계를 건너뛴다.

2. **팀이 선택되면 즉시 link로 진행한다.** 무엇이 일어날지 사용자에게 알리지만 별도의 확인은 묻지 않는다:
   ```
   Linking this project to <team name> on Vercel. This will create a Vercel
   project or associate this directory with the chosen existing project.
   ```

3. **git remote가 있으면**, 선택한 팀 스코프로 repo 기반 링크를 사용한다:
   ```bash
   vercel link --repo --scope <team-slug>
   ```
   git remote URL을 읽어 그 repo에서 배포하는 기존 Vercel 프로젝트와 매칭한다. `.vercel/repo.json`을 만든다. 이는 디렉토리명으로 매칭을 시도해 로컬 폴더와 Vercel 프로젝트명이 다를 때 자주 실패하는 `vercel link`, without `--repo`, 보다 훨씬 신뢰할 수 있다.

   **git remote가 없으면**, 표준 link로 폴백한다:
   ```bash
   vercel link --scope <team-slug>
   ```
   사용자에게 프로젝트 선택 또는 생성을 프롬프트한다. `.vercel/project.json`을 만든다.

4. CLI로 프리뷰를 배포하고 반환된 URL의 배포 상태를 확인한다.

---

### 연결 안 됨 + CLI 인증 안 됨 → 설치, 인증, link, 배포

Vercel CLI가 전혀 설정되지 않은 상태.

1. **CLI 설치, 아직 설치 안 된 경우:**
   ```bash
   npm install -g vercel
   ```

2. **인증:**
   ```bash
   vercel login
   ```
   사용자가 브라우저에서 인증을 완료한다. 로그인이 불가능한 비인터랙티브 환경이라면, 아래 **no-auth fallback**으로 건너뛴다.

3. **어느 팀에 배포할지 묻는다**: `vercel teams list --format json`의 팀 슬러그를 글머리 기호 목록으로 제시. 팀이 하나/개인 계정이면 건너뛴다. 선택되면 즉시 진행.

4. **선택한 팀 스코프로 프로젝트를 link한다**, git remote가 있으면 `--repo` 사용, 없으면 plain `vercel link`:
   ```bash
   vercel link --repo --scope <team-slug>   # if git remote exists
   vercel link --scope <team-slug>          # if no git remote
   ```

5. CLI로 배포하고 반환된 URL의 배포 상태를 확인한다.

---

### No-Auth Fallback: claude.ai sandbox

**사용 시점:** claude.ai sandbox에서 CLI를 설치하거나 인증할 수 없을 때의 최후의 수단. 인증이 필요하지 않다, **Preview URL**, 라이브 사이트, 과 **Claim URL**, Vercel 계정으로 이전, 을 반환한다.

```bash
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh [path]
```

**Arguments:**
- `path` - 배포할 디렉토리 또는 `.tgz` 파일, 기본값: 현재 디렉토리

아카이브 최상위에 디렉터리가 하나이고 그 안에 `package.json`, `vercel.json`, `.vercelignore` 또는 HTML 페이지가 있으면 해당 디렉터리가 배포 루트다. 그 밖의 아카이브는 최상위 구조를 유지한다.

**Examples:**
```bash
# Deploy current directory
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh

# Deploy specific project
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh /path/to/project

# Deploy existing tarball
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh /path/to/project.tgz
```

스크립트는 업로드 전에 프로젝트의 `.vercelignore`를 적용한다. 업로드가 접수되면 배포 응답을 출력한다. 아래 출력 지침에 따라 종료 코드와 확인 결과를 보고한다.

---

### No-Auth Fallback: Codex sandbox

**사용 시점:** CLI가 인증되지 않을 수 있는 Codex sandbox에서. Codex는 기본적으로 sandboxed 환경에서 실행된다, 먼저 CLI를 시도하고, 인증 실패 시 deploy 스크립트로 폴백한다.

1. **Vercel CLI가 설치되어 있는지 확인**, 이 체크는 권한 상승 불필요:
   ```bash
   command -v vercel
   ```

2. **`vercel`이 설치되어 있으면**, CLI로 배포 시도:
   ```bash
   vercel deploy [path] --environment=preview -y --no-wait
   ```

3. **`vercel`이 설치되지 않았거나 CLI가 "No existing credentials found"로 실패하면**, fallback 스크립트 사용:
   ```bash
   skill_dir="<path-to-skill>"

   # Deploy current directory
   bash "$skill_dir/scripts/deploy-codex.sh"

   # Deploy specific project
   bash "$skill_dir/scripts/deploy-codex.sh" /path/to/project

   # Deploy existing tarball
   bash "$skill_dir/scripts/deploy-codex.sh" /path/to/project.tgz
   ```

스크립트는 업로드 전에 프로젝트의 `.vercelignore`를 적용한다. 업로드가 접수되면 배포 응답을 출력한다. 아래 출력 지침에 따라 종료 코드와 확인 결과를 보고한다.


---

## 에이전트별 노트

### Claude Code / 터미널 기반 에이전트

전체 셸 접근 권한이 있다. `/mnt/skills/` 경로를 사용하지 말 것. CLI를 직접 사용해 위 결정 흐름을 따른다.

no-auth fallback은 skill의 설치 위치에서 deploy 스크립트를 실행한다:
```bash
bash ~/.claude/skills/deploy-to-vercel/scripts/deploy.sh [path]
```
경로는 사용자가 skill을 어디에 설치했는지에 따라 달라질 수 있다.

### Sandboxed 환경, claude.ai

`vercel login`이나 `git push`를 실행할 수 없을 가능성이 높다. **no-auth fallback, claude.ai sandbox**로 직접 간다.

### Codex

Codex는 sandbox에서 실행된다. CLI가 사용 가능한지 먼저 확인하고, 그 후 deploy 스크립트로 폴백한다. **no-auth fallback, Codex sandbox**로 간다.

---

## 출력

CLI 배포는 반환된 URL이나 ID로 `vercel inspect <url> --wait --timeout=5m`을 실행해 확인한 빌드 상태를 알린다. Git 배포는 대상 환경과 커밋의 상태 검사를 확인한다.

폴백 스크립트는 업로드가 접수되면 프리뷰 URL과 claim URL을 JSON으로 출력한다. 종료 코드 0은 프리뷰 URL이 HTTP 2xx를 반환했음을 뜻한다. 종료 코드 2는 확인 미완료를 뜻한다. 두 경우 모두 확보한 URL과 관찰한 상태를 전달한다. Vercel에서 빌드 완료를 확인한 경우에만 완료로 보고한다. 스크립트는 Bash, Git, tar, curl, Python 3을 사용한다. 압축 파일 입력에는 tar 추출 필터를 지원하는 Python이 필요하다.

---

## 트러블슈팅

### Network Egress Error, claude.ai

claude.ai의 네트워크 제한으로 배포가 실패하면, 사용자에게 알린다:

```
Deployment failed due to network restrictions. To fix this:

1. Go to https://claude.ai/settings/capabilities
2. Add *.vercel.com to the allowed domains
3. Try deploying again
```

### 업로드 확인이 끊겼을 때

응답을 기다리다 연결이 끊겼으면 기존 배포의 URL, ID 또는 커밋 상태로 접수 여부를 확인한다. 접수된 배포는 같은 대상을 조회해 상태를 확인한다. 네트워크 접근은 현재 세션의 권한을 따른다.

### CLI Auth Failure

`vercel login` 또는 `vercel deploy`가 인증 에러로 실패하면, no-auth deploy 스크립트로 폴백한다, 환경에 따라 claude.ai 또는 Codex 변형.

---

## Token 기반 인증, 비인터랙티브

`vercel login`을 사용할 수 없는 CI/CD 또는 비인터랙티브 환경용. 브라우저 인증 대신 `VERCEL_TOKEN` 환경 변수를 사용한다.

### 자격 증명 확인

CI 비밀 저장소나 프로젝트에서 사용하는 환경 로더로 토큰을 주입한다. 셸 추적을 끄고 설정 여부만 확인한다. `vercel whoami`로 인증된 계정을 확인한다.

```bash
set +x
test -n "${VERCEL_TOKEN:-}" && echo 'VERCEL_TOKEN is set'
```

### 프로젝트 및 팀 위치, `.vercel/` 디렉토리 건너뛰기

```bash
printenv VERCEL_PROJECT_ID
printenv VERCEL_ORG_ID
```

둘 다 설정되어 있으면 CLI가 자동으로 사용하고 모든 `.vercel/` 디렉토리를 건너뛴다:
```bash
export VERCEL_ORG_ID="<org-id>"
export VERCEL_PROJECT_ID="<project-id>"
```

**반드시 함께 설정**: 하나만 설정하면 에러가 발생한다.

프로젝트 URL에서 팀 슬러그 추출:
```bash
# e.g. "my-team" from "https://vercel.com/my-team/my-project"
echo "$PROJECT_URL" | sed 's|https://vercel.com/||' | cut -d/ -f1
```

### 빠른 토큰 배포, link 불필요

```bash
vercel deploy --environment=preview -y --no-wait --scope <team-slug>
```

### 환경 변수 관리, Token 사용

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
set +x
test -n "${VERCEL_TOKEN:-}" || echo 'VERCEL_TOKEN is missing'
```

**인증 에러**, `Authentication required`:
- 토큰이 만료되었거나 유효하지 않을 수 있음
- 검증: `vercel whoami`, env에서 `VERCEL_TOKEN` 사용
- 사용자에게 기존 비밀 저장소에서 토큰을 갱신하도록 안내한다

**잘못된 팀:**
```bash
vercel whoami --scope <team-slug>
```
