"""Exercise install.sh using isolated release assets and an injected rename failure."""

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile


def fixture(root, version):
    name = f"hibi-ai-{version}-linux"
    payload = root / name
    payload.mkdir()
    executable = payload / "hibi"
    executable.write_text(f"#!/bin/sh\nprintf '%s\\n' '{version}'\n")
    release = root / f"v{version}"
    release.mkdir()
    archive = release / f"{name}.tar.gz"
    with tarfile.open(archive, "w:gz") as output:
        output.add(payload, arcname=name)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (release / "checksums.txt").write_text(f"{digest}  {archive.name}\n")


def executable(path, body):
    path.write_text(f"#!/bin/sh\nset -eu\n{body}")
    path.chmod(0o755)


def smoke(root):
    for version in ("1.0.0", "2.0.0"):
        fixture(root, version)
    shims = root / "commands"
    shims.mkdir()
    executable(shims / "curl", 'url=$2\ncp "$FIXTURE_RELEASES/${url#*/download/}" "$4"\n')
    executable(shims / "mv", '''
case "$FIXTURE_FAILURE:$2" in
    prepare:*/hibi-ai-2.0.0-linux|activate:*/next|publish:*/bin/.hibi-install.*/hibi)
        echo "Injected $FIXTURE_FAILURE failure" >&2
        exit 1 ;;
    after_publish:*/bin/.hibi-install.*/hibi)
        "$REAL_MV" "$@"
        kill -TERM "$PPID"
        exit 0 ;;
esac
exec "$REAL_MV" "$@"
''')
    env = dict(os.environ, PATH=f"{shims}:{os.environ['PATH']}", CDPATH=".",
               FIXTURE_RELEASES=str(root), REAL_MV=shutil.which("mv"), HIBI_PREFIX="prefix")
    script = Path(__file__).resolve().parents[3] / "install.sh"
    binary = root / "prefix/bin/hibi"

    def install(version, failure=""):
        return subprocess.run(["sh", script], cwd=root, check=False, capture_output=True,
                              env=dict(env, HIBI_VERSION=version, FIXTURE_FAILURE=failure))

    result = install("1.0.0")
    assert result.returncode == 0, result.stderr
    assert binary.readlink().is_absolute()
    for failure in ("prepare", "activate", "publish"):
        result = install("2.0.0", failure)
        assert result.returncode != 0 and b"Injected" in result.stderr, result
        assert subprocess.check_output([binary]).strip() == b"1.0.0"
        assert not (root / "prefix/lib/.hibi-install-previous").exists()
    result = install("2.0.0")
    assert result.returncode == 0, result.stderr
    assert subprocess.check_output([binary]).strip() == b"2.0.0"

    standalone = root / "standalone/bin/hibi"
    standalone.parent.mkdir(parents=True)
    executable(standalone, "echo standalone\n")
    interrupted = subprocess.run(["sh", script], cwd=root, check=False, capture_output=True,
        env=dict(env, HIBI_PREFIX="standalone", HIBI_VERSION="2.0.0", FIXTURE_FAILURE="after_publish"))
    assert interrupted.returncode == 143, interrupted
    assert subprocess.check_output([standalone]).strip() == b"2.0.0"
    print("PASS relative prefix, staged upgrade failures, and successful retry")


if __name__ == "__main__":
    with tempfile.TemporaryDirectory(prefix="hibi-install-smoke-") as directory:
        smoke(Path(directory))
