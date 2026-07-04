#!/usr/bin/env python3
"""Render captured ANSI terminal output to a self-contained SVG "screenshot".

    purefetch | ... captured to out.ansi   (colored: run under a pty)
    python3 scripts/ansisvg.py out.ansi assets/purefetch.svg [title]

Handles reset(0), bold(1), truecolor fg (38;2;r;g;b), standard/bright fg
(30-37, 90-97) and bg (40-47, 100-107). Draws a rounded terminal window with a
title bar. No external dependencies.
"""
import sys
import re
from html import escape

# Dracula-ish 16-color ANSI palette (index 0..15).
PALETTE = [
    "#21222c", "#ff5555", "#50fa7b", "#f1fa8c",
    "#bd93f9", "#ff79c6", "#8be9fd", "#f8f8f2",
    "#6272a4", "#ff6e6e", "#69ff94", "#ffffa5",
    "#d6acff", "#ff92df", "#a4ffff", "#ffffff",
]
BG = "#282a36"
DEFAULT_FG = "#f8f8f2"

CW = 9.0
LH = 20.0
FS = 15
PAD = 22.0
TOPBAR = 34.0
ASCENT = 15.0

SGR_RE = re.compile(r"\x1b\[([0-9;]*)m")


class State:
    def __init__(self):
        self.reset()

    def reset(self):
        self.fg = None
        self.bg = None
        self.bold = False


def apply_sgr(state, params):
    codes = [int(p) for p in params.split(";") if p != ""] or [0]
    i = 0
    while i < len(codes):
        c = codes[i]
        if c == 0:
            state.reset()
        elif c == 1:
            state.bold = True
        elif c == 22:
            state.bold = False
        elif c == 38 and i + 2 < len(codes) and codes[i + 1] == 2:
            state.fg = "#%02x%02x%02x" % (codes[i + 2], codes[i + 3], codes[i + 4])
            i += 4
        elif c == 39:
            state.fg = None
        elif 30 <= c <= 37:
            state.fg = PALETTE[c - 30]
        elif 90 <= c <= 97:
            state.fg = PALETTE[c - 90 + 8]
        elif c == 48 and i + 2 < len(codes) and codes[i + 1] == 2:
            state.bg = "#%02x%02x%02x" % (codes[i + 2], codes[i + 3], codes[i + 4])
            i += 4
        elif c == 49:
            state.bg = None
        elif 40 <= c <= 47:
            state.bg = PALETTE[c - 40]
        elif 100 <= c <= 107:
            state.bg = PALETTE[c - 100 + 8]
        i += 1


def parse_line(line, state):
    runs = []
    col = 0
    pos = 0
    for m in SGR_RE.finditer(line):
        text = line[pos:m.start()]
        if text:
            runs.append((col, text, state.fg or DEFAULT_FG, state.bg, state.bold))
            col += len(text)
        apply_sgr(state, m.group(1))
        pos = m.end()
    tail = line[pos:]
    if tail:
        runs.append((col, tail, state.fg or DEFAULT_FG, state.bg, state.bold))
        col += len(tail)
    return runs, col


def main():
    data = open(sys.argv[1]).read() if len(sys.argv) > 1 else sys.stdin.read()
    out = sys.argv[2] if len(sys.argv) > 2 else None
    title = sys.argv[3] if len(sys.argv) > 3 else "purefetch"
    lines = data.rstrip("\n").split("\n")

    state = State()
    parsed = []
    maxcol = 0
    for ln in lines:
        runs, cols = parse_line(ln, state)
        parsed.append(runs)
        maxcol = max(maxcol, cols)

    width = maxcol * CW + 2 * PAD
    height = TOPBAR + len(lines) * LH + 2 * PAD - 6

    svg = []
    svg.append(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0f}" '
        f'height="{height:.0f}" viewBox="0 0 {width:.0f} {height:.0f}" '
        f'font-family="ui-monospace, \'DejaVu Sans Mono\', \'JetBrains Mono\', monospace" '
        f'font-size="{FS}">'
    )
    svg.append(f'<rect width="{width:.0f}" height="{height:.0f}" rx="10" fill="{BG}"/>')
    for i, c in enumerate(("#ff5f56", "#ffbd2e", "#27c93f")):
        svg.append(f'<circle cx="{20 + i*20:.0f}" cy="17" r="6" fill="{c}"/>')
    svg.append(
        f'<text x="{width/2:.0f}" y="21" fill="#6272a4" text-anchor="middle" '
        f'font-size="12">{escape(title)}</text>'
    )

    y0 = TOPBAR + PAD
    for row, runs in enumerate(parsed):
        y = y0 + row * LH
        for col, text, fg, bg, bold in runs:
            if bg:
                x = PAD + col * CW
                svg.append(
                    f'<rect x="{x:.1f}" y="{y:.1f}" width="{len(text)*CW:.1f}" '
                    f'height="{LH:.1f}" fill="{bg}"/>'
                )
        for col, text, fg, bg, bold in runs:
            if text.strip() == "":
                continue
            x = PAD + col * CW
            weight = ' font-weight="bold"' if bold else ""
            svg.append(
                f'<text x="{x:.1f}" y="{y + ASCENT:.1f}" fill="{fg}" '
                f'xml:space="preserve"{weight}>{escape(text)}</text>'
            )
    svg.append("</svg>")
    result = "\n".join(svg) + "\n"

    if out:
        with open(out, "w") as f:
            f.write(result)
        print(f"wrote {out}: {width:.0f}x{height:.0f}, {len(lines)} lines")
    else:
        sys.stdout.write(result)


if __name__ == "__main__":
    main()
