import os
import pty
import signal
import subprocess
import sys
import time
from pathlib import Path

app = Path(sys.argv[1]).resolve()
binary = app / "Contents/MacOS/hibi"

def window():
    return subprocess.check_output([sys.argv[2], binary], text=True).strip()

def wait_window():
    for _ in range(100):
        if value := window():
            return int(value)
        time.sleep(.15)
    raise AssertionError("No visible Hibi window from the packaged executable")

def close(pid):
    os.kill(pid, signal.SIGTERM)
    for _ in range(50):
        if not window():
            return
        time.sleep(.1)
    raise AssertionError("Hibi window did not close")

assert not window()
subprocess.run(["open", "-n", app], check=True)
pid = wait_window()
print("PASS Launch Services opened packaged Hibi.app; window pid", pid, flush=True)
close(pid)
master_pid, master = pty.fork()
if not master_pid:
    os.execv(str(binary), [str(binary)])
pid = wait_window()
try:
    assert pid != master_pid
    assert os.waitpid(master_pid, os.WNOHANG) == (master_pid, 0)
    os.close(master)
    time.sleep(.5)
    assert window() == str(pid)
    print("PASS terminal returned and the same window survived PTY closure", pid)
finally:
    close(pid)
