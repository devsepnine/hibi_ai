#!/usr/bin/env python3
"""Exercise the desktop installer through X11 with an isolated home.

Requires python-xlib, Pillow, Tesseract with English data, and an X11 display.
Use Xvfb rather than a user's desktop to make synthetic input reliable.
Example: python tools/installer/tests/gui_smoke.py --binary tools/installer/target/debug/hibi --display :99 --artifacts /tmp/hibi-gui-smoke
"""

import argparse
import csv
import fcntl
import hashlib
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


def text_layer(image, dark):
    # Suppress button outlines while retaining antialiased letter strokes.
    ink = ImageOps.grayscale(image)
    if dark:
        ink = ImageOps.invert(ink)
    return ink.point(lambda value: (value - 25) * 255 / (130 - 25))


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
        env.update(LANGUAGE="en", HOME=str(self.home), DISPLAY=self.args.display, WAYLAND_DISPLAY="",
                   XDG_CONFIG_HOME=str(self.home / ".config"), XDG_CACHE_HOME=str(self.home / ".cache"),
                   XDG_DATA_HOME=str(self.home / ".local/share"), PATH=f"{stub}:/usr/bin:/bin",
                   CLAUDE_CONFIG_DIR=str(self.home / ".claude"))
        self.log = (self.artifacts / "application.log").open("a")
        existing = {w.id for w in self.root.query_tree().children}
        self.process = subprocess.Popen([str(self.args.binary.resolve())], env=env,
                                        cwd=Path(__file__).resolve().parents[3],
                                        start_new_session=True, stdin=subprocess.DEVNULL,
                                        stdout=self.log, stderr=self.log)
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

    def capture(self, *, park_pointer=True):
        if park_pointer:
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

    def words(self, image=None):
        if image is None:
            image = self.capture(park_pointer=False)
        key = image.tobytes()
        if self.ocr_cache and self.ocr_cache[0] == key:
            return self.ocr_cache[1]
        # Upscaling helps small secondary labels. Keep color so Tesseract can
        # independently recognize light and dark text on colored buttons.
        regions = [(image, 0, 0, "11")]
        dark = sum(image.getpixel((0, 0))) < 300
        regions.append((text_layer(image, dark), 0, 0, "11"))
        # The selection count can be skipped in sparse-page recognition.
        # Read the work bar independently of the results and sidebar.
        bar_top = image.height - 126
        regions.append((image.crop((245, bar_top, image.width - 12, image.height - 64)),
                        245, bar_top, "11"))
        # A single text line removes the glass buttons' top and bottom borders.
        actions_left = image.width - 360
        regions.append((image.crop((actions_left, bar_top + 16, image.width - 40, bar_top + 41)),
                        actions_left, bar_top + 16, "7"))
        install_left = image.width - 125
        regions.append((image.crop((install_left, bar_top + 8, image.width - 23, image.height - 75)),
                        install_left, bar_top + 8, "7"))
        # Recognize labels inside solid buttons independently, including neutral palette fills.
        for count, color in sorted(image.getcolors(image.width * image.height), reverse=True)[:24]:
            if count < 400:
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
                    if not 18 <= position - top <= 80:
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
                                if mask.crop(box).histogram()[255] < (box[2]-box[0]) * (box[3]-box[1]) / 2:
                                    left = None
                                    continue
                                gray = ImageOps.grayscale(image.crop(box))
                                background = .299*color[0] + .587*color[1] + .114*color[2]
                                if background < 150:
                                    gray = ImageOps.invert(gray)
                                gray = ImageOps.autocontrast(gray)
                                regions.append((ImageOps.expand(gray, border=8, fill=255), box[0]-8, box[1]-8, "7"))
                            left = None
        words = self.recognize(regions, image.height)
        self.ocr_cache = (key, words)
        return words

    def recognize(self, regions, window_height):
        words = []
        for number, (region, offset_x, offset_y, psm) in enumerate(regions):
            path = self.artifacts / f"ocr-input-{number}.png"
            region.resize((region.width * 3, region.height * 3)).save(path)
            result = subprocess.run([self.args.tesseract, str(path), "stdout", "--psm", psm,
                                     "-c", "tessedit_create_tsv=1"], text=True, capture_output=True, check=True)
            for item in csv.DictReader(io.StringIO(result.stdout), delimiter="\t", quoting=csv.QUOTE_NONE):
                label = item.get("text", "").strip()
                if label:
                    # Tesseract can confuse the small Install button's t and l glyphs.
                    if normalized(label) in {"inslall", "loslall"} and offset_y + int(item["top"])/3 > window_height - 100:
                        label = "Install"
                    words.append(dict(text=label, x=int(item["left"])/3 + offset_x,
                                      y=int(item["top"])/3 + offset_y,
                                      w=int(item["width"])/3, h=int(item["height"])/3))
        return words

    def text(self, image=None):
        return " ".join(word["text"] for word in self.words(image))

    def expect(self, label, timeout=8):
        return self.expect_any((label,), timeout)

    def observation_frames(self, timeout):
        # Parking the pointer can trigger a redraw after the last OCR sample.
        for park_pointer in (False, True):
            frame = self.capture(park_pointer=park_pointer)
            deadline = time.monotonic() + timeout
            while True:
                expired = time.monotonic() >= deadline
                yield frame
                if expired:
                    break
                time.sleep(.1)
                frame = self.capture(park_pointer=False)
        frame.save(self.artifacts / "last-unmatched-frame.png")

    def expect_any(self, labels, timeout=8):
        for frame in self.observation_frames(timeout):
            recognized = self.text(frame)
            screen = normalized(recognized)
            for label in labels:
                if normalized(label) in screen:
                    return label
        raise AssertionError(f"Expected one of {labels!r}; recognized: {recognized}")

    def find(self, label, region=None, timeout=8):
        _, bounds = self.find_any((label,), region, timeout)
        return bounds

    def find_any(self, labels, region, timeout=8):
        for frame in self.observation_frames(timeout):
            words = self.words(frame)
            for label in labels:
                bounds = self.label_bounds(words, label, region)
                if bounds is not None:
                    return label, bounds
            if region:
                # Read the requested region as a block when page OCR skips a bordered field.
                crop = frame.crop(region)
                dark = sum(frame.getpixel((0, 0))) < 300
                nearby = self.recognize(
                    [(layer, *region[:2], "6") for layer in (crop, text_layer(crop, dark))],
                    frame.height,
                )
                for label in labels:
                    bounds = self.label_bounds(nearby, label, region)
                    if bounds is not None:
                        return label, bounds
        recognized = " ".join(word["text"] for word in words)
        raise AssertionError(f"Could not locate any of {labels!r}; recognized: {recognized}")

    @staticmethod
    def label_bounds(words, label, region):
        wanted = normalized(label)
        fallback = None
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
                    bounds = first["x"], first["y"], right-first["x"], bottom-first["y"]
                    if " ".join(w["text"] for w in matches) == label:
                        return bounds
                    fallback = fallback or bounds
                    break
                if not wanted.startswith(phrase):
                    break
        return fallback

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
        self.click(geometry.width - 4 - 8 - 18, 4 + 18)

    def expect_theme(self, theme):
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            dark = sum(self.capture(park_pointer=False).getpixel((0, 0))) < 300
            if dark == (theme == "dark"):
                return
            time.sleep(.1)
        self.screenshot("failure")
        raise AssertionError(f"Expected {theme} theme")

    def expect_scroll(self, before, label):
        # Both scroll panes fill the center of the window below the dialog title.
        viewport = (before.width // 8, before.height // 4,
                    before.width * 7 // 8, before.height * 3 // 4)
        previous = before.crop(viewport)
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            current = self.capture(park_pointer=False).crop(viewport)
            if ImageChops.difference(previous, current).getbbox():
                return
            time.sleep(.1)
        raise AssertionError(f"{label} did not scroll")

    def check_sidebar_repaint(self):
        # The panel margin should stay stable while hover redraws the sidebar rows.
        margin = (30, 175, 37, 620)
        before = self.capture().crop(margin)
        for label in ("Agents", "Commands", "Skills", "Hooks", "Styles"):
            x, y, w, h = self.find(label, (40, 200, 210, 620))
            self.window.warp_pointer(round(x + w/2), round(y + h/2))
            self.d.sync(); time.sleep(.15)
        after = self.capture().crop(margin)
        assert ImageChops.difference(before, after).getbbox() is None, "Sidebar glass changed outside hover rows"

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

    def expect_clipboard(self, expected):
        connection = display.Display(self.args.display)
        receiver = connection.screen().root.create_window(0, 0, 1, 1, 0, X.CopyFromParent)
        try:
            clipboard = connection.intern_atom("CLIPBOARD")
            utf8 = connection.intern_atom("UTF8_STRING")
            property_atom = connection.intern_atom("HIBI_GUI_SMOKE_CLIPBOARD")
            deadline = time.monotonic() + 8
            while time.monotonic() < deadline:
                receiver.convert_selection(clipboard, utf8, property_atom, X.CurrentTime)
                connection.flush()
                reply_deadline = min(deadline, time.monotonic() + .5)
                while time.monotonic() < reply_deadline:
                    if not connection.pending_events():
                        time.sleep(.03)
                        continue
                    reply = connection.next_event()
                    if reply.type != X.SelectionNotify:
                        continue
                    if reply.property == property_atom:
                        content = receiver.get_full_property(property_atom, utf8)
                        receiver.delete_property(property_atom)
                        if content is not None and content.value == expected.encode("utf-8"):
                            return
                    break
                time.sleep(.1)
            raise AssertionError(f"X11 clipboard did not receive {expected!r}")
        finally:
            connection.close()

    def search(self, query, submit=True):
        self.key("f", control=True)
        field = self.expect_search_focus()
        self.key("a", control=True)
        self.key("BackSpace")
        for char in query:
            self.key({"-": "minus", "_": "minus", ".": "period", " ": "space"}.get(char, char.lower()), shift=char.isupper() or char == "_")
        if submit:
            self.key("Return")
        return field

    def expect_search_focus(self):
        # The search outline is visible as soon as the input can receive keys.
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            image = self.capture(park_pointer=False)
            dark = sum(image.getpixel((0, 0))) < 300
            accent = (250, 250, 250) if dark else (63, 61, 218)
            center = image.crop((365, 88, 515, 210))
            for y in range(center.height):
                if center.crop((0, y, center.width, y + 1)).getcolors(1) == [(center.width, accent)]:
                    return 365 + center.width // 2, 88 + y + 8
            time.sleep(.1)
        self.screenshot("search-focus-failure")
        raise AssertionError("Search input did not show keyboard focus")

    def expect_result_count(self, count, timeout=15):
        width = self.window.get_geometry().width
        expected = f"{count} result" if count == 1 else f"{count} results"
        self.find(expected, (width - 145, 73, width - 8, 121), timeout)

    def clear_search(self):
        self.key("Escape")
        self.expect_components()
        # Selection counts are scoped to the query until the field is cleared.
        self.find("Search this category", (240, 95, 700, 210), timeout=15)

    def expect_components(self):
        self.find("Components", (15, 80, 230, 280), timeout=20)

    def close_diff(self):
        self.key("Escape")
        self.expect_components()

    def select_status(self, label):
        x, y, width, height = self.find("Filters", (220, 70, 1120, 350))
        self.click(x + width/2, y + height/2)
        self.click_text(label, (max(220, x-260), y+height+4, 1120, 560))
        self.click_text("Filters", (220, 70, 1120, 350))

    def reset_status(self):
        self.key("f")
        self.find("Visibility", (800, 180, 1120, 560))
        self.key("Return")
        self.key("Escape")
        self.find("Filters Default", (800, 70, 1120, 220))

    def text_selection_checks(self):
        match = re.search(r"(\d+)\s+selected", self.text())
        assert match, self.text()
        selected = int(match.group(1))
        for theme in ("dark", "light"):
            self.click_theme()
            self.expect_theme(theme)
            for modifier in ("control", "logo"):
                keys = {modifier: True}
                field = self.search("architect", submit=False)
                self.expect_result_count(1)
                # Select-all alone must not insert an 'a' or affect row selection.
                self.key("a", **keys); self.key("a", **keys)
                self.key("Return")
                self.expect_result_count(1); self.expect("architect.md")
                self.expect("1 selected")
                self.click(*field)
                self.expect_search_focus()
                self.key("a", **keys)
                for char in "auditor": self.key(char)
                self.key("Return")
                self.expect_result_count(1); self.expect("assurance-auditor.md")
                self.expect("1 selected")
                # Enter returns focus to the list. Command+A must not toggle rows.
                self.key("a", **keys)
                self.expect_result_count(1); self.expect("1 selected")
                self.clear_search()
                self.expect(f"{selected} selected")
            self.passed(f"03j-{theme}-search-text-select-all")
        self.clipboard_shortcuts(selected)

    def clipboard_shortcuts(self, selected):
        for modifier, (copy_query, copy_file), (cut_query, cut_file) in (
            ("control", ("architect", "architect.md"), ("auditor", "assurance-auditor.md")),
            ("logo", ("e2e", "e2e-runner.md"), ("refactor", "refactor-cleaner.md")),
        ):
            keys = {modifier: True}
            self.search(copy_query, submit=False)
            self.expect_result_count(1)
            self.expect(copy_file)
            self.key("a", **keys)
            self.key("c", **keys)
            self.expect_clipboard(copy_query)
            self.search(cut_query, submit=False)
            self.expect_result_count(1)
            self.expect(cut_file)
            self.key("a", **keys)
            self.key("x", **keys)
            self.expect_clipboard(cut_query)
            self.expect(f"{selected} selected")
            self.key("v", **keys)
            self.key("Return")
            self.expect_result_count(1)
            self.expect(cut_file)
            self.expect("1 selected")
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
        self.expect_result_count(selected)
        self.expect(f"{selected} selected")
        self.passed("03e-search-by-source")
        self.search("architect")
        self.expect_result_count(1)
        self.select_status("Modified")
        self.expect("No matching items")
        self.expect("0 selected")
        self.reset_status()
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
        self.find("MCP", (235, 70, 550, 115))
        self.search("MEMORY_FILE_PATH")
        self.expect_result_count(1)
        self.select_one_result("memory")
        self.search("framework")
        self.expect_result_count(2)
        self.expect("0 selected")
        self.key("a")
        self.clear_search()
        self.expect("3 selected")
        self.passed("03h-mcp-metadata-and-hidden-selection")
        self.key("1"); self.key("Down"); self.key("2")
        self.find("Plugins", (235, 70, 550, 115))
        self.search("anthropics rust")
        self.expect_result_count(1)
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
        self.d.sync()
        self.ocr_cache = None
        # A window may already have its new size while still showing the old layout.
        self.find("Install", (410, height - 126, width, height - 64), timeout=15)

    def inert(self, label, compare_region=None):
        before = self.capture()
        left, top, right, bottom = compare_region or (0, 52, before.width, before.height)
        # Hover and focus can change the work bar and footer without starting an action.
        content = (left, top, right, min(bottom, before.height - 134))
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
        assert ImageChops.difference(before.crop(content), after.crop(content)).getbbox() is None, f"Disabled {label} changed the content"

    def clear_selection(self):
        footer = (220, 300, 1120, 780)
        action, _ = self.find_any(("0 selected", "Clear selection", "Select all"), footer)
        if action == "0 selected":
            return
        if action == "Select all":
            self.click_text("Select all", footer)
        self.click_text("Clear selection", footer)
        self.find("0 selected", footer)

    def select_one_result(self, label):
        self.expect_result_count(1)
        self.click_text(label, (235, 218, 1100, 650))
        footer = (220, 600, 1120, 780)
        action, _ = self.find_any(("1 selected", "Select all"), footer)
        if action == "Select all":
            self.click_text("Select all", footer)
            self.find("1 selected", footer)

    def confirm_single_removal(self, label):
        self.find("Remove 1 selected item?")
        self.expect(label)
        self.key("y")

    def back_to_home(self):
        self.click_text("Switch CLI", (35, 80, 215, 205))
        screen = self.expect_any(("Your workspace", "Leave this environment"), timeout=20)
        if screen == "Leave this environment":
            self.key("y")
            self.expect("Your workspace", timeout=20)

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
        self.expect("System theme")
        if sum(self.capture().getpixel((0, 0))) >= 300:
            self.click_theme()
        self.expect_theme("dark")
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
        self.expect_components()
        self.expect("Agents")
        # A normal-size capture records the entire category sidebar for visual review.
        self.passed("03-claude-sidebar")
        self.check_sidebar_repaint()
        self.click_theme()
        self.expect_theme("dark")
        self.passed("03a-dark-claude-sidebar")
        self.check_sidebar_repaint()
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
        self.reset_status()
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
        self.expect_scroll(before, "Diff")
        self.passed("06-diff-keyboard-scroll")
        self.close_diff()
        self.click_text("Install", (220, 300, 1120, 780))
        installed = self.home / ".claude/agents/architect.md"
        self.expect("Operation complete", timeout=15)
        assert installed.is_file(), "Selected bundled component was not installed"
        assert sorted(path.name for path in installed.parent.glob("*.md")) == ["architect.md"], "Search installed a selected but hidden component"
        manifest = json.loads((self.home / ".hibi/install.json").read_text())
        assert manifest["components"] == ["agents/architect.md"], manifest
        self.passed("07-install-and-manifest")
        self.click_text("Back to components")
        self.expect_components()
        self.clear_search()
        self.clear_selection()
        self.search("assurance-auditor")
        self.select_one_result("assurance-auditor.md")
        self.click_text("Install", (220, 300, 1120, 780))
        self.expect("Operation complete", timeout=15)
        other_installed = installed.parent / "assurance-auditor.md"
        assert other_installed.is_file()
        self.click_text("Back to components")
        self.clear_search()
        self.clear_selection()
        self.search("architect")
        self.select_one_result("architect.md")
        self.search("assurance-auditor")
        self.select_one_result("assurance-auditor.md")
        self.search("architect")
        self.expect("1 selected")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.find("Remove 1 selected item?")
        self.expect("architect.md")
        assert self.label_bounds(self.words(), "assurance-auditor.md", None) is None, "Confirmation shows a hidden item"
        self.click_text("Cancel")
        assert installed.is_file() and other_installed.is_file(), "Cancelling removal changed an installed file"
        self.expect_components()
        self.passed("08-remove-cancel")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.confirm_single_removal("architect.md")
        self.expect("Operation complete", timeout=15)
        assert not installed.exists(), "Confirmed removal left the component installed"
        manifest = json.loads((self.home / ".hibi/install.json").read_text())
        assert manifest["components"] == ["agents/assurance-auditor.md"], manifest
        assert other_installed.is_file(), "Search removed a selected but hidden installed file"
        self.passed("09-remove-confirm-and-manifest")
        self.click_text("Back to components")
        self.clear_search()
        self.search("assurance-auditor")
        self.select_one_result("assurance-auditor.md")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.confirm_single_removal("assurance-auditor.md")
        self.expect("Operation complete", timeout=15)
        assert not other_installed.exists()
        self.click_text("Back to components")
        self.clear_search()
        self.click_text("Keyboard shortcuts")
        self.expect("Browse & review")
        before = self.capture()
        for _ in range(8): self.key("Down")
        self.expect_scroll(before, "Help")
        self.passed("10-help-keyboard-scroll")
        self.click_text("Back", (before.width // 2, 60, before.width, 125))
        self.expect_components()
        self.resize(760, 560)
        self.expect("Install")
        self.passed("11-minimum-window")
        self.resize(1120, 780)
        self.manage_individual_settings()
        self.back_to_home()
        self.click_text("Codex CLI")
        self.expect_components()
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
        self.key("a")
        self.expect("Where are your components")
        self.click_text("Git repository")
        self.expect("Connect a Git repository")
        self.key("Return")
        self.expect("Only HTTPS git URLs are allowed")
        self.passed("14-invalid-empty-git-url")
        self.click_text("Cancel")
        assert not (self.home / ".hibi/sources.yaml").exists()
        self.expect("Bundled library")
        self.key("a")
        self.expect("Where are your components")
        self.click_text("Local folder")
        self.expect("Choose a local folder")
        self.key("Return")
        self.expect("Local source path is empty")
        self.passed("15-empty-local-path")
        self.click_text("Cancel")
        self.expect("Bundled library")
        self.click_text("Back")
        self.expect("Your workspace")
        self.click_theme()
        self.expect_theme("dark")
        self.passed("16-dark-home-final")
        self.wait_for_appearance("dark")
        self.close_window()
        self.launch()
        self.expect_theme("dark")
        self.passed("17-dark-theme-restored")
        self.check_palette_restore()
        self.close_window()
        self.remove_source_with_busy_cache()
        (self.artifacts / "results.json").write_text(json.dumps({"passed": self.results, "isolated_home": str(self.home)}, indent=2))

    def check_palette_restore(self):
        width = self.window.get_geometry().width
        self.click(width - 120, 25)
        self.click_text("Nord", (width - 320, 32, width - 40, 700))
        self.wait_for_appearance({"palette": "Nord"})
        backdrop = self.capture().getpixel((0, 0))
        self.close_window()
        self.launch()
        self.find("Nord", (width - 320, 0, width - 40, 52))
        assert self.capture().getpixel((0, 0)) == backdrop
        self.click(width - 120, 25)
        self.click_text("System theme", (width - 320, 32, width - 40, 700))
        self.wait_for_appearance("system")
        self.passed("17a-palette-restored-and-system-reselected")

    def manage_individual_settings(self):
        settings_file = self.home / ".claude/settings.json"
        settings_file.parent.mkdir(parents=True, exist_ok=True)
        instructions = settings_file.with_name("CLAUDE.md")
        user_instructions = "# Custom workspace instructions\n"
        instructions.write_text(user_instructions)
        original = {
            "model": "custom",
            "env": {"MCP_TIMEOUT": 5000, "USER_VARIABLE": True},
            "hooks": {"PostToolUse": [{"hooks": [{"type": "command", "command": "./user-hook.sh"}]}]},
            "outputStyle": "my-style",
            "statusLine": {"type": "command", "command": "./status.sh"},
        }
        settings_file.write_text(json.dumps(original))
        self.back_to_home()
        self.click_text("Claude Code")
        self.expect_components()
        self.wheel(140, 420, -10)
        self.click_text("Config", (0, 150, 220, 665))
        self.find("Config", (235, 70, 550, 115))
        self.search("MCP_TIMEOUT")
        self.expect_result_count(1)
        self.expect("0 selected")
        self.select_one_result("env.MCP_TIMEOUT")
        self.inert("Remove", (220, 160, 1120, 780))
        self.click_text("Diff", (220, 300, 1120, 780))
        self.expect("Compare configuration", timeout=20)
        self.expect("5000")
        self.expect("10000")
        self.close_diff()
        self.clear_search()
        self.expect("1 selected")
        self.click_text("Install", (220, 300, 1120, 780))
        self.expect("Operation complete", timeout=15)
        expected = {**original, "env": {**original["env"], "MCP_TIMEOUT": 10000}}
        assert json.loads(settings_file.read_text()) == expected
        assert instructions.read_text() == user_instructions
        self.passed("11a-install-one-setting")

        self.click_text("Back to components")
        self.search("MCP_TIMEOUT")
        self.select_one_result("env.MCP_TIMEOUT")
        self.click_text("Remove", (220, 300, 1120, 780))
        self.confirm_single_removal("env.MCP_TIMEOUT")
        self.expect("Operation complete", timeout=15)
        del expected["env"]["MCP_TIMEOUT"]
        assert json.loads(settings_file.read_text()) == expected
        self.passed("11b-remove-one-setting")
        self.click_text("Back to components")
        self.settings_read_error(settings_file)
        settings_file.write_text(json.dumps(expected))

    def settings_read_error(self, settings_file):
        settings_file.write_text("{broken")
        self.back_to_home()
        self.click_text("Claude Code")
        self.expect_components()
        self.wheel(140, 420, -10)
        self.click_text("Config", (0, 150, 220, 665))
        self.find("Config", (235, 70, 550, 115))
        self.search("MCP_TIMEOUT")
        self.expect("blocked")
        self.select_one_result("env.MCP_TIMEOUT")
        self.inert("Install", (220, 160, 1120, 780))
        self.inert("Remove", (220, 160, 1120, 780))
        self.click_text("Diff", (220, 300, 1120, 780))
        self.expect("Invalid JSON")
        assert settings_file.read_text() == "{broken"
        self.passed("11c-settings-read-error")
        self.close_diff()

    def remove_source_with_busy_cache(self):
        url = "https://example.invalid/gui-smoke.git"
        key = hashlib.sha256(json.dumps([url, None], separators=(",", ":")).encode()).hexdigest()
        cache = self.home / ".hibi/cache/git" / key
        (cache / ".git").mkdir(parents=True)
        marker = cache / "AGENTS.md"
        marker.write_text("Shared cache fixture")
        config = self.home / ".hibi/sources.yaml"
        config.write_text(f"auto_update: false\nsources:\n  - type: git\n    url: {url}\n")

        self.launch()
        self.click_text("Manage sources")
        self.expect("Bundled library")
        self.key("Down")
        self.click_text("Remove source")
        self.expect("Disconnect this library?")
        with cache.with_suffix(".lock").open("wb") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.key("y")
            self.expect("Git cache is in use", timeout=15)
            assert "sources: []" in config.read_text()
            assert marker.read_text() == "Shared cache fixture"
            self.passed("18-remove-source-with-busy-cache")
            self.click_text("Back")
            self.expect("Your workspace")
        self.close_window()

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
