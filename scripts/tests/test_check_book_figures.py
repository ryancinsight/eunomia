"""Tests for the deterministic mdBook figure-existence contract."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from scripts.check_book_figures import main, missing_figures
from scripts.generate_book_figures import FIGURES, generate


class CheckBookFiguresTests(unittest.TestCase):
    """Figure references pass only when their committed files exist."""

    def test_nested_reference_resolves(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            book = Path(directory)
            page = book / "examples" / "example.md"
            page.parent.mkdir()
            figure = book / "figures" / "ch01" / "fig01.svg"
            figure.parent.mkdir(parents=True)
            figure.write_text("<svg />\n", encoding="utf-8")
            page.write_text("![Figure](../figures/ch01/fig01.svg)\n", encoding="utf-8")

            self.assertEqual(missing_figures(book), [])
            self.assertEqual(main([str(book)]), 0)

    def test_missing_reference_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            book = Path(directory)
            page = book / "chapter.md"
            page.write_text("![Figure](figures/ch01/missing.svg)\n", encoding="utf-8")

            self.assertEqual(
                missing_figures(book),
                ["chapter.md:1: figures/ch01/missing.svg -> figures/ch01/missing.svg"],
            )
            self.assertEqual(main([str(book)]), 1)

    def test_fenced_image_is_not_a_book_reference(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            book = Path(directory)
            (book / "chapter.md").write_text(
                "```markdown\n![Example](figures/ch01/not-an-asset.svg)\n```\n",
                encoding="utf-8",
            )

            self.assertEqual(missing_figures(book), [])

    def test_generator_is_reproducible_for_all_manifest_figures(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            book = Path(directory)
            self.assertEqual(generate(book, check=False), 0)
            self.assertEqual(len(list((book / "figures").rglob("*.svg"))), 17)
            self.assertEqual(generate(book, check=True), 0)

            first = book / "figures" / FIGURES[0].path
            first.write_text(
                first.read_text(encoding="utf-8") + "<!-- drift -->\n",
                encoding="utf-8",
            )
            self.assertEqual(generate(book, check=True), 1)


if __name__ == "__main__":
    unittest.main()
