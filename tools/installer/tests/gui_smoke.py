#!/usr/bin/env python3
"""Exercise the desktop installer through X11 with an isolated home.

Requires python-xlib, Pillow, Tesseract with English data, and an X11 display.
Use Xvfb rather than a user's desktop to make synthetic input reliable.
Example: python tools/installer/tests/gui_smoke.py --binary tools/installer/target/debug/hibi --display :99 --artifacts /tmp/hibi-gui-smoke
"""

import argparse
import csv
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time

try:
    from PIL import Image, ImageChops, ImageOps
    from Xlib import X, XK, display, protocol
    from Xlib.ext import xtest
except ImportError as error:
    sys.exit(f"GUI smoke dependencies missing: {error}. Install python-xlib and Pillow.")


def normalized(value):
    return re.sub(r"[^a-z0-9]", "", value.lower())


class Desktop:
    def __init__(self, args):
        self.args = args
        self.artifacts = args.artifacts.resolve()
        self.artifacts.mkdir(parents=True, exist_ok=True)
        self.home = Path(tempfile.mkdtemp(prefix="home-", dir=self.artifacts))
        self.process = None
        self.log = None
        self.window = None
        self.results = []
        self.d = display.Display(args.display)
        self.root = self.d.screen().root
        self.ocr_cache = None

    def launch(self):
        stub = self.home / "bin"
        stub.mkdir(exist_ok=True)
        for name in ("claude", "codex"):
            path = stub / name
            path.write_text("""#!/bin/sh
case "$*" in
    --version) echo gui-smoke-fixture ;;
    *"list --json") echo '[]' ;;
    *list*) echo '' ;;
    *) echo 'External CLI operations are disabled in this smoke test' >&2; exit 1 ;;
esac
""")
            path.chmod(0o755)
        env = os.environ.copy()
        env.update(HOME=str(self.home), DISPLAY=self.args.display, WAYLAND_DISPLAY="",
                   XDG_CONFIG_HOME=str(self.home / ".config"), XDG_CACHE_HOME=str(self.home / ".cache"),
                   XDG_DATA_HOME=str(self.home / ".local/share"), PATH=f"{stub}:/usr/bin:/bin")
        self.log = (self.artifacts / "application.log").open("a")
        existing = {w.id for w in self.root.query_tree().children}
        self.process = subprocess.Popen([str(self.args.binary.resolve())], env=env,
                                        cwd=Path(__file__).resolve().parents[3],
                                        stdin=subprocess.DEVNULL, stdout=self.log, stderr=self.log)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RuntimeError("Installer exited before opening a window; inspect application.log")
            for window in self.root.query_tree().children:
                if window.id not in existing and window.get_wm_name() == "HIBI AI" and window.get_attributes().map_state == X.IsViewable:
                    self.window = window
                    self.window.set_input_focus(X.RevertToParent, X.CurrentTime)
                    self.d.sync()
                    self.expect("Your workspace")
                    return
            time.sleep(.1)
        raise TimeoutError("Installer did not open its X11 window")

    def capture(self):
        self.window.warp_pointer(8, 8)
        self.d.sync()
        time.sleep(.08)
        geometry = self.window.get_geometry()
        raw = self.window.get_image(0, 0, geometry.width, geometry.height, X.ZPixmap, 0xffffffff)
        return Image.frombytes("RGB", (geometry.width, geometry.height), raw.data, "raw", "BGRX")

    def screenshot(self, name):
        path = self.artifacts / f"{name}.png"
        self.capture().save(path)
        return path

    def words(self):
        image = self.capture()
        key = image.tobytes()
        if self.ocr_cache and self.ocr_cache[0] == key:
            return self.ocr_cache[1]
        # Upscaling helps small secondary labels. Keep color so Tesseract can
        # independently recognize light and dark text on colored buttons.
        words = []
        regions = [(image, 0, 0, "11")]
        # Sparse-page OCR may ignore white text inside filled action buttons.
        # Recognize their saturated background rectangles independently.
        for count, color in sorted(image.getcolors(image.width * image.height), reverse=True)[:24]:
            if count < 400 or max(color) - min(color) < 55:
                continue
            difference = ImageChops.difference(image, Image.new("RGB", image.size, color))
            mask = difference.convert("L").point(lambda value: 255 if value == 0 else 0)
            _, visible_rows = mask.getprojection()
            starts = []
            for position, visible in enumerate(visible_rows + [0]):
                if visible and not starts:
                    starts.append(position)
                elif not visible and starts:
                    top = starts.pop()
                    if position - top < 18:
                        continue
                    band = mask.crop((0, top, image.width, position))
                    columns, _ = band.getprojection()
                    left = None
                    for x, present in enumerate(columns + [0]):
                        if present and left is None:
                            left = x
                        elif not present and left is not None:
                            if 30 < x - left < 500:
                                box = (left+3, top+3, x-3, position-3)
                                gray = ImageOps.grayscale(image.crop(box))
                                background = .299*color[0] + .587*color[1] + .114*color[2]
                                if background < 150:
                                    gray = gray.point(lambda value: 0 if value > 180 else 255)
                                else:
                                    gray = gray.point(lambda value: 0 if value < 100 else 255)
                                regions.append((ImageOps.expand(gray, border=8, fill=255), box[0]-8, box[1]-8, "7"))
                            left = None
        for number, (region, offset_x, offset_y, psm) in enumerate(regions):
            path = self.artifacts / f"ocr-input-{number}.png"
            region.resize((region.width * 3, region.height * 3)).save(path)
            result = subprocess.run([self.args.tesseract, str(path), "stdout", "--psm", psm,
                                     "-c", "tessedit_create_tsv=1"], text=True, capture_output=True, check=True)
            for item in csv.DictReader(io.StringIO(result.stdout), delimiter="\t", quoting=csv.QUOTE_NONE):
                if item.get("text", "").strip():
                    words.append(dict(text=item["text"], x=int(item["left"])/3 + offset_x,
                                      y=int(item["top"])/3 + offset_y,
                                      w=int(item["width"])/3, h=int(item["height"])/3))
        self.ocr_cache = (key, words)
        return words

    def text(self):
        return " ".join(word["text"] for word in self.words())

    def expect(self, label, timeout=8):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if normalized(label) in normalized(self.text()):
                return
            time.sleep(.1)
        self.screenshot("failure")
        raise AssertionError(f"Expected text {label!r}; recognized: {self.text()}")

    def find(self, label, region=None):
        words = self.words()
        wanted = normalized(label)
        for index, first in enumerate(words):
            if region and not (region[0] <= first["x"] <= region[2] and region[1] <= first["y"] <= region[3]):
                continue
            phrase = ""
            matches = []
            for word in words[index:index+12]:
                if abs(word["y"] - first["y"]) > max(12, first["h"]):
                    break
                phrase += normalized(word["text"])
                matches.append(word)
                if phrase == wanted:
                    right = max(w["x"] + w["w"] for w in matches)
                    bottom = max(w["y"] + w["h"] for w in matches)
                    return first["x"], first["y"], right-first["x"], bottom-first["y"]
                if not wanted.startswith(phrase):
                    break
        raise AssertionError(f"Could not locate {label!r}; recognized: {self.text()}")

    def click(self, x, y):
        self.window.warp_pointer(round(x), round(y))
        self.d.sync(); time.sleep(.08)
        xtest.fake_input(self.d, X.ButtonPress, 1)
        self.d.sync(); time.sleep(.04)
        if not self.window.query_pointer().mask & X.Button1Mask:
            raise RuntimeError("Synthetic clicks are blocked by this display. Use isolated Xvfb.")
        xtest.fake_input(self.d, X.ButtonRelease, 1)
        self.d.sync(); time.sleep(.15)
        self.ocr_cache = None

    def click_text(self, label, region=None):
        x, y, w, h = self.find(label, region)
        self.click(x+w/2, y+h/2)

    def click_theme(self):
        geometry = self.window.get_geometry()
        padding = 14 if geometry.height < 680 else 20
        self.click(geometry.width - padding - 18, padding + 18)

    def expect_theme(self, theme):
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            dark = sum(self.capture().getpixel((0, 0))) < 300
            if dark == (theme == "dark"):
                return
            time.sleep(.1)
        self.screenshot("failure")
        raise AssertionError(f"Expected {theme} theme")

    def key(self, name, shift=False, control=False, logo=False):
        controlcode = self.d.keysym_to_keycode(XK.string_to_keysym("Control_L"))
        if control:
            xtest.fake_input(self.d, X.KeyPress, controlcode)
        logocode = self.d.keysym_to_keycode(XK.string_to_keysym("Super_L"))
        if logo:
            xtest.fake_input(self.d, X.KeyPress, logocode)
        shiftcode = self.d.keysym_to_keycode(XK.string_to_keysym("Shift_L"))
        if shift:
            xtest.fake_input(self.d, X.KeyPress, shiftcode)
        code = self.d.keysym_to_keycode(XK.string_to_keysym(name))
        xtest.fake_input(self.d, X.KeyPress, code)
        self.d.sync(); time.sleep(.025)
        xtest.fake_input(self.d, X.KeyRelease, code)
        if shift:
            xtest.fake_input(self.d, X.KeyRelease, shiftcode)
        if control:
            xtest.fake_input(self.d, X.KeyRelease, controlcode)
        if logo:
            xtest.fake_input(self.d, X.KeyRelease, logocode)
        self.d.sync(); time.sleep(.12)
        self.ocr_cache = None

    def search(self, query, submit=True):
        self.key("f", control=True)
        self.key("a", control=True)
        self.key("BackSpace")
        for char in query:
            self.key({"-": "minus", "_": "minus", ".": "period", " ": "space"}.get(char, char.lower()), shift=char.isupper() or char == "_")
        if submit:
            self.key("Return")

    def clear_search(self):
        # Escape must clear a nonempty query while retaining the environment.
        self.key("Escape")
        self.expect("COMPONENTS")

    def select_status(self, label):
        x, y, width, height = self.find("Filters", (220, 70, 1120, 350))
        self.click(x + width/2, y + height/2)
        self.click_text(label, (max(220, x-260), y+height+4, 1120, 560))
        self.click_text("Filters", (220, 70, 1120, 350))

    def text_selection_checks(self):
        match = re.search(r"(\d+)\s+selected", self.text())
        assert match, self.text()
        selected = int(match.group(1))
        for theme in ("dark", "light"):
            self.click_theme()
            self.expect_theme(theme)
            for modifier in ("control", "logo"):
                keys = {modifier: True}
                self.search("architect", submit=False)
                # Select-all alone must not insert an 'a' or affect row selection.
                self.key("a", **keys); self.key("a", **keys)
                self.key("Return")
                self.expect("1 result"); self.expect("architect.md")
                self.expect("1 selected")
                self.click_text("architect", (220, 130, 1120, 220))
                self.key("a", **keys)
                for char in "auditor": self.key(char)
                self.key("Return")
                self.expect("1 result"); self.expect("assurance-auditor.md")
                self.expect("1 selected")
                # Enter returns focus to the list. Command+A must not toggle rows.
                self.key("a", **keys)
                self.expect("1 result"); self.expect("1 selected")
                self.clear_search()
                self.expect(f"{selected} selected")
            self.passed(f"03j-{theme}-search-text-select-all")
        self.search("architect", submit=False)
        for modifier in ("control", "logo"):
            keys = {modifier: True}
            self.key("a", **keys); self.key("c", **keys)
            self.key("Return")
            self.expect("1 result"); self.expect("architect.md")
            self.key("f", control=True)
            self.key("a", **keys); self.key("x", **keys)
            self.expect(f"{selected} selected")
            self.key("v", **keys)
            self.key("Return")
            self.expect("1 result"); self.expect("architect.md")
            self.expect("1 selected")
            self.key("f", control=True)
        self.key("Return")
        self.clear_search()
        self.expect(f"{selected} selected")
        self.passed("03k-search-text-copy-cut-paste")

    def search_checks(self):
        match = re.search(r"(\d+)\s+selected", self.text())
        assert match, self.text()
        selected = int(match.group(1))
        assert selected > 1, "The fixture needs selected rows outside a search result"
        self.search("ARCHITECT")
        self.expect("1 selected")
        self.expect("architect.md")
        self.passed("03c-search-case-insensitive")
        self.click_text("Clear selection", (220, 300, 1120, 780))
        self.expect("0 selected")
        self.key("f", control=True)
        self.clear_search()
        self.expect(f"{selected - 1} selected")
        self.key("a")
        self.expect(f"{selected} selected")
        self.passed("03d-search-preserves-hidden-selections-and-focused-escape")
        self.search("BuNdLeD")
        self.expect(f"{selected} selected")
        self.passed("03e-search-by-source")
        self.search("architect")
        self.select_status("Modified")
        self.expect("No matching items")
        self.expect("0 selected")
        self.select_status("Default")
        self.expect("1 selected")
        self.passed("03f-search-and-status-filter-intersection")
        self.search("hibi-search-no-matches")
        self.expect("No matching items")
        self.expect("0 selected")
        self.inert("Install")
        self.inert("Remove")
        self.key("i")
        self.expect("No matching items")
        self.key("r")
        self.expect("No matching items")
        assert not list((self.home / ".claude/agents").glob("*.md")), "A shortcut processed selected rows hidden by an empty search result"
        self.passed("03g-search-no-results-disabled-actions")
        self.clear_search()
        self.expect(f"{selected} selected")
        # Navigate by keyboard so metadata search also exercises original-index routing.
        self.key("1")
        for _ in range(9): self.key("Down")
        self.key("2")
        self.search("MEMORY_FILE_PATH")
        self.expect("1 result")
        self.find("memory", (220, 250, 1120, 640))
        self.click_text("memory", (220, 250, 1120, 640)); self.key("space")
        self.expect("1 selected")
        self.search("framework")
        self.expect("2 results")
        self.expect("0 selected")
        self.key("a")
        self.clear_search()
        self.expect("3 selected")
        self.passed("03h-mcp-metadata-and-hidden-selection")
        self.key("1"); self.key("Down"); self.key("2")
        self.search("anthropics rust")
        self.expect("1 result")
        self.expect("rust-analyzer")
        self.search("hibi-plugin-no-matches")
        self.expect("No matching items")
        assert "rustanalyzer" not in normalized(self.text()), "Hidden plugin metadata leaked into the empty-result inspector"
        self.passed("03i-plugin-metadata-and-empty-inspector")
        self.clear_search()
        self.key("1"); self.key("Down"); self.key("2")
        self.expect("Agents")
        self.expect(f"{selected} selected")

    def wheel(self, x, y, steps):
        self.window.warp_pointer(x, y)
        self.d.sync()
        for _ in range(abs(steps)):
            button = 5 if steps > 0 else 4
            xtest.fake_input(self.d, X.ButtonPress, button)
            xtest.fake_input(self.d, X.ButtonRelease, button)
        self.d.sync(); time.sleep(.15)
        self.ocr_cache = None

    def resize(self, width, height):
        self.window.configure(width=width, height=height)
        self.d.sync(); time.sleep(.25)
        self.ocr_cache = None

    def inert(self, label):
        before = self.capture()
        try:
            self.click_text(label)
        except AssertionError:
            if label != "Install":
                raise
            # Disabled primary text may be too faint for OCR. Its button is
            # immediately after Remove in the fixed action group.
            x, y, w, h = self.find("Remove", (220, 300, 1120, 780))
            self.click(x + w + 50, y + h / 2)
        after = self.capture()
        assert ImageChops.difference(before, after).getbbox() is None, f"Disabled {label} changed the screen"

    def clear_selection(self):
        for _ in range(3):
            if "0selected" in normalized(self.text()):
                return
            self.key("a")
        raise AssertionError("Could not clear selection in the current view")

    def back_to_home(self):
        self.click_text("Switch CLI")
        if "leavethisenvironment" in normalized(self.text()):
            self.click_text("Discard selections")
        self.expect("Your workspace")

    def wait_for_appearance(self, appearance):
        deadline = time.monotonic() + 8
        path = self.home / ".hibi/preferences.json"
        while time.monotonic() < deadline:
            try:
                if json.loads(path.read_text()).get("appearance") == appearance:
                    return
            except (FileNotFoundError, json.JSONDecodeError):
                pass
            time.sleep(.1)
        raise AssertionError(f"Appearance {appearance} was not persisted")

    def close_window(self):
        self.window.send_event(protocol.event.ClientMessage(
            window=self.window.id,
            client_type=self.d.intern_atom("WM_PROTOCOLS"),
            data=(32, [self.d.intern_atom("WM_DELETE_WINDOW"), X.CurrentTime, 0, 0, 0]),
        ))
        self.d.sync()
        time.sleep(.2)
        if self.process.poll() is None:
            self.click_text("Close application")
        self.process.wait(timeout=8)
        assert self.process.returncode == 0
        self.log.close()
        self.ocr_cache = None

    def passed(self, name):
        self.screenshot(name)
        self.results.append(name)
        print(f"PASS {name}", flush=True)

    def run(self):
        self.launch()
        self.passed("01-dark-home")
        self.click_theme()
        self.expect_theme("light")
        assert sum(self.capture().getpixel((0, 0))) > 600
        self.passed("02-light-home")
        self.wait_for_appearance("light")
        self.close_window()
        self.launch()
        self.expect_theme("light")
        self.passed("02b-light-theme-restored")
        self.click_text("Claude Code")
        self.expect("COMPONENTS")
        self.expect("Agents")
        # A normal-size capture records the entire category sidebar for visual review.
        self.passed("03-claude-sidebar")
        self.click_theme()
        self.expect_theme("dark")
        self.passed("03a-dark-claude-sidebar")
        self.click_theme()
        self.expect_theme("light")
        sidebar_text = " ".join(w["text"] for w in self.words() if w["x"] < 210)
        self.wheel(140, 420, 6)
        sidebar_text += " " + " ".join(w["text"] for w in self.words() if w["x"] < 210)
        for category in ["Agents", "Commands", "Contexts", "Rules", "Skills", "Hooks", "Styles", "Statusline", "Config", "MCP", "Plugins"]:
            assert normalized(category) in normalized(sidebar_text), (category, sidebar_text)
        self.passed("03b-all-eleven-categories-reachable")
        self.wheel(140, 420, -10)
        self.text_selection_checks()
        self.search_checks()
        self.clear_selection()
        self.inert("Install")
        self.inert("Remove")
        self.passed("04-zero-selection-disabled")
        self.select_status("Modified")
        self.expect("Nothing to show")
        self.expect("0 selected")
        self.passed("05-empty-filter")
        self.select_status("Default")
        self.expect("architect.md")
        self.clear_selection()
        # Keep nonmatching rows selected to prove search constrains installation.
        self.key("a")
        self.search("architect")
        self.expect("1 selected")
        self.click_text("Diff", (220, 300, 1120, 780))
        self.expect("Compare configuration")
        before = self.capture()
        self.key("Down"); self.key("Down")
        assert ImageChops.difference(before, self.capture()).getbbox(), "Diff did not scroll"
        self.passed("06-diff-keyboard-scroll")
        self.click_text("Back to components")
        self.click_text("Install", (220, 300, 1120, 780))
        installed = self.home / ".claude/agents/architect.md"
        self.expect("Operation complete", timeout=15)
        assert installed.is_file(), "Selected bundled component was not installed"
        assert sorted(path.name for path in installed.parent.glob("*.md")) == ["architect.md"], "Search installed a selected but hidden component"
        manifest = json.loads((self.home / ".hibi/install.json").read_text())
        assert manifest["components"] == ["agents/architect.md"], manifest
        self.passed("07-install-and-manifest")
        self.click_text("Back to components")
        self.expect("COMPONENTS")
        self.clear_search()
        self.clear_selection()
        self.search("assurance-auditor")
        self.click_text("assurance-auditor.md"); self.key("space")
        self.click_text("Install", (220, 300, 1120, 780))
        self.expect("Operation complete", timeout=15)
        other_installed = installed.parent / "assurance-auditor.md"
        assert other_installed.is_file()
        self.click_text("Back to components")
        self.clear_search()
        self.clear_selection()
        self.click_text("architect.md"); self.key("space")
        self.search("assurance-auditor")
        self.click_text("assurance-auditor.md"); self.key("space")
        self.search("architect")
        self.expect("1 selected")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.expect("Remove 1 selected items")
        self.click_text("Cancel")
        assert installed.is_file() and other_installed.is_file(), "Cancelling removal changed an installed file"
        self.expect("COMPONENTS")
        self.passed("08-remove-cancel")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.click_text("Remove items")
        self.expect("Operation complete", timeout=15)
        assert not installed.exists(), "Confirmed removal left the component installed"
        manifest = json.loads((self.home / ".hibi/install.json").read_text())
        assert manifest["components"] == ["agents/assurance-auditor.md"], manifest
        assert other_installed.is_file(), "Search removed a selected but hidden installed file"
        self.passed("09-remove-confirm-and-manifest")
        self.click_text("Back to components")
        self.clear_search()
        self.search("assurance-auditor")
        self.click_text("assurance-auditor.md"); self.key("space")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.click_text("Remove items")
        self.expect("Operation complete", timeout=15)
        assert not other_installed.exists()
        self.click_text("Back to components")
        self.clear_search()
        self.click_text("Keyboard shortcuts")
        self.expect("Selection")
        before = self.capture()
        for _ in range(8): self.key("Down")
        assert ImageChops.difference(before, self.capture()).getbbox(), "Help did not scroll"
        self.passed("10-help-keyboard-scroll")
        self.click_text("Back", (800, 70, 1120, 200))
        self.resize(760, 560)
        self.expect("Install")
        self.passed("11-minimum-window")
        self.resize(1120, 780)
        self.back_to_home()
        self.click_text("Codex CLI")
        self.expect("COMPONENTS")
        sidebar = " ".join(w["text"] for w in self.words() if w["x"] < 210)
        assert all(name in sidebar for name in ["Skills", "Config", "MCP"]), sidebar
        assert all(name not in sidebar for name in ["Agents", "Commands", "Plugins"]), sidebar
        self.passed("12-codex-three-categories")
        self.back_to_home()
        self.click_text("Manage sources")
        self.expect("Bundled library")
        self.inert("Remove source")
        self.inert("Edit source")
        self.passed("13-bundled-source-protected")
        self.click_text("Add source")
        self.click_text("Git repository")
        self.expect("Connect a Git repository")
        self.click_text("Continue")
        self.expect("Only HTTPS git URLs are allowed")
        self.passed("14-invalid-empty-git-url")
        self.click_text("Cancel")
        assert not (self.home / ".hibi/sources.yaml").exists()
        self.click_text("Add source")
        self.click_text("Local folder")
        self.click_text("Continue")
        self.expect("Choose a local folder")
        self.expect("Local source path is empty")
        self.passed("15-empty-local-path")
        self.click_text("Cancel")
        self.expect("Bundled library")
        self.click_text("Back", (850, 600, 1120, 780))
        self.expect("Your workspace")
        self.click_theme()
        self.expect_theme("dark")
        self.passed("16-dark-home-final")
        assert sum(self.capture().getpixel((0, 0))) < 150
        self.wait_for_appearance("dark")
        self.close_window()
        self.launch()
        self.expect_theme("dark")
        self.passed("17-dark-theme-restored")
        self.close_window()
        (self.artifacts / "results.json").write_text(json.dumps({"passed": self.results, "isolated_home": str(self.home)}, indent=2))

    def close(self):
        if self.process and self.process.poll() is None:
            self.process.terminate()
            try: self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill(); self.process.wait(timeout=5)
        if self.log: self.log.close()
        self.d.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--display", default=os.environ.get("DISPLAY", ":99"))
    parser.add_argument("--artifacts", type=Path, default=Path(tempfile.mkdtemp(prefix="hibi-gui-smoke-")))
    parser.add_argument("--tesseract", default=shutil.which("tesseract"))
    args = parser.parse_args()
    if not args.binary.is_file(): parser.error("Build the GUI binary first, then pass its path with --binary")
    if not args.tesseract: parser.error("Tesseract with English language data is required; install it or use --tesseract /path/to/tesseract")
    languages = subprocess.run([args.tesseract, "--list-langs"], text=True, capture_output=True)
    if languages.returncode or "eng" not in languages.stdout.split():
        parser.error("Tesseract English language data is required; check TESSDATA_PREFIX or install its English language pack")
    desktop = Desktop(args)
    try:
        desktop.run()
    except Exception:
        if desktop.window:
            try: desktop.screenshot("failure")
            except Exception: pass
        raise
    finally:
        desktop.close()
    print(f"GUI smoke passed. Artifacts: {args.artifacts.resolve()}")


if __name__ == "__main__":
    main()
