"""Manifest and publication checks; fixtures never modify the real corpus."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location(
    "learning_site", Path(__file__).resolve().parents[1] / "bin/learning-site.py")
SITE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SITE)


class LearningSiteTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.pages = [
            dict(id="L01", path="tutorial/first.dice", part="Part A", status="pilot"),
            dict(id="L02", path="tutorial/next.dice", part="Part A", status="pilot",
                 prerequisites=["L01"], difficulty="beginner"),
            dict(id="R01", path="cookbook/recipe.dice", game="Generic", mechanic="Totals",
                 scope="component", status="pilot", prerequisites=["L02"]),
        ]
        for page in self.pages:
            source = self.root / "docs" / page["path"]
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text(f'# {page["id"]}\n\n```dice\noutput("die", d(6))\n```\n')
        self.aliases = []
        self.save()
        root_patch = patch.object(SITE, "ROOT", self.root)
        root_patch.start()
        self.addCleanup(root_patch.stop)

    def save(self):
        (self.root / "docs/learning-content.json").write_text(
            json.dumps(dict(version=1, pages=self.pages, aliases=self.aliases)))

    def test_inventory_and_prerequisites(self):
        self.assertEqual([p["id"] for p in SITE.inventory()], ["L01", "L02", "R01"])

    def test_rejects_forward_or_unknown_prerequisite(self):
        for prerequisite in ("L02", "missing", "L01"):
            self.pages[0]["prerequisites"] = [prerequisite]
            self.save()
            with self.assertRaisesRegex(ValueError, "earlier content IDs"):
                SITE.inventory()

    def test_rejects_unknown_status(self):
        self.pages[0]["status"] = "certified without review"
        self.save()
        with self.assertRaisesRegex(ValueError, "unknown review status"):
            SITE.inventory()

    def test_rejects_duplicate_identity(self):
        self.pages[1]["id"] = "L01"
        self.save()
        with self.assertRaisesRegex(ValueError, "duplicate content identity"):
            SITE.inventory()

    def test_rejects_unlisted_document(self):
        (self.root / "docs/tutorial/unlisted.dice").write_text("# Unlisted")
        with self.assertRaisesRegex(ValueError, "manifest/corpus mismatch"):
            SITE.inventory()

    def test_rejects_second_pilot_fence(self):
        source = self.root / "docs/tutorial/first.dice"
        source.write_text(source.read_text() + '\n```dice\noutput("extra", d(4))\n```\n')
        with self.assertRaisesRegex(ValueError, "exactly one executable fence"):
            SITE.inventory()

    def test_navigation_uses_manifest_order_and_cross_section_prerequisites(self):
        dist = self.root / "dist"
        for page in self.pages:
            target = dist / Path(page["path"]).with_suffix(".html")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("<main>Report</main>")
        SITE.publish(dist)
        first = (dist / "tutorial/first.html").read_text()
        self.assertIn('Next: <a href="next.html">L02</a>', first)
        second = (dist / "tutorial/next.html").read_text()
        self.assertIn('Previous: <a href="first.html">L01</a>', second)
        recipe = (dist / "cookbook/recipe.html").read_text()
        self.assertIn('href="../tutorial/next.html">L02</a>', recipe)
        cookbook = (dist / "cookbook/index.html").read_text()
        self.assertIn("By game", cookbook)
        self.assertIn("By mechanic", cookbook)
        self.assertIn("human and browser review", cookbook)

    def test_aliases_preserve_old_routes_with_explicit_migration(self):
        self.aliases = [dict(id="old-L01", path="tutorial/old-first.dice", target="L01")]
        self.save()
        dist = self.root / "dist"
        for page in self.pages:
            target = dist / Path(page["path"]).with_suffix(".html")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text("<main>Report</main>")
        SITE.publish(dist)
        old = (dist / "tutorial/old-first.html").read_text()
        self.assertIn("Lesson moved", old)
        self.assertIn('href="../tutorial/first.html"', old)
        self.assertEqual((dist / "tutorial/old-first.dice").read_text(),
                         (self.root / "docs/tutorial/first.dice").read_text())

    def test_alias_cannot_collide_or_chain(self):
        for alias in [dict(id="old", path="tutorial/first.dice", target="L01"),
                      dict(id="old", path="tutorial/old.dice", target="missing")]:
            self.aliases = [alias]
            self.save()
            with self.assertRaises(ValueError):
                SITE.inventory()


if __name__ == "__main__":
    unittest.main()
