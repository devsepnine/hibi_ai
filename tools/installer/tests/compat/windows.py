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
mode = sys.argv[2] if len(sys.argv) > 2 else "driver"

if mode == "desktop":
    assert k.GetConsoleCP() == 0
    info = ExecInfo()
    info.cbSize, info.mask, info.file, info.show = c.sizeof(info), 0x40, exe, 1
    if not shell.ShellExecuteExW(c.byref(info)):
        raise c.WinError(c.get_last_error())
    print(k.GetProcessId(info.process))
    k.CloseHandle(info.process)
elif mode == "terminal":
    assert k.GetConsoleCP() != 0
    proc = sp.Popen([exe])
    assert proc.wait(timeout=15) == 0
    print(proc.pid)
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
            print("PASS --tui stays in caller console")
        finally:
            proc.terminate()
            proc.wait(timeout=5)
else:
    build = sys.getwindowsversion().build
    print("Windows build", build, "CPU", os.environ.get("PROCESSOR_ARCHITECTURE"), flush=True)
    def helper(mode, flags, *args):
        return sp.check_output([sys.executable, __file__, exe, mode, *args],
                               creationflags=flags, text=True, timeout=45).strip()
    assert not window()
    for launch, flags in [("terminal", sp.CREATE_NEW_CONSOLE), ("desktop", sp.DETACHED_PROCESS)]:
        original = int(helper(launch, flags))
        pid = wait_window()
        try:
            time.sleep(.5)
            assert window() == pid
            helper("console-probe", sp.DETACHED_PROCESS, str(pid))
            print(f"PASS {launch}: launcher={original} window={pid} attached_console=false", flush=True)
            if launch == "terminal":
                assert original != pid
            else:
                print("Desktop needed console handoff:", original != pid, flush=True)
        finally:
            sp.run(["taskkill", "/pid", str(pid), "/f"], check=True)
    print(helper("tui", sp.CREATE_NEW_CONSOLE))
    with tempfile.TemporaryFile() as out:
        sp.run([exe, "--version"], stdout=out, check=True)
        out.seek(0)
        assert out.read().strip() == b"hibi v2.1.2 (Config Installer)"
    assert b"Usage: hibi" in sp.check_output([exe, "--help"])
    print("PASS --version file redirection and --help pipe")
