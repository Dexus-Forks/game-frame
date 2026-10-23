#!/usr/bin/env python3
"""Check the animated diagrams for the faults a reader sees: events that drift apart, elements
that show before their turn, animations the browser drops, and loops that jump.

Usage: python3 site/diagrams/animcheck.py [FILE.svg ...]     default: every diagram
       python3 site/diagrams/animcheck.py --json

It reads the SVG files that make.py writes, in docs/assets/diagrams/. For each <animate>,
<animateMotion> and <animateTransform> it checks:
- drift: every animation that repeats for ever has a duration that divides the diagram's
  longest one, so the events of one loop meet the same events in every later loop. An
  animation meant to run free (a ball that bounces at its own rate) says so with
  data-drift="free", and the decorative art and banners (art-*, banner-*) are left out;
- keyTimes: as many keyTimes as values, starting at 0, ending at 1 (except in discrete mode),
  never going back (a browser ignores an animation that breaks these, and the element stays
  still);
- early show: an element whose opacity animation starts after 0 is hidden before it starts
  (its own opacity, or the animation's first value, is 0), so it does not flash at load;
- seam: a repeating animation ends on the value it starts with, so the loop does not jump,
  unless the jump is to a hidden state (opacity 0 at the end or the start).
Output: one line per problem, with the file, the element and the fix. Exit status 1 when there
is a problem. It does not render anything: animrender.html, served from the repository's root,
checks layering, size and timing in a browser, frame by frame.
Standard library only.
"""
import json
import math
import os
import re
import sys
from fractions import Fraction
from xml.etree import ElementTree

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DIAGRAM_DIR = os.path.join(ROOT, 'docs', 'assets', 'diagrams')
NS = '{http://www.w3.org/2000/svg}'
ANIMS = {NS + 'animate', NS + 'animateMotion', NS + 'animateTransform'}


def seconds(text):
    """'6.0s', '1200ms', '2' -> Fraction seconds; None for anything else (indefinite, ids)."""
    m = re.fullmatch(r'\s*(-?[\d.]+)\s*(ms|s)?\s*', text or '')
    if not m:
        return None
    v = Fraction(m.group(1)).limit_denominator(1000)
    return v / 1000 if m.group(2) == 'ms' else v


def describe(el, parent):
    tag = parent.tag.replace(NS, '')
    ident = parent.get('id') or parent.get('x') or parent.get('cx') or parent.get('d', '')[:20]
    what = el.get('attributeName') or el.get('type') or 'motion'
    return f'<{tag} {ident}> {el.tag.replace(NS, "")} {what}'


def check(path):
    tree = ElementTree.parse(path)
    parents = {c: p for p in tree.iter() for c in p}
    anims = [e for e in tree.iter() if e.tag in ANIMS]
    problems = []
    say = lambda el, msg: problems.append({'file': os.path.basename(path),
                                           'element': describe(el, parents[el]), 'problem': msg})
    decorative = os.path.basename(path).startswith(('art-', 'banner-'))
    looping = [(e, seconds(e.get('dur'))) for e in anims
               if e.get('repeatCount') == 'indefinite' and seconds(e.get('dur'))]
    cycle = max((d for _, d in looping), default=None)
    for e, d in looping:
        if decorative or e.get('data-drift') == 'free':
            continue
        if cycle and (cycle / d).denominator != 1:
            say(e, f'dur {float(d)}s does not divide the {float(cycle)}s cycle: it drifts against the '
                   f'other animations each loop; use {float(cycle)}s (or a divisor) and move its keyTimes')
    for e in anims:
        values = e.get('values')
        kt = e.get('keyTimes')
        if values is not None and kt is not None and e.get('calcMode') != 'paced':
            vs = [v for v in values.split(';')]
            ts = [float(t) for t in kt.split(';')]
            if len(vs) != len(ts):
                say(e, f'{len(vs)} values but {len(ts)} keyTimes: the browser drops this animation')
            elif ts and abs(ts[0]) > 1e-9:
                say(e, 'keyTimes must start at 0: the browser drops this animation')
            elif ts and e.get('calcMode') != 'discrete' and abs(ts[-1] - 1) > 1e-9:
                say(e, 'keyTimes must end at 1 unless calcMode is discrete: the browser drops this animation')
            elif any(b < a for a, b in zip(ts, ts[1:])):
                say(e, 'keyTimes go backwards: the browser drops this animation')
        if e.get('attributeName') in ('opacity', 'fill-opacity') and values:
            vs = [v.strip() for v in values.split(';')]
            begin = seconds(e.get('begin', '0s'))
            target = parents[e]
            base = target.get(e.get('attributeName'), '1')
            if begin and begin > 0 and float(base) > 0 and float(vs[0]) == 0:
                say(e, f'starts at {float(begin)}s but the element is visible until then (its '
                       f'{e.get("attributeName")} is {base}): give it {e.get("attributeName")}="0"')
            if (e.get('repeatCount') == 'indefinite' and vs[0] != vs[-1]
                    and float(vs[0]) > 0 and float(vs[-1]) > 0):
                say(e, f'loops from {vs[-1]} back to {vs[0]}: the element jumps at the seam')
    return problems


def main(argv):
    if argv and argv[0] in ('-h', '--help'):
        print(__doc__.strip())
        return 0
    as_json = '--json' in argv
    files = [a for a in argv if a != '--json'] or sorted(
        os.path.join(DIAGRAM_DIR, f) for f in os.listdir(DIAGRAM_DIR) if f.endswith('.svg'))
    problems = [p for f in files for p in check(f)]
    if as_json:
        print(json.dumps(problems, indent=1))
    else:
        for p in problems:
            print(f'{p["file"]}: {p["element"]}: {p["problem"]}')
        print(f'{len(files)} diagrams, {len({p["file"] for p in problems})} with problems, {len(problems)} problems')
    return 1 if problems else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
