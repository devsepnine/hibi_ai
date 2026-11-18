# Detect framework from package.json
detect_framework() {
    local pkg_json="$1"

    if [ ! -f "$pkg_json" ]; then
        echo "null"
        return
    fi

    local dependencies
    dependencies=$(python3 - "$pkg_json" <<'PYTHON'
import json
import sys

with open(sys.argv[1]) as source:
    manifest = json.load(source)
for field in ["dependencies", "devDependencies"]:
    for name in manifest.get(field, {}):
        print(name)
PYTHON
    ) || return 1

    has_dep_exact() {
        grep -Fxq -- "$1" <<< "$dependencies"
    }

    has_dep_prefix() {
        grep -q "^$1" <<< "$dependencies"
    }

    # Order matters - check more specific frameworks first

    # Blitz
    if has_dep_exact "blitz"; then echo "blitzjs"; return; fi

    # Next.js
    if has_dep_exact "next"; then echo "nextjs"; return; fi

    # Gatsby
    if has_dep_exact "gatsby"; then echo "gatsby"; return; fi

    # Remix
    if has_dep_prefix "@remix-run/"; then echo "remix"; return; fi

    # React Router (v7 framework mode)
    if has_dep_prefix "@react-router/"; then echo "react-router"; return; fi

    # TanStack Start
    if has_dep_exact "@tanstack/start"; then echo "tanstack-start"; return; fi

    # Astro
    if has_dep_exact "astro"; then echo "astro"; return; fi

    # Hydrogen (Shopify)
    if has_dep_exact "@shopify/hydrogen"; then echo "hydrogen"; return; fi

    # SvelteKit
    if has_dep_exact "@sveltejs/kit"; then echo "sveltekit-1"; return; fi

    # Svelte (standalone)
    if has_dep_exact "svelte"; then echo "svelte"; return; fi

    # Nuxt
    if has_dep_exact "nuxt"; then echo "nuxtjs"; return; fi

    # Vue with Vitepress
    if has_dep_exact "vitepress"; then echo "vitepress"; return; fi

    # Vue with Vuepress
    if has_dep_exact "vuepress"; then echo "vuepress"; return; fi

    # Gridsome
    if has_dep_exact "gridsome"; then echo "gridsome"; return; fi

    # SolidStart
    if has_dep_exact "@solidjs/start"; then echo "solidstart-1"; return; fi

    # Docusaurus
    if has_dep_exact "@docusaurus/core"; then echo "docusaurus-2"; return; fi

    # RedwoodJS
    if has_dep_prefix "@redwoodjs/"; then echo "redwoodjs"; return; fi

    # Hexo
    if has_dep_exact "hexo"; then echo "hexo"; return; fi

    # Eleventy
    if has_dep_exact "@11ty/eleventy"; then echo "eleventy"; return; fi

    # Angular / Ionic Angular
    if has_dep_exact "@ionic/angular"; then echo "ionic-angular"; return; fi
    if has_dep_exact "@angular/core"; then echo "angular"; return; fi

    # Ionic React
    if has_dep_exact "@ionic/react"; then echo "ionic-react"; return; fi

    # Create React App
    if has_dep_exact "react-scripts"; then echo "create-react-app"; return; fi

    # Ember
    if has_dep_exact "ember-cli" || has_dep_exact "ember-source"; then echo "ember"; return; fi

    # Dojo
    if has_dep_exact "@dojo/framework"; then echo "dojo"; return; fi

    # Polymer
    if has_dep_prefix "@polymer/"; then echo "polymer"; return; fi

    # Preact
    if has_dep_exact "preact"; then echo "preact"; return; fi

    # Stencil
    if has_dep_exact "@stencil/core"; then echo "stencil"; return; fi

    # UmiJS
    if has_dep_exact "umi"; then echo "umijs"; return; fi

    # Sapper (legacy Svelte)
    if has_dep_exact "sapper"; then echo "sapper"; return; fi

    # Saber
    if has_dep_exact "saber"; then echo "saber"; return; fi

    # Sanity
    if has_dep_exact "sanity"; then echo "sanity-v3"; return; fi
    if has_dep_prefix "@sanity/"; then echo "sanity"; return; fi

    # Storybook
    if has_dep_prefix "@storybook/"; then echo "storybook"; return; fi

    # NestJS
    if has_dep_exact "@nestjs/core"; then echo "nestjs"; return; fi

    # Elysia
    if has_dep_exact "elysia"; then echo "elysia"; return; fi

    # Hono
    if has_dep_exact "hono"; then echo "hono"; return; fi

    # Fastify
    if has_dep_exact "fastify"; then echo "fastify"; return; fi

    # h3
    if has_dep_exact "h3"; then echo "h3"; return; fi

    # Nitro
    if has_dep_exact "nitropack"; then echo "nitro"; return; fi

    # Express
    if has_dep_exact "express"; then echo "express"; return; fi

    # Vite (generic - check last among JS frameworks)
    if has_dep_exact "vite"; then echo "vite"; return; fi

    # Parcel
    if has_dep_exact "parcel"; then echo "parcel"; return; fi

    # No framework detected
    echo "null"
}

INPUT_PATH="${1:-.}"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT
TARBALL="$TEMP_DIR/project.tgz"
STAGING_DIR="$TEMP_DIR/staging"
file_helper="$(dirname "${BASH_SOURCE[0]}")/deployment-files.py"
# Keep macOS tar from adding AppleDouble sidecars to the deployment.
export COPYFILE_DISABLE=1

if [[ -f "$INPUT_PATH" && "$INPUT_PATH" == *.tgz ]]; then
    mkdir "$TEMP_DIR/input"
    python3 "$file_helper" --extract "$INPUT_PATH" "$TEMP_DIR/input"
    INPUT_PATH="$TEMP_DIR/input"
    shopt -s nullglob dotglob
    roots=("$INPUT_PATH"/*)
    if [[ ${#roots[@]} -eq 1 && -d "${roots[0]}" ]]; then
        wrapper="${roots[0]}"
        pages=("$wrapper"/*.html)
        if [[ -f "$wrapper/package.json" || -f "$wrapper/vercel.json" || -f "$wrapper/.vercelignore" || ${#pages[@]} -gt 0 ]]; then
            INPUT_PATH="$wrapper"
        fi
    fi
    shopt -u nullglob dotglob
fi

if [[ ! -d "$INPUT_PATH" ]]; then
    echo "Expected a project directory or a .tgz archive." >&2
    exit 1
fi
PROJECT_PATH=$(cd "$INPUT_PATH" && pwd -P)
SCAN_ROOT="$PROJECT_PATH"
if [[ ! -e "$PROJECT_PATH/.vercelignore" ]]; then
    REPO_ROOT=$(git -C "$PROJECT_PATH" rev-parse --show-toplevel 2>/dev/null || true)
    if [[ -n "$REPO_ROOT" && -f "$REPO_ROOT/.vercelignore" ]]; then
        SCAN_ROOT="$REPO_ROOT"
    fi
fi

python3 "$file_helper" "$PROJECT_PATH" "$SCAN_ROOT" "$TEMP_DIR/filter" "$STAGING_DIR"

FRAMEWORK=$(detect_framework "$STAGING_DIR/package.json")
if [[ ! -f "$STAGING_DIR/package.json" ]]; then
    shopt -s nullglob
    html_files=("$STAGING_DIR"/*.html)
    if [[ ${#html_files[@]} -eq 1 && -f "${html_files[0]}" && "${html_files[0]}" != "$STAGING_DIR/index.html" ]]; then
        mv "${html_files[0]}" "$STAGING_DIR/index.html"
    fi
fi
tar -czf "$TARBALL" -C "$STAGING_DIR" .

echo "Submitting deployment..." >&2
response_file="$TEMP_DIR/response.json"
curl --fail --silent --show-error --connect-timeout 15 --max-time 300 \
    -o "$response_file" -F "file=@$TARBALL" -F "framework=$FRAMEWORK" "$DEPLOY_ENDPOINT"

preview_url=$(python3 - "$response_file" <<'PYTHON'
import json
import sys
from urllib.parse import urlsplit

with open(sys.argv[1]) as response:
    payload = json.load(response)
if not isinstance(payload, dict) or payload.get("error"):
    sys.exit("The deployment service rejected the upload.")
for field in ["previewUrl", "claimUrl"]:
    value = payload.get(field)
    if not isinstance(value, str) or any(char.isspace() for char in value):
        sys.exit(f"Invalid deployment response field: {field}")
    parts = urlsplit(value)
    if parts.scheme != "https" or not parts.hostname or parts.username or parts.password:
        sys.exit(f"Expected an HTTPS URL in {field}.")
print(payload["previewUrl"])
PYTHON
)

# Preserve the deployment identity even if verification times out.
cat "$response_file"
printf '\n'
echo "Upload accepted. Checking the preview URL." >&2
for ((attempt = 1; attempt <= 60; attempt++)); do
    status=$(curl --silent --output /dev/null --write-out '%{http_code}' \
        --connect-timeout 5 --max-time 5 "$preview_url") || status=000
    case "$status" in
        2??)
            echo "The preview URL returned HTTP $status. Build status requires Vercel Inspect." >&2
            exit 0 ;;
        000|404|429|5??) ;;
        *)
            echo "Preview verification incomplete: HTTP $status. Inspect this deployment in Vercel." >&2
            exit 2 ;;
    esac
    if (( attempt < 60 )); then sleep 5; fi
done
echo "Preview verification timed out. Inspect the existing deployment before retrying." >&2
exit 2
