import os
from pathlib import Path
import pty
import tempfile
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

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools/installer/tests"))
from cli_smoke import terminal

with tempfile.TemporaryDirectory() as home:
    terminal(exe, Path(home), dict(os.environ, HOME=home, TERM="xterm-256color"), quit_at_picker=True)
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
