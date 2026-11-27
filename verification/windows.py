import ctypes as c
from ctypes import wintypes as w
import os
from pathlib import Path
import subprocess as sp
import sys
import tempfile
import time

k, u, shell = (c.WinDLL(name, use_last_error=True) for name in ("kernel32", "user32", "shell32"))
k.GetProcessId.argtypes = [w.HANDLE]
k.CloseHandle.argtypes = [w.HANDLE]
u.FindWindowW.argtypes = [w.LPCWSTR, w.LPCWSTR]
u.FindWindowW.restype = w.HWND
u.GetWindowThreadProcessId.argtypes = [w.HWND, c.POINTER(w.DWORD)]
u.IsWindowVisible.argtypes = [w.HWND]

class ExecInfo(c.Structure):
    _fields_ = [("cbSize", w.DWORD), ("mask", w.ULONG), ("hwnd", w.HWND),
                ("verb", w.LPCWSTR), ("file", w.LPCWSTR), ("args", w.LPCWSTR),
                ("cwd", w.LPCWSTR), ("show", c.c_int), ("inst", w.HINSTANCE),
                ("list", c.c_void_p), ("cls", w.LPCWSTR), ("key", w.HKEY),
                ("hotkey", w.DWORD), ("icon", w.HANDLE), ("process", w.HANDLE)]

def window():
    hwnd = u.FindWindowW(None, "HIBI AI")
    pid = w.DWORD()
    if hwnd and u.IsWindowVisible(hwnd):
        u.GetWindowThreadProcessId(hwnd, c.byref(pid))
    return pid.value

def wait_window():
    for _ in range(250):
        if pid := window():
            return pid
        time.sleep(.1)
    raise AssertionError("No visible HIBI AI window")

exe = str(Path(sys.argv[1]).resolve())
mode = sys.argv[2]
def record(message):
    Path(os.environ["PROBE_RESULT"]).write_text(str(message), encoding="utf-8")


if mode == "desktop":
    assert k.GetConsoleCP() == 0
    info = ExecInfo()
    info.cbSize, info.mask, info.file, info.show = c.sizeof(info), 0x40, exe, 1
    if not shell.ShellExecuteExW(c.byref(info)):
        raise c.WinError(c.get_last_error())
    record(k.GetProcessId(info.process))
    k.CloseHandle(info.process)
elif mode == "terminal":
    assert k.GetConsoleCP() != 0
    with open('CONIN$', 'rb') as cin, open('CONOUT$', 'wb') as cout:
        proc = sp.Popen([exe], stdin=cin, stdout=cout, stderr=cout)
    record(f"spawned {proc.pid}, waiting for launcher exit")
    assert proc.wait(timeout=15) == 0
    record(proc.pid)
elif mode == "console-probe":
    assert k.GetConsoleCP() == 0
    if k.AttachConsole(int(sys.argv[3])):
        k.FreeConsole()
        raise AssertionError("GUI was attached to a console")
    assert c.get_last_error() == 6, c.get_last_error()
elif mode == "tui":
    assert k.GetConsoleCP() != 0
    # Use console handles for the app even when this helper's output is captured.
    with open('CONIN$', 'rb') as cin, open('CONOUT$', 'wb') as cout:
        proc = sp.Popen([exe, "--tui"], stdin=cin, stdout=cout, stderr=cout)
        try:
            time.sleep(2)
            ids = (w.DWORD * 32)()
            count = k.GetConsoleProcessList(ids, len(ids))
            assert proc.poll() is None and proc.pid in ids[:count] and not window()
            record("PASS --tui stays in caller console")
        finally:
            proc.terminate()
            proc.wait(timeout=5)
else:
    build = sys.getwindowsversion().build
    print("Windows build", build, "CPU", os.environ.get("PROCESSOR_ARCHITECTURE"), flush=True)
    def helper(name, flags, *args):
        with tempfile.NamedTemporaryFile(dir=os.environ["RUNNER_TEMP"], prefix="hibi-probe-", delete=False) as marker:
            result = Path(marker.name)
        print("Starting", name, flush=True)
        with result.with_suffix(".log").open("w") as log:
            process = sp.Popen([sys.executable, "-u", __file__, exe, name, *args],
                               creationflags=flags, stdin=sp.DEVNULL, stdout=log, stderr=log,
                               env=dict(os.environ, PROBE_RESULT=str(result)))
            try:
                status = process.wait(timeout=35)
            except sp.TimeoutExpired:
                process.kill()
                raise
            finally:
                print(name, "result:", result.read_text(encoding="utf-8"),
                      result.with_suffix(".log").read_text(errors="replace"), flush=True)
        assert status == 0, (name, status)
        return result.read_text(encoding="utf-8").strip()

    scenario = sys.argv[3]
    if scenario in ("terminal", "desktop"):
        assert not window()
        flags = sp.CREATE_NEW_CONSOLE if scenario == "terminal" else sp.DETACHED_PROCESS
        original = int(helper(scenario, flags))
        pid = wait_window()
        try:
            time.sleep(.5)
            assert window() == pid
            helper("console-probe", sp.DETACHED_PROCESS, str(pid))
            print(f"PASS {scenario}: launcher={original} window={pid} attached_console=false", flush=True)
            if scenario == "terminal":
                assert original != pid
            else:
                print("Desktop needed console handoff:", original != pid, flush=True)
                if build >= 26100:
                    assert original == pid, "Desktop launch allocated a console before opening the GUI"
        finally:
            sp.run(["taskkill", "/pid", str(pid), "/f"], check=True, timeout=10)
    elif scenario == "tui":
        print(helper("tui", sp.CREATE_NEW_CONSOLE))
    else:
        with tempfile.TemporaryFile() as out:
            sp.run([exe, "--version"], stdout=out, check=True, timeout=10)
            out.seek(0)
            assert out.read().strip() == b"hibi v2.1.2 (Config Installer)"
        assert b"Usage: hibi" in sp.check_output([exe, "--help"], timeout=10)
        print("PASS --version file redirection and --help pipe")
