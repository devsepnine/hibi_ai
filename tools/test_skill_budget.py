"""Verify the dependency-free CLI against shipped skills and YAML boundary cases."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SKILLS = Path(__file__).resolve().parents[1] / "src" / "skills"
SCRIPT = SKILLS / "eval-harness" / "scripts" / "skill_budget.py"


class SkillBudgetTest(unittest.TestCase):
    def measure(self, directory, *options):
        return subprocess.run(
            [sys.executable, "-S", SCRIPT, directory, "--quiet", *options],
            capture_output=True, text=True,
        )

    def test_shipped_frontmatter_is_supported_without_pyyaml(self):
        # Budget tuning should not invalidate the frontmatter compatibility check.
        result = self.measure(SKILLS, "--context-window", "10000000")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"skills={len(list(SKILLS.glob('*/SKILL.md')))} ", result.stdout)

    def test_shipped_skill_listing_fits_default_budget(self):
        result = self.measure(SKILLS)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_scalar_and_mapping_boundaries(self):
        with tempfile.TemporaryDirectory() as directory:
            skill = Path(directory) / "s"
            skill.mkdir()
            for header, code in (
                ("description: alpha beta # comment", 0),
                ("description: 'alpha beta'", 0),
                ("description: +123", 2),
                ("description: true", 2),
                ("description: abc\nmetadata: stable\n  version: stable", 2),
                ("description: abc\nmetadata:\n  argument-hint: <file-or-pattern>", 0),
            ):
                with self.subTest(header=header):
                    (skill / "SKILL.md").write_text(f"---\nname: s\n{header}\n---\n")
                    result = self.measure(directory, "--context-window", "375")
                    self.assertEqual(result.returncode, code, result.stderr)


if __name__ == "__main__":
    unittest.main()
