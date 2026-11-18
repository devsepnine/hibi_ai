#!/bin/bash
set -euo pipefail

DEPLOY_ENDPOINT="https://codex-deploy-skills.vercel.sh/api/deploy"
source "$(dirname "${BASH_SOURCE[0]}")/deploy-common.sh"
