#!/usr/bin/env python3
"""Render terminal text as a small SVG picture of a terminal window, for the docs.

Usage:
    term_svg.py TITLE OUT.svg < text

Input:  plain text on stdin (no ANSI colour codes). A line that starts with "$ " is a
        command and is drawn in the brand colour. In test output, "ok" is drawn green,
        "FAIL" red and "skip" grey. A line that is only "..." marks lines left out.
Output: one SVG file with a dark window, a title bar with TITLE, and the text in a
        monospace font. The picture has its own dark background, so it reads the same on
        light and dark pages, on the site and on GitHub.

It does not wrap long lines (keep them under about 100 characters), read ANSI colours or
embed fonts: the viewer's monospace font is used, so widths are estimated.
"""
import sys
from xml.sax.saxutils import escape

FONT = "ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace"
SIZE = 13          # font size, px
LINE = 19          # line height, px
CHAR = 7.9         # estimated advance of one character at SIZE, px
PAD = 18           # padding inside the window, px
BAR = 30           # title bar height, px

BG, BAR_BG, BORDER = "#0d1b1e", "#132a2d", "#1f3d40"
TEXT, DIM, PROMPT = "#d5e4e2", "#7c9895", "#2dd4bf"
GREEN, RED, AMBER = "#4ade80", "#f87171", "#f59e0b"


def tspans(line: str) -> str:
    """The SVG text content for one line, with its colours."""
    if line.startswith("$ "):
        return f'<tspan fill="{DIM}">$ </tspan><tspan fill="{PROMPT}">{escape(line[2:])}</tspan>'
    if line.strip() == "...":
        return f'<tspan fill="{DIM}">  …</tspan>'
    for word, colour in (("ok", GREEN), ("FAIL", RED), ("skip", DIM)):
        lead = line[: len(line) - len(line.lstrip())]
        if line.lstrip().startswith(word + " "):
            rest = line[len(lead) + len(word):]
            return f'{escape(lead)}<tspan fill="{colour}">{word}</tspan>{escape(rest)}'
    if line and not line.startswith(" ") and not line.startswith("//") and line[0].isupper() \
            and len(line) < 40 and not line.endswith("."):
        return f'<tspan fill="{AMBER}">{escape(line)}</tspan>'  # a section heading
    return escape(line)


def render(title: str, text: str) -> str:
    lines = text.rstrip("\n").expandtabs(4).split("\n")
    width = int(PAD * 2 + max(60, max(len(l) for l in lines)) * CHAR)
    height = BAR + PAD + len(lines) * LINE + PAD - 4
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" role="img" aria-label="{escape(title)}">',
        f'<rect x="0.5" y="0.5" width="{width - 1}" height="{height - 1}" rx="9" fill="{BG}" stroke="{BORDER}"/>',
        f'<path d="M1 {BAR}V10a9 9 0 0 1 9-9h{width - 20}a9 9 0 0 1 9 9v{BAR - 10}z" fill="{BAR_BG}"/>',
    ]
    for i, colour in enumerate(("#f87171", "#fbbf24", "#4ade80")):
        out.append(f'<circle cx="{18 + i * 18}" cy="{BAR / 2}" r="5" fill="{colour}" opacity="0.8"/>')
    out.append(f'<text x="{width / 2}" y="{BAR / 2 + 4}" text-anchor="middle" fill="{DIM}" '
               f'font-family="{FONT}" font-size="12">{escape(title)}</text>')
    out.append(f'<text font-family="{FONT}" font-size="{SIZE}" fill="{TEXT}" xml:space="preserve">')
    for i, line in enumerate(lines):
        y = BAR + PAD + 10 + i * LINE
        out.append(f'<tspan x="{PAD}" y="{y}">{tspans(line)}</tspan>')
    out.append("</text></svg>")
    return "\n".join(out) + "\n"


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__.strip())
        return 2
    title, path = sys.argv[1], sys.argv[2]
    with open(path, "w", encoding="utf-8") as f:
        f.write(render(title, sys.stdin.read()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
