---
name: deploy-to-vercel
description: Deploy to Vercel, interactive or VERCEL_TOKEN auth, preview by default, production only when explicit. Use when deploying a site. 버셀 배포, Vercel 배포, 프리뷰 배포, CI 배포.
metadata:
  author: vercel
  version: "3.1.0"
---

# Deploy to Vercel

Deploy any project to Vercel. **Always deploy as preview**, not production, unless the user explicitly asks for production.

Use the project and deployment environment the user requested.

## Step 1: Gather Project State

Run all four checks before deciding which method to use:

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

### Team selection

If the user belongs to multiple teams, present all available team slugs as a bulleted list and ask which one to deploy to. Once the user picks a team, proceed immediately to the next step, do not ask for additional confirmation.

Pass the team slug via `--scope` on all subsequent CLI commands, `vercel deploy`, `vercel link`, `vercel inspect`, etc.:

```bash
vercel deploy [path] --environment=preview -y --no-wait --scope <team-slug>
```

If the project is already linked, `.vercel/project.json` or `.vercel/repo.json` exists, the `orgId` in those files determines the team, no need to ask again. If there is only one team, or just a personal account, skip the prompt and use it directly.

**About the `.vercel/` directory:** A linked project has either:
- `.vercel/project.json`, created by `vercel link`, single project linking. Contains `projectId` and `orgId`.
- `.vercel/repo.json`, created by `vercel link --repo`, repo-based linking. Contains `orgId`, `remoteName`, and a `projects` array mapping directories to Vercel project IDs.

Either file means the project is linked. Check for both.

**Do NOT** use `vercel project inspect`, `vercel ls`, or `vercel link` to detect state in an unlinked directory, without a `.vercel/` config, they will interactively prompt, or with `--yes`, silently link as a side-effect. Only `vercel whoami` is safe to run anywhere.

## Step 2: Choose a Deploy Method

### Linked project

Use the CLI to deploy a preview from the working tree on any branch. If the user specifically requests a Git deployment, verify the configured production branch and Git integration in Vercel first. A push to the production branch requires an explicit production request and authorization for the push. When Git deployment is authorized, identify the result by project, environment, and commit SHA.

```bash
vercel deploy [path] --environment=preview -y --no-wait
```

Use `--no-wait` so the CLI returns immediately with the deployment URL instead of blocking until the build finishes, builds can take a while. Then check on the deployment status with:

```bash
vercel inspect <deployment-url> --wait --timeout=5m
```

For production deploys, only if user explicitly asks:
```bash
vercel deploy [path] --prod -y --no-wait
```

---

### Not linked + CLI is authenticated → Link first, then deploy

The CLI is working but the project isn't linked yet. This is the opportunity to get the user into the best state.

1. **Ask the user which team to deploy to.** Present the team slugs from Step 1 as a bulleted list. If there's only one team, or just a personal account, skip this step.

2. **Once a team is selected, proceed directly to linking.** Tell the user what will happen but do not ask for separate confirmation:
   ```
   Linking this project to <team name> on Vercel. This will create a Vercel
   project or associate this directory with the chosen existing project.
   ```

3. **If a git remote exists**, use repo-based linking with the selected team scope:
   ```bash
   vercel link --repo --scope <team-slug>
   ```
   This reads the git remote URL and matches it to existing Vercel projects that deploy from that repo. It creates `.vercel/repo.json`. This is much more reliable than `vercel link`, without `--repo`, which tries to match by directory name and often fails when the local folder and Vercel project are named differently.

   **If there is no git remote**, fall back to standard linking:
   ```bash
   vercel link --scope <team-slug>
   ```
   This prompts the user to select or create a project. It creates `.vercel/project.json`.

4. Deploy the preview with the CLI and inspect the URL it returns.

---

### Not linked + CLI not authenticated → Install, auth, link, deploy

The Vercel CLI isn't set up at all.

1. **Install the CLI, if not already installed:**
   ```bash
   npm install -g vercel
   ```

2. **Authenticate:**
   ```bash
   vercel login
   ```
   The user completes auth in their browser. If running in a non-interactive environment where login is not possible, skip to the **no-auth fallback** below.

3. **Ask which team to deploy to**: present team slugs from `vercel teams list --format json` as a bulleted list. If only one team / personal account, skip. Once selected, proceed immediately.

4. **Link the project** with the selected team scope, use `--repo` if a git remote exists, plain `vercel link` otherwise:
   ```bash
   vercel link --repo --scope <team-slug>   # if git remote exists
   vercel link --scope <team-slug>          # if no git remote
   ```

5. Deploy with the CLI and inspect the URL it returns.

---

### No-Auth Fallback: claude.ai sandbox

**When to use:** Last resort when the CLI can't be installed or authenticated in the claude.ai sandbox. This requires no authentication, it returns a **Preview URL**, live site, and a **Claim URL**, transfer to your Vercel account.

```bash
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh [path]
```

**Arguments:**
- `path` - Directory to deploy, or a `.tgz` file, defaults to current directory

For an archive containing one top level directory, use that directory as the deployment root when it contains `package.json`, `vercel.json`, `.vercelignore`, or an HTML page. Other archives retain their top level layout.

**Examples:**
```bash
# Deploy current directory
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh

# Deploy specific project
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh /path/to/project

# Deploy existing tarball
bash /mnt/skills/user/deploy-to-vercel/scripts/deploy.sh /path/to/project.tgz
```

The script applies the project’s `.vercelignore` before uploading. It prints the deployment response on acceptance. Use the exit status and output guidance below to report verification.

---

### No-Auth Fallback: Codex sandbox

**When to use:** In the Codex sandbox where the CLI may not be authenticated. Codex runs in a sandboxed environment by default, try the CLI first, and fall back to the deploy script if auth fails.

1. **Check whether the Vercel CLI is installed**, no escalation needed for this check:
   ```bash
   command -v vercel
   ```

2. **If `vercel` is installed**, try deploying with the CLI:
   ```bash
   vercel deploy [path] --environment=preview -y --no-wait
   ```

3. **If `vercel` is not installed, or the CLI fails with "No existing credentials found"**, use the fallback script:
   ```bash
   skill_dir="<path-to-skill>"

   # Deploy current directory
   bash "$skill_dir/scripts/deploy-codex.sh"

   # Deploy specific project
   bash "$skill_dir/scripts/deploy-codex.sh" /path/to/project

   # Deploy existing tarball
   bash "$skill_dir/scripts/deploy-codex.sh" /path/to/project.tgz
   ```

The script applies the project’s `.vercelignore` before uploading. It prints the deployment response on acceptance. Use the exit status and output guidance below to report verification.


---

## Agent-Specific Notes

### Claude Code / terminal-based agents

You have full shell access. Do NOT use the `/mnt/skills/` path. Follow the decision flow above using the CLI directly.

For the no-auth fallback, run the deploy script from the skill's installed location:
```bash
bash ~/.claude/skills/deploy-to-vercel/scripts/deploy.sh [path]
```
The path may vary depending on where the user installed the skill.

### Sandboxed environments, claude.ai

You likely cannot run `vercel login` or `git push`. Go directly to the **no-auth fallback, claude.ai sandbox**.

### Codex

Codex runs in a sandbox. Check if the CLI is available first, then fall back to the deploy script. Go to the **no-auth fallback, Codex sandbox**.

---

## Output

For a CLI deployment, inspect its exact URL or ID with `vercel inspect <url> --wait --timeout=5m`. Report the observed build state. For a Git deployment, inspect the commit check for the target environment.

The fallback prints JSON with the preview and claim URLs after the upload is accepted. Exit 0 indicates the preview endpoint returned HTTP 2xx. Exit 2 means verification is incomplete. Share the URLs and the observed status in either case. Report the build as ready only when Vercel confirms that state. The scripts use Bash, Git, tar, curl and Python 3. Archive inputs require Python’s tar extraction filters.

---

## Troubleshooting

### Network Egress Error, claude.ai

If deployment fails due to network restrictions on claude.ai, tell the user:

```
Deployment failed due to network restrictions. To fix this:

1. Go to https://claude.ai/settings/capabilities
2. Add *.vercel.com to the allowed domains
3. Try deploying again
```

### Interrupted uploads

If a connection drops while waiting for a response, use the deployment URL, ID or commit check to determine whether the upload was accepted. Resume verification on the existing deployment when available. Follow the network permissions of the current session.

### CLI Auth Failure

If `vercel login` or `vercel deploy` fails with authentication errors, fall back to the no-auth deploy script, claude.ai or Codex variant, depending on the environment.

---

## Token-Based Authentication, Non-Interactive

For CI/CD or non-interactive environments where `vercel login` cannot be used. Uses `VERCEL_TOKEN` env var instead of browser auth.

### Check credentials

Supply the credential through the CI secret store or the project’s existing trusted environment loader. Check its presence with shell tracing disabled. Use `vercel whoami` to verify the resulting identity.

```bash
set +x
test -n "${VERCEL_TOKEN:-}" && echo 'VERCEL_TOKEN is set'
```

### Locate Project and Team, Skip `.vercel/` Directory

```bash
printenv VERCEL_PROJECT_ID
printenv VERCEL_ORG_ID
```

If both are set, the CLI uses these automatically and skips any `.vercel/` directory:
```bash
export VERCEL_ORG_ID="<org-id>"
export VERCEL_PROJECT_ID="<project-id>"
```

**Must be set together**: setting only one causes an error.

Extract team slug from project URL:
```bash
# e.g. "my-team" from "https://vercel.com/my-team/my-project"
echo "$PROJECT_URL" | sed 's|https://vercel.com/||' | cut -d/ -f1
```

### Quick Token Deploy, no linking needed

```bash
vercel deploy --environment=preview -y --no-wait --scope <team-slug>
```

### Managing Environment Variables, with Token

```bash
echo "value" | vercel env add VAR_NAME --scope <team-slug>
echo "value" | vercel env add VAR_NAME production --scope <team-slug>
vercel env ls --scope <team-slug>
vercel env pull --scope <team-slug>
vercel env rm VAR_NAME --scope <team-slug> -y
```

### Token Authentication Troubleshooting

**Token not found:**
```bash
set +x
test -n "${VERCEL_TOKEN:-}" || echo 'VERCEL_TOKEN is missing'
```

**Authentication error**, `Authentication required`:
- Token may be expired or invalid
- Verify: `vercel whoami`, uses `VERCEL_TOKEN` from env
- Have the user refresh the secret in the configured credential store

**Wrong team:**
```bash
vercel whoami --scope <team-slug>
```
