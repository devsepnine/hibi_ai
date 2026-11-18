#!/usr/bin/env python3
"""Check headless CLI flags and terminal cleanup using isolated Unix PTYs.

Uses Python's standard library only. Every invocation gets a temporary HOME.
Example: python tools/installer/tests/cli_smoke.py --binary tools/installer/target/release/hibi
"""

import argparse
import errno
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import shutil
import signal
import struct
import sys
import subprocess
import tempfile
import termios
import time


def isolated_environment(home):
    env = os.environ.copy()
    for name in ("DISPLAY", "WAYLAND_DISPLAY", "WAYLAND_SOCKET", "XAUTHORITY", "XDG_RUNTIME_DIR"):
        env.pop(name, None)
    env.update(
        HOME=str(home),
        USERPROFILE=str(home),
        XDG_CONFIG_HOME=str(home / ".config"),
        XDG_CACHE_HOME=str(home / ".cache"),
        XDG_DATA_HOME=str(home / ".local/share"),
        TERM="xterm-256color",
        NO_COLOR="1",
    )
    return env


def cli(binary, arguments, cwd, env):
    result = subprocess.run(
        [str(binary), *arguments], cwd=cwd, env=env,
        stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        timeout=15, check=False,
    )
    assert result.returncode == 0, result.stdout.decode(errors="replace")
    assert b"\x1b[?1049h" not in result.stdout, "Headless command entered alternate screen"
    assert b"GUI startup failed" not in result.stdout, result.stdout.decode(errors="replace")
    return result.stdout


def terminal(binary, cwd, env, quit_at_picker=False):
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))
    process = None
    output = bytearray()
    sent_quit = False

    def establish_terminal():
        os.setsid()
        fcntl.ioctl(0, termios.TIOCSCTTY, 0)

    try:
        command = [str(binary), "--tui"]
        if os.uname().sysname == "Darwin":
            # Darwin revokes the slave PTY when its session leader exits.
            # Keep the session alive until the child's terminal flags are checked.
            command = [sys.executable, "-c", """
import subprocess, sys, termios
before = termios.tcgetattr(0)
result = subprocess.run(sys.argv[1:]).returncode
after = termios.tcgetattr(0)
# PENDIN is the Darwin kernel's pending input state after cooked mode resumes.
before[3] &= ~termios.PENDIN
after[3] &= ~termios.PENDIN
sys.exit(result if before == after else 90)
""", *command]
        process = subprocess.Popen(
            command, cwd=cwd, env=env,
            stdin=slave, stdout=slave, stderr=slave,
            close_fds=True, preexec_fn=establish_terminal,
        )
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            readable, _, _ = select.select([master], [], [], .1)
            if readable:
                try:
                    chunk = os.read(master, 65536)
                except OSError as error:
                    if error.errno != errno.EIO:
                        raise
                    chunk = b""
                output.extend(chunk)
                if b"\x1b]11;?" in chunk:
                    os.write(master, b"\x1b]11;rgb:1e1e/1e1e/2e2e\x1b\\")
                if quit_at_picker and not sent_quit and b"Select target" in output:
                    os.write(master, b"q")
                    sent_quit = True
            if process.poll() is not None:
                # Read all final escape sequences written immediately before exit.
                while select.select([master], [], [], 0)[0]:
                    try:
                        chunk = os.read(master, 65536)
                    except OSError as error:
                        if error.errno != errno.EIO:
                            raise
                        break
                    if not chunk:
                        break
                    output.extend(chunk)
                break
        else:
            raise AssertionError("Terminal command did not exit within 15 seconds: "
                                 + output.decode(errors="replace")[-1200:])

        result = bytes(output)
        if os.uname().sysname == "Darwin":
            assert process.returncode != 90, result.decode(errors="replace")[-1800:]
        else:
            assert termios.tcgetattr(slave) == before, "Terminal input flags were not restored"
        assert b"\x1b[?1049h" in result, "Terminal never entered alternate screen"
        assert b"\x1b[?1049l" in result, "Terminal never left alternate screen"
        assert b"\x1b[?25h" in result, "Cursor visibility was not restored"
        if quit_at_picker:
            assert sent_quit, "CLI picker was never rendered"
            assert process.returncode == 0, result.decode(errors="replace")[-1200:]
        else:
            assert process.returncode != 0, "Initialization failure returned exit status zero"
            assert b"Cannot find source directory" in result, result.decode(errors="replace")[-1200:]
        return result
    finally:
        if process is not None and process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
        if os.uname().sysname != "Darwin":
            termios.tcsetattr(slave, termios.TCSANOW, before)
        os.close(master)
        os.close(slave)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--artifacts", type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    repository = Path(__file__).resolve().parents[3]
    results = []
    with tempfile.TemporaryDirectory(prefix="hibi-cli-smoke-") as directory:
        root = Path(directory)
        home = root / "home"
        home.mkdir()
        isolated = root / "isolated"
        isolated.mkdir()
        copied = isolated / "hibi"
        shutil.copy2(binary, copied)
        env = isolated_environment(home)
        gui_enabled = False
        for name, arguments, expected in (
            ("help_without_display", ["--help"], b"Usage: hibi"),
            ("version_without_display", ["--version"], b"hibi "),
            ("help_precedes_sync_and_terminal", ["--sync", "--tui", "--help"], b"Usage: hibi"),
            ("version_precedes_sync", ["--sync", "--version"], b"hibi "),
        ):
            output = cli(copied, arguments, isolated, env)
            assert expected in output, output.decode(errors="replace")
            if arguments == ["--help"]:
                gui_enabled = b"launch the desktop installer" in output
            if "precedes" in name:
                assert b"Syncing sources" not in output
            results.append({"test": name, "status": "passed"})
        if gui_enabled and os.uname().sysname == "Linux":
            result = subprocess.run(
                [str(copied)], cwd=isolated, env=env, stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=15, check=False,
            )
            assert result.returncode != 0, "Headless GUI startup unexpectedly succeeded"
            assert b"--tui" in result.stdout, result.stdout.decode(errors="replace")
            assert b"panicked" not in result.stdout.lower(), result.stdout.decode(errors="replace")
            results.append({"test": "headless_gui_reports_terminal_guidance_without_panic", "status": "passed"})
        else:
            results.append({"test": "headless_gui_reports_terminal_guidance_without_panic", "status": "skipped"})
        output = cli(copied, ["--sync"], repository, env)
        assert b"bundled: local" in output, output.decode(errors="replace")
        results.append({"test": "sync_with_isolated_home", "status": "passed"})
        failure = terminal(copied, isolated, env)
        results.append({"test": "tui_initialization_failure_restores_terminal", "status": "passed"})
        success = terminal(copied, repository, env, quit_at_picker=True)
        results.append({"test": "tui_quit_restores_terminal", "status": "passed"})
        if args.artifacts:
            args.artifacts.mkdir(parents=True, exist_ok=True)
            (args.artifacts / "tui-failure.log").write_bytes(failure)
            (args.artifacts / "tui-quit.log").write_bytes(success)
            (args.artifacts / "results.json").write_text(json.dumps(results, indent=2) + "\n")
    for result in results:
        label = "PASS" if result["status"] == "passed" else "SKIP"
        print(f"{label} {result['test']}")


if __name__ == "__main__":
    main()
