#!/usr/bin/env python3
"""Generate the deterministic conceptual figures used by the Eunomia book."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from html import escape
from pathlib import Path


@dataclass(frozen=True)
class Figure:
    """Describe one figure's output path, title, and domain labels."""

    path: str
    title: str
    labels: tuple[str, ...]


FIGURES = (
    Figure("ch01/fig01_1_floating_point_scalar_types.svg", "1. Floating-Point Scalar Types", ("f16", "bf16", "f32", "f64")),
    Figure("ch01/fig02_example_choosing_a_precision.svg", "Example: Choosing a Precision", ("input", "precision", "error bound")),
    Figure("ch02/fig01_2_integer_scalar_types.svg", "2. Integer Scalar Types", ("signed", "unsigned", "width", "range")),
    Figure("ch03/fig01_3_complex_numbers.svg", "3. Complex Numbers", ("real", "imaginary", "Complex<T>")),
    Figure("ch03/fig02_example_complex_arithmetic_in_a_solver.svg", "Example: Complex Arithmetic in a Solver", ("field", "operator", "solution")),
    Figure("ch04/fig01_4_numericelement_the_monomorphization_extension_point.svg", "4. NumericElement: The Monomorphization Extension Point", ("Scalar", "NumericElement<T>", "kernel")),
    Figure("ch05/fig01_5_floatelement_the_transcendental_surface.svg", "5. FloatElement: The Transcendental Surface", ("exp / log", "sqrt", "trigonometry")),
    Figure("ch06/fig01_6_scalar_fields_realfield_and_complexfield.svg", "6. Scalar Fields: RealField and ComplexField", ("RealField", "ComplexField", "field laws")),
    Figure("ch07/fig01_7_unitscalar_the_physical_unit_seam.svg", "7. UnitScalar: The Physical-Unit Seam", ("value", "unit", "quantity")),
    Figure("ch08/fig01_8_the_cast_lattice_castfrom_and_castto.svg", "8. The Cast Lattice: CastFrom and CastTo", ("From", "TryFrom", "CastFrom", "CastTo")),
    Figure("ch09/fig01_9_the_native_conversion_kernel.svg", "9. The Native Conversion Kernel", ("validate", "convert", "preserve")),
    Figure("ch09/fig02_example_rounding_behaviour.svg", "Example: Rounding Behaviour", ("value", "round", "error bound")),
    Figure("ch10/fig01_10_byte_layout_pod_and_zeroable.svg", "10. Byte Layout: Pod and Zeroable", ("layout", "Pod", "Zeroable")),
    Figure("ch11/fig01_11_packed_sub_byte_formats.svg", "11. Packed Sub-byte Formats", ("bits", "encode", "decode")),
    Figure("ch12/fig01_12_relative_equality.svg", "12. Relative Equality", ("absolute", "relative", "epsilon")),
    Figure("ch13/fig01_13_element_operations.svg", "13. Element Operations", ("add", "multiply", "fused multiply-add")),
    Figure("ch14/fig01_14_position_in_the_atlas_stack.svg", "14. Position in the Atlas Stack", ("eunomia", "leto", "apollo")),
)


def _text_lines(label: str, limit: int = 19) -> tuple[str, ...]:
    """Wrap one label into deterministic SVG text lines."""
    words = label.split()
    lines: list[str] = []
    current = ""
    for word in words:
        candidate = f"{current} {word}".strip()
        if current and len(candidate) > limit:
            lines.append(current)
            current = word
        else:
            current = candidate
    if current:
        lines.append(current)
    return tuple(lines or ("",))


def render(figure: Figure) -> str:
    """Render one figure from its domain manifest."""
    width = 1000
    height = 420
    center_x = width / 2
    box_width = min(220, max(150, width // max(4, len(figure.labels) + 1)))
    gap = (width - len(figure.labels) * box_width) / (len(figure.labels) + 1)
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-label="{escape(figure.title)}">',
        '<defs><marker id="arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8 Z" fill="#64748b"/></marker></defs>',
        '<rect width="1000" height="420" fill="#f8fafc"/>',
        f'<text x="{center_x:g}" y="42" text-anchor="middle" font-family="Arial, sans-serif" font-size="22" fill="#111827">{escape(figure.title)}</text>',
    ]
    y = 175
    box_height = 86
    centers: list[float] = []
    for index, label in enumerate(figure.labels):
        x = gap + index * (box_width + gap)
        centers.append(x + box_width / 2)
        if index:
            parts.append(
                f'<line x1="{centers[index - 1]:g}" y1="{y + box_height / 2:g}" x2="{x - 8:g}" y2="{y + box_height / 2:g}" stroke="#64748b" stroke-width="2" marker-end="url(#arrow)"/>'
            )
        parts.append(f'<rect x="{x:g}" y="{y}" width="{box_width:g}" height="{box_height}" rx="8" fill="#ffffff" stroke="#2563eb" stroke-width="2"/>')
        lines = _text_lines(label)
        first_y = y + box_height / 2 - (len(lines) - 1) * 9
        for line_index, text in enumerate(lines):
            parts.append(
                f'<text x="{centers[index]:g}" y="{first_y + line_index * 18:g}" text-anchor="middle" font-family="Arial, sans-serif" font-size="15" fill="#1f2937">{escape(text)}</text>'
            )
    parts.extend(
        [
            f'<text x="{center_x:g}" y="330" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" fill="#475569">{escape("Generated from Eunomia's domain figure manifest")}</text>',
            "</svg>",
        ]
    )
    return "\n".join(parts) + "\n"


def generate(book_dir: Path, check: bool) -> int:
    """Generate figures, or check that committed output is reproducible."""
    failures: list[str] = []
    for figure in FIGURES:
        target = book_dir / "figures" / figure.path
        rendered = render(figure)
        if check:
            if not target.is_file() or target.read_text(encoding="utf-8") != rendered:
                failures.append(figure.path)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(rendered, encoding="utf-8", newline="\n")
    if failures:
        print("generate-book-figures: stale or missing outputs:", *failures, sep="\n")
        return 1
    print(f"generate-book-figures: {'checked' if check else 'generated'} {len(FIGURES)} figures")
    return 0


def main() -> int:
    """Parse the book path and execute the generator."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("book_dir", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if not args.book_dir.is_dir():
        parser.error(f"book directory not found: {args.book_dir}")
    return generate(args.book_dir, args.check)


if __name__ == "__main__":
    raise SystemExit(main())
