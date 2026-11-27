"""Run the document checks shipped with the architecture and feature map skills."""

from pathlib import Path
import re
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


SKILLS = Path(__file__).resolve().parents[1] / "src" / "skills"


class TriggerChecks(unittest.TestCase):
    def test_negative_measurement_requires_clean_exit(self):
        run_query = runpy.run_path(str(SKILLS / "eval-harness/scripts/trigger_eval.py"))["run_query"]
        success = b'{"type":"result","subtype":"success"}\n'
        for status, expected in [(0, "PASS"), (7, "INCONCLUSIVE")]:
            process = subprocess.CompletedProcess([], status, success, b"")
            with self.subTest(status=status), patch("subprocess.run", return_value=process):
                self.assertEqual(run_query("sample", False, "how", Path("."), 10, 6).verdict, expected)
        with patch("subprocess.run", side_effect=subprocess.TimeoutExpired("claude", 10, output=success)):
            self.assertEqual(run_query("sample", False, "how", Path("."), 10, 6).verdict, "INCONCLUSIVE")


class DocumentChecks(unittest.TestCase):
    def setUp(self):
        work = tempfile.TemporaryDirectory(prefix="skill review ")
        self.addCleanup(work.cleanup)
        self.repo = Path(work.name)
        (self.repo / "docs/features").mkdir(parents=True)
        (self.repo / "source files").mkdir()

    def run_check(self, script, *args):
        return subprocess.run([sys.executable, SKILLS / script, *args], cwd=self.repo,
                              capture_output=True, text=True, encoding="utf-8")

    @unittest.skipUnless(shutil.which("bash"), "the documented grader uses Bash")
    def test_documented_grader_propagates_failure(self):
        src = self.repo / "src"
        src.mkdir()
        for guide in ("SKILL.md", "SKILL-ko.md"):
            text = (SKILLS / "eval-harness" / guide).read_text(encoding="utf-8")
            grader = next(block for block in re.findall(r"^```bash\n(.*?)^```", text, re.M | re.S)
                          if "src/auth.ts" in block)
            for content, expected in [("", 1), ("export function handleAuth", 0)]:
                with self.subTest(guide=guide, expected=expected):
                    (src / "auth.ts").write_text(content, encoding="utf-8")
                    result = subprocess.run(["bash", "-c", "npm() { return 0; }\n" + grader],
                                            cwd=self.repo, capture_output=True)
                    self.assertEqual(result.returncode, expected)

    def test_architecture_fields_belong_to_each_rule(self):
        document = self.repo / "docs/ARCHITECTURE.md"
        fields = "".join(f"- **{key}**: example\n" for key in
                         ("Level", "Rule", "Why", "Evidence", "Exceptions", "Enforced by"))
        checker = "architecture-rules/scripts/check_rules.py"
        first = "### `arch-first`\n" + fields
        second = "### `arch-second`\n" + fields
        document.write_text(first + second, encoding="utf-8")
        self.assertEqual(self.run_check(checker).returncode, 0)
        document.write_text(first + "- **Why**: duplicate\n" + second.replace("- **Why**: example\n", ""),
                            encoding="utf-8")
        result = self.run_check(checker)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("FIELD arch-first: Why occurs 2 times", result.stdout)
        self.assertIn("FIELD arch-second: Why occurs 0 times", result.stdout)
        document.write_text("## Rules\n", encoding="utf-8")
        self.assertNotEqual(self.run_check(checker).returncode, 0)

    def test_feature_text_must_appear_outside_the_map(self):
        checker = "feature-map/scripts/check_map.py"
        index = self.repo / "docs/FEATURES.md"
        index.write_text("# Index\n", encoding="utf-8")
        (self.repo / "docs/features/split map.md").write_text('- **UI 문구**: "열기"\n', encoding="utf-8")
        result = self.run_check(checker, ".")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("MISSING-STRING 열기", result.stdout)
        (self.repo / "source files/ui.txt").write_text("열기", encoding="utf-8")
        self.assertEqual(self.run_check(checker, "source files").returncode, 0)
        self.assertNotEqual(self.run_check(checker, "absent directory").returncode, 0)
        index.unlink()
        self.assertNotEqual(self.run_check(checker, "source files").returncode, 0)

    def test_feature_symbol_path_with_spaces(self):
        doc = self.repo / "docs/FEATURES.md"
        doc.write_text("- Source: `source files/screen.py` → `render`\n", encoding="utf-8")
        source = self.repo / "source files/screen.py"
        source.write_text("def draw(): pass\n", encoding="utf-8")
        checker = "feature-map/scripts/check_map.py"
        result = self.run_check(checker, source)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("MISSING-SYMBOL source files/screen.py → render", result.stdout)
        source.write_text("def render(): pass\n", encoding="utf-8")
        self.assertEqual(self.run_check(checker, source).returncode, 0)


if __name__ == "__main__":
    unittest.main()
