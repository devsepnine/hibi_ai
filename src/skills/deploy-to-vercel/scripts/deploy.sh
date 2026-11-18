#!/bin/bash
set -euo pipefail

DEPLOY_ENDPOINT="https://claude-skills-deploy.vercel.com/api/deploy"
source "$(dirname "${BASH_SOURCE[0]}")/deploy-common.sh"
