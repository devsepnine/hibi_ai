---
name: deps-updater
description: "Check or upgrade npm dependencies and lockfiles. npm 의존성, 패키지 버전 업데이트."
---

# Update npm dependencies

Use this skill to check npm dependency versions or apply a package upgrade. For a version check, report the findings. An update request goes through the preparation and verification steps below.

## Scope the update

1. Identify the npm project root and affected manifests. Use the project's declared Node and npm versions, registry configuration, and lockfile policy. For an npm workspace, identify the selected workspace packages and the shared lockfile. If the project declares a different package manager, use its workflow.
2. Read the affected manifests and active lockfile. Record each selected direct dependency's declared range and installed or locked version. Distinguish an installed version from a locked one. If neither is available, mark it unknown.
3. Check available registry versions for the selected packages. Use `npm outdated --json` as a starting point. Read its JSON results even when npm exits with status 1 after finding outdated packages. For a highest stable version check, read the published version list with `npm view "$package" versions --json`. Compare versions using SemVer and exclude prereleases unless the user requests them. If a lookup fails, report the package and the lookup error.
4. Show the proposed version changes, which ones fit the declared range, and known compatibility concerns. Check release notes and affected usages when a version change could break the project. Keep exact pins and nonregistry sources unless the user asks to change them. Confirm scope only where the request leaves it unclear, such as whether a bulk upgrade includes major versions.

## Prepare and verify in an isolated copy

1. Start from the current project files so existing edits are part of the baseline. Use a private temporary copy with its own `node_modules`. Preserve the workspace layout and make sure local dependencies resolve inside the copy. If the layout or an input cannot be reproduced, report the blocker before applying changes. Save a separate baseline of the affected dependency files and record which files are absent.
2. Resolve the selected updates and review the manifest and lockfile diff before running dependency code. Use the applicable npm command with `--ignore-scripts` for this step. For lockfile-only preparation, also use `--package-lock-only` when the command supports it. Check resolved versions, dependency sections, ranges and pins. Keep only lockfile changes needed by the selected updates and resolve any out-of-scope direct dependency changes.
3. Verify the project's usual install and relevant tests or builds. Use `npm ci` when the project expects a clean install from a compatible lockfile. A temporary copy shares the process's filesystem and environment permissions. When the verification would execute untrusted dependency code, use a disposable sandbox that cannot reach the original checkout, private files, or credentials. For private registries, retrieve the packages first with scripts disabled and give the sandbox only the required package data, such as a prepared cache. Leave registry tokens and credential-bearing `.npmrc` files out of the sandbox. If the necessary isolation is unavailable, report the checks you could complete and keep the candidate in the copy. If install or verification fails, report the command and affected packages and preserve the original project.

## Apply the verified changes

1. Build a patch from the baseline to the verified copy. Include only the selected manifests and the active lockfile the project intends to keep. Compare the original files with the baseline just before applying. If they changed meanwhile, preserve the new edits and refresh the candidate before applying it.
2. Use a checked patch to transfer the changes as one operation. Make the patch paths relative to the checkout root, or to the project root outside Git. Check the file list before applying. Keep the patch and baseline until the result is verified.

   ```bash
   git -C "$patch_root" apply --check "$dependency_patch" &&
     git -C "$patch_root" apply "$dependency_patch"
   ```

3. Read back the original manifests and lockfile and compare them with the verified copy. If applying fails, inspect the actual files and report which changes, if any, landed. Preserve user edits when resolving the failure.
4. Report the versions applied, the verification results, and any skipped packages. Say where dependencies were installed and which install procedure to use if the original checkout needs its installed packages refreshed.
