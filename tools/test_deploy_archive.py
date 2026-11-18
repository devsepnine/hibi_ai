"""Inspect the archive passed to curl by both Vercel fallback scripts."""

import os
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1] / "src/skills/deploy-to-vercel/scripts"


def uploaded_project(base, entrypoint, project):
    curl = base / "curl"
    curl.write_text("""#!/bin/sh
set -eu
for arg do
    case "$arg" in
        framework=*) echo "${arg#framework=}" > "$RECORDED_FRAMEWORK" ;;
        file=@*) cp "${arg#file=@}" "$CAPTURE" ;;
        -o) output=$2 ;;
    esac
    shift
done
if [ -n "${output:-}" ]; then
    echo '{"previewUrl":"https://example.invalid","claimUrl":"https://example.invalid/claim"}' > "$output"
else
    echo 200
fi
""")
    curl.chmod(0o755)
    archive = base / "captured.tgz"
    env = {**os.environ, "PATH": str(base) + os.pathsep + os.environ["PATH"],
                   "CAPTURE": str(archive), "RECORDED_FRAMEWORK": str(base / "framework")}
    env.pop("COPYFILE_DISABLE", None)
    subprocess.run(["bash", SCRIPTS / entrypoint, project], env=env, check=True, capture_output=True)
    with tarfile.open(archive) as bundle:
        files = {member.name.removeprefix("./"): bundle.extractfile(member).read()
                 for member in bundle if member.isfile()}
        directories = {member.name.removeprefix("./") for member in bundle if member.isdir()}
    return files, directories, (base / "framework").read_text().strip()


class DeployArchiveTest(unittest.TestCase):
    def test_checked_out_submodule_and_parent_ignore_rules(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            repo = base / "repo"
            vendor = base / "vendor"
            for directory in (repo, vendor):
                subprocess.run(["git", "init", "-q", directory], check=True)

            app = repo / "apps/site"
            app.mkdir(parents=True)
            (app / ".well-known").write_text("public")
            (repo / "outside.txt").write_text("sibling")
            (repo / ".vercelignore").write_text(
                "/apps/site/library/private/\n*.tmp\n!/apps/site/library/keep.tmp\n"
                "/apps/site/library/logs/**\n!/apps/site/library/logs/public\n"
            )
            for name in ("source.txt", "keep.tmp", "drop.tmp", ".env.local", "private/hidden",
                         "logs/public", "logs/hidden"):
                path = vendor / name
                path.parent.mkdir(exist_ok=True)
                path.write_text(name)
            subprocess.run(["git", "add", "--force", "."], cwd=vendor, check=True)
            subprocess.run(
                ["git", "-c", "user.name=Fixture", "-c", "user.email=test@example.invalid",
                 "-c", "commit.gpgsign=false", "commit", "-qm", "fixture"],
                cwd=vendor, check=True,
            )
            subprocess.run(
                ["git", "-c", "protocol.file.allow=always", "submodule", "add", "-q",
                 vendor, "apps/site/library"],
                cwd=repo, check=True,
            )

            (app / "empty").mkdir()
            for name in ("deploy.sh", "deploy-codex.sh"):
                with self.subTest(script=name):
                    files, directories, _ = uploaded_project(base, name, app)
                    self.assertEqual(files, {
                        ".well-known": b"public",
                        "library/source.txt": b"source.txt",
                        "library/keep.tmp": b"keep.tmp",
                        "library/logs/public": b"logs/public",
                    })
                    self.assertIn("empty", directories)
                    self.assertNotIn("library/private", directories)

    def test_wrapped_archive_with_appledouble_sidecars(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            site = base / "site"
            site.mkdir()
            manifest = b'{"dependencies":{"next":"1"}}'
            (site / "package.json").write_bytes(manifest)
            (site / "empty").mkdir()
            sidecar = bytes.fromhex("0005160700020000") + bytes(16) + bytes.fromhex(
                "0001 00000009 00000026 00000001 00"
            )
            (base / "._site").write_bytes(sidecar)
            (site / "._package.json").write_bytes(sidecar)
            (site / "._ordinary").write_bytes(b"content")
            archive = base / "input.tgz"
            with tarfile.open(archive, "w:gz") as bundle:
                bundle.add(site, arcname="site")
                bundle.add(base / "._site", arcname="._site")
            files, directories, framework = uploaded_project(base, "deploy.sh", archive)
            self.assertEqual(framework, "nextjs")
            self.assertEqual(files, {"package.json": manifest, "._ordinary": b"content"})
            self.assertIn("empty", directories)


if __name__ == "__main__":
    unittest.main()
