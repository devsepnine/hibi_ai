import os
from pathlib import Path
import pty
import select
import signal
import subprocess
import sys
import time

exe = Path(sys.argv[1]).resolve()
probe = sys.argv[2]
def start(*args):
    pid, master = pty.fork()
    if not pid:
        os.execv(str(exe), [str(exe), *args])
    return pid, master

pid, master = start("--tui")
try:
    data = b""
    until = time.monotonic() + 25
    while time.monotonic() < until and b"Claude" not in data:
        if select.select([master], [], [], 1)[0]:
            data += os.read(master, 65536)
    assert b"Claude" in data, data[-500:]
finally:
    os.kill(pid, signal.SIGTERM)
    os.close(master)
    os.waitpid(pid, 0)
pid, master = start()
gui = None
try:
    for _ in range(100):
        result = subprocess.check_output([probe, str(exe)], text=True).strip()
        if result:
            gui = int(result)
            break
        time.sleep(.15)
    assert gui and gui != pid
    os.close(master)
    time.sleep(.5)
    assert subprocess.check_output([probe, str(exe)], text=True).strip() == str(gui)
finally:
    if gui:
        os.kill(gui, signal.SIGTERM)
print("PASS Homebrew TUI, GUI window and survival after terminal closure")
