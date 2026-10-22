"""The drawing helpers of the guides' diagrams: the colours of the site's dark cards, and a small
SVG writer. site/diagrams/make.py and the diagram modules use them. Standard library only.
"""

BG, EDGE = '#0d1b1e', '#1f3d40'
TEXT, DIM = '#e2ece9', '#8fa9a5'
TEAL, AMBER, BLUE, RED, GREEN, PURPLE, GREY = '#2dd4bf', '#f59e0b', '#60a5fa', '#f87171', '#4ade80', '#c084fc', '#64748b'
FONT = "ui-sans-serif, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace"


def esc(s):
    return str(s).replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')


class Svg:
    def __init__(self, w, h, label):
        self.w, self.h, self.parts = w, h, []
        self.label = label

    def add(self, s):
        self.parts.append(s)
        return self

    def text(self, x, y, s, size=13, fill=TEXT, anchor='start', weight='400', mono=False, extra=''):
        return self.add(f'<text x="{x:.1f}" y="{y:.1f}" font-family="{MONO if mono else FONT}" font-size="{size}" '
                        f'font-weight="{weight}" fill="{fill}" text-anchor="{anchor}"{extra}>{esc(s)}</text>')

    def rect(self, x, y, w, h, fill, stroke='none', rx=4, extra=''):
        return self.add(f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{rx}" fill="{fill}" '
                        f'stroke="{stroke}"{extra}/>')

    def line(self, x1, y1, x2, y2, stroke=DIM, width=1.5, extra=''):
        return self.add(f'<path d="M{x1:.1f} {y1:.1f}L{x2:.1f} {y2:.1f}" stroke="{stroke}" stroke-width="{width}" '
                        f'fill="none"{extra}/>')

    def heading(self, s, sub=None):
        self.text(24, 34, s.upper(), 12, TEAL, weight='600', extra=' letter-spacing="0.08em"')
        if sub:
            self.text(24, 54, sub, 13, DIM)

    def render(self):
        head = (f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" '
                f'viewBox="0 0 {self.w} {self.h}" role="img" aria-label="{esc(self.label)}">\n'
                f'<title>{esc(self.label)}</title>\n'
                f'<rect x="0.5" y="0.5" width="{self.w - 1}" height="{self.h - 1}" rx="10" fill="{BG}" stroke="{EDGE}"/>\n'
                '<defs><marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" '
                f'orient="auto"><path d="M0 0L8 4L0 8z" fill="{DIM}"/></marker></defs>\n')
        return head + '\n'.join(self.parts) + '\n</svg>\n'
