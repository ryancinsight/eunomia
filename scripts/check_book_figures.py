#!/usr/bin/env python3
"""Verify that every local mdBook figure reference resolves to a file."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


IMAGE_LINK = re.compile(r"!\[[^\]]*\]\((?P<target>[^)\s]+)")
FENCE = re.compile(r"^\s*(?P<marker>`{3,}|~{3,})")
EXTERNAL_PREFIXES = ("http://", "https://", "mailto:", "data:")


def _figure_targets(page: Path, book_root: Path) -> list[tuple[int, str, Path]]:
    """Return local figure targets in one Markdown page."""
    targets: list[tuple[int, str, Path]] = []
    fence_marker: str | None = None
    for line_number, line in enumerate(page.read_text(encoding="utf-8").splitlines(), 1):
        fence = FENCE.match(line)
        if fence is not None:
            marker = fence.group("marker")
            if fence_marker is None:
                fence_marker = marker[0]
            elif marker.startswith(fence_marker):
                fence_marker = None
            continue
        if fence_marker is not None:
            continue

        for match in IMAGE_LINK.finditer(line):
            target = match.group("target").split("#", 1)[0].split("?", 1)[0]
            if not target or target.startswith(("#", "/")):
                continue
            if target.lower().startswith(EXTERNAL_PREFIXES):
                continue

            resolved = (page.parent / target).resolve()
            try:
                relative = resolved.relative_to(book_root)
            except ValueError:
                continue
            if relative.parts and relative.parts[0] == "figures":
                targets.append((line_number, target, resolved))
    return targets


def missing_figures(book_dir: Path) -> list[str]:
    """Return deterministic diagnostics for missing local book figures."""
    book_root = book_dir.resolve()
    if not book_root.is_dir():
        raise FileNotFoundError(f"book directory not found: {book_dir}")

    missing: list[str] = []
    for page in sorted(book_root.rglob("*.md")):
        page_name = page.relative_to(book_root).as_posix()
        for line_number, target, resolved in _figure_targets(page, book_root):
            if not resolved.is_file():
                relative_target = resolved.relative_to(book_root).as_posix()
                missing.append(
                    f"{page_name}:{line_number}: {target} -> {relative_target}"
                )
    return missing


def main(argv: list[str] | None = None) -> int:
    """Check a book directory and return a CI-friendly status code."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("book_dir", type=Path, help="Path to the mdBook source")
    args = parser.parse_args(argv)

    try:
        missing = missing_figures(args.book_dir)
    except (FileNotFoundError, OSError) as error:
        print(f"check-book-figures: {error}", file=sys.stderr)
        return 2

    if missing:
        print("check-book-figures: missing local figure files:", file=sys.stderr)
        for item in missing:
            print(f"  {item}", file=sys.stderr)
        return 1

    print("check-book-figures: all local figure references resolve")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
