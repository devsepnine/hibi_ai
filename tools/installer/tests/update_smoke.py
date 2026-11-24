"""Exercise the signed updater with two native builds and a disposable signing key."""
import argparse
import functools
import hashlib
import http.server
import json
import os
import pathlib
import shutil
import subprocess
import tarfile
import tempfile
import threading
import time


def run(*args, **kwargs):
    subprocess.run(args, check=True, **kwargs)


def smoke(installer, work):
    manifest = installer / "Cargo.toml"
    target = installer / "target"
    env = dict(os.environ, CARGO_TARGET_DIR=str(target))
    current = subprocess.check_output([target / "debug/hibi", "--version"], text=True).split()[1].removeprefix("v")
    next_version = current + ".1"
    if "-" not in current:
        major, minor, patch = map(int, current.split("."))
        next_version = f"{major}.{minor}.{patch + 1}-smoke.1"
    platform = "macos" if os.uname().sysname == "Darwin" else "linux"
    prefix = f"hibi-ai-{next_version}-{platform}"
    old = work / f"hibi-ai-{current}-{platform}"
    old.mkdir()
    shutil.copy2(target / "debug/hibi", old / "hibi")
    (old / ".hibi-bundle").write_text(current)
    (old / "user-added-file").write_text("keep")
    build = work / "build"
    build.mkdir()
    for name in ["src", "assets", "build.rs"]:
        (build / name).symlink_to(installer / name)
    # Keep the package version replacement scoped to this disposable build.
    candidate = manifest.read_text().replace(f'version = "{current}"', f'version = "{next_version}"', 1)
    # A rehearsal shares the dependency cache while keeping the developer's hibi binary usable.
    (build / "Cargo.toml").write_text(candidate.replace('name = "hibi"', 'name = "hibi-update-candidate"', 1))
    shutil.copy2(installer / "Cargo.lock", build / "Cargo.lock")
    run("cargo", "build", "--manifest-path", build / "Cargo.toml", env=dict(env, CARGO_INCREMENTAL="0"))
    bundle = work / prefix
    bundle.mkdir()
    shutil.copy2(target / "debug/hibi-update-candidate", bundle / "hibi")
    if platform == "linux":
        run("strip", "--strip-debug", bundle / "hibi")
    (bundle / ".hibi-bundle").write_text(next_version)
    source = installer.parents[1] / "src"
    for name in ["agents", "commands", "skills", "hooks", "mcps", "plugins", "output-styles", "statusline"]:
        shutil.copytree(source / name, bundle / name, ignore=shutil.ignore_patterns("*-ko.md"))
    for name in ["settings.json", "CLAUDE.md", "AGENTS.md"]:
        shutil.copy2(source / name, bundle / name)
    (bundle / "skills/probe").write_text(next_version)
    archive = work / f"{prefix}.tar.gz"
    with tarfile.open(archive, "w:gz", compresslevel=1) as output:
        output.add(bundle, arcname=prefix)
    archives = [archive]
    if platform == "macos":
        items = [item.name for item in bundle.iterdir() if item.name not in {"hibi", ".hibi-bundle"}]
        run("bash", installer / "package-macos-app.sh", next_version, bundle, *items)
        app_zip = bundle / f"Hibi-{next_version}-macos.zip"
        archives.append(app_zip)
        old_app = work / "Renamed Hibi.app" / "Contents"
        (old_app / "MacOS").mkdir(parents=True)
        (old_app / "Resources").mkdir()
        (old_app / "Info.plist").write_text("<plist version=\"1.0\"><dict/></plist>")
        (old_app / "Resources/.hibi-bundle").write_text(current)
        shutil.copy2(target / "debug/hibi", old_app / "MacOS/hibi")
    (work / "checksums.txt").write_text("".join(
        f"{hashlib.sha256(item.read_bytes()).hexdigest()}  {item.name}\n" for item in archives))
    run("minisign", "-G", "-W", "-s", work / "fixture.key", "-p", work / "fixture.pub", stdout=subprocess.DEVNULL)
    run("minisign", "-S", "-s", work / "fixture.key", "-m", work / "checksums.txt")
    public = work / "release-assets"
    public.mkdir()
    for item in [*archives, work / "checksums.txt", work / "checksums.txt.minisig"]:
        os.link(item, public / item.name)
    home = work / "home"
    settings = {
        ".hibi/preferences.json": '{"language":"ko","appearance":"dark"}',
        ".hibi/install.json": json.dumps({
            "source": "https://github.com/devsepnine/hibi_ai", "version": "v1.25.0",
            "target": ".claude", "updated_at": "2026-11-01T12:00:00Z", "components": ["config/CLAUDE.md"],
        }),
        ".claude/CLAUDE.md": "Private workspace instructions",
        ".codex/AGENTS.md": "Private Codex instructions",
    }
    for name, value in settings.items():
        (home / name).parent.mkdir(parents=True, exist_ok=True)
        (home / name).write_text(value)
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=public)
    with http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
        thread = threading.Thread(target=server.serve_forever)
        thread.start()
        try:
            run("cargo", "test", "--locked", "--manifest-path", manifest, "signed_native_update", "--", "--ignored",
                env=dict(env, HOME=str(home),
                         CARGO_HOME=env.get("CARGO_HOME", str(pathlib.Path.home() / ".cargo")),
                         RUSTUP_HOME=env.get("RUSTUP_HOME", str(pathlib.Path.home() / ".rustup")),
                         HIBI_TEST_UPDATE_DIR=str(work), HIBI_TEST_NEXT_VERSION=next_version,
                         HIBI_TEST_UPDATE_URL=f"http://127.0.0.1:{server.server_port}"))
        finally:
            server.shutdown()
            thread.join()
    for name, value in settings.items():
        assert (home / name).read_text() == value, name


def launch_and_observe(exe):
    home = pathlib.Path(os.environ["HIBI_TEST_UPDATE_DIR"]) / "home"
    env = dict(os.environ, HOME=home, XDG_CONFIG_HOME=home)
    platform = os.uname().sysname
    if platform != "Darwin":
        # xwininfo observes DISPLAY, which can differ from the parent Wayland session.
        env.pop("WAYLAND_DISPLAY", None)
        env.pop("WAYLAND_SOCKET", None)
    app = subprocess.Popen([exe], cwd=home, env=env, start_new_session=True)
    try:
        deadline = time.monotonic() + 25
        while time.monotonic() < deadline:
            if app.poll() is not None:
                raise RuntimeError(f"Updated GUI exited with status {app.returncode}")
            if platform == "Darwin":
                # Query only the process we launched through NSWorkspace.
                script = ("ObjC.import('AppKit'); "
                          f"var app = $.NSRunningApplication.runningApplicationWithProcessIdentifier({app.pid}); "
                          "app && app.finishedLaunching ? 'ready' : 'waiting'")
                result = subprocess.run(["osascript", "-l", "JavaScript", "-e", script],
                                        text=True, capture_output=True, timeout=5)
                if result.stdout.strip() == "ready":
                    return
            else:
                windows = subprocess.check_output(["xwininfo", "-root", "-tree"], text=True)
                if '"HIBI AI"' in windows:
                    return
            time.sleep(0.2)
        raise RuntimeError("Updated GUI did not finish launching")
    finally:
        if app.poll() is None:
            app.terminate()
            try:
                app.wait(timeout=5)
            except subprocess.TimeoutExpired:
                app.kill()
                app.wait()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--launch", type=pathlib.Path, help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.launch:
        launch_and_observe(args.launch)
        raise SystemExit(0)
    installer = pathlib.Path(__file__).resolve().parents[1]
    run("cargo", "build", "--locked", "--manifest-path", installer / "Cargo.toml")
    with tempfile.TemporaryDirectory() as directory:
        smoke(installer, pathlib.Path(directory))
