#!/usr/bin/env python3
"""Check a port against a recording of the original that was read from outside, sample by sample.

Usage: replay.py RECORDING PORT_TRACE --clock NAME [--fields A,B,...] [--block A,B ...]
                 [--require NAME=RATE ...] [--show N] [--json]

  RECORDING   JSON Lines, one sample a line: {"t": ..., "n": ..., NAME: value or [values]}, as
              tools/live/memrec.py record writes
  PORT_TRACE  JSON Lines from the port: one line for every state it passes through (after each
              frame or step), with the same NAMEs, the clock among them
  --clock     the value that orders both: the game's own clock or frame counter, read by the
              recorder and kept by the port
  --fields    the values to compare (default: every name in the recording the trace also has,
              except the clock, t and n)
  --block     values the recorder reads together, compared as one: a sample passes only when
              one port state holds all of them (repeat for several blocks)
  --require   the share of samples a value must pass, 0 to 1 (default 1: every sample). Give a
              lower rate only for a value the port cannot reproduce from the recording's input
  --show      how many failures to print for each value (default 3)

Why: a recorder outside the game reads one value after another while the game runs, and sees
only some frames. One sample can hold the clock of one frame and positions from the next
(docs/knowhow/replay-sample-straddles-frames.md). Comparing each sample with the port's state
at the sample's clock then fails where the port is right. Here a value passes when any state
the port passed through in the sample's window holds it: from the sample's clock up to the
next sample's clock, plus the frame after that. A value that is wrong in the port fails in
every window it reaches.

Output: for each value (or block), the samples that passed, and the first failures with the
recorded value and what the port held in the window. Exit status 1 when a value falls below
its rate, or when a sample's window holds no port state (the trace does not cover the run).

It does not drive the port: the port replays the recording's input itself and writes its
trace. Standard library only.
"""
import bisect
import json
import sys


def load(path):
    with open(path, encoding='utf-8') as f:
        return [json.loads(line) for line in f if line.strip()]


def parse(argv):
    opts = {'clock': None, 'fields': None, 'blocks': [], 'require': {}, 'show': 3, 'json': False}
    pos, i = [], 0
    while i < len(argv):
        a = argv[i]
        if a == '--json':
            opts['json'] = True
        elif a in ('--clock', '--fields', '--block', '--require', '--show') and i + 1 < len(argv):
            v = argv[i + 1]
            i += 1
            if a == '--clock':
                opts['clock'] = v
            elif a == '--fields':
                opts['fields'] = [x for x in v.split(',') if x]
            elif a == '--block':
                opts['blocks'].append([x for x in v.split(',') if x])
            elif a == '--require':
                k, r = v.split('=')
                opts['require'][k] = float(r)
            else:
                opts['show'] = int(v)
        else:
            pos.append(a)
        i += 1
    return pos, opts


def check(recording, trace, clock, fields=None, blocks=(), require=None, show=3):
    """Compare; return {unit: {passed, samples, rate, required, failures: [...]}, '_uncovered': n}."""
    require = require or {}
    trace = sorted(trace, key=lambda s: s[clock])
    clocks = [s[clock] for s in trace]
    skip = {clock, 't', 'n'}
    if fields is None:
        fields = [k for k in recording[0] if k not in skip and k in trace[0]] if recording and trace else []
    in_block = {f for b in blocks for f in b}
    units = [tuple(b) for b in blocks] + [(f,) for f in fields if f not in in_block]
    report = {' '.join(u) if len(u) > 1 else u[0]: {'passed': 0, 'samples': 0, 'failures': []} for u in units}
    uncovered = 0
    for i, sample in enumerate(recording):
        lo = sample[clock]
        # The window ends at the next sample's clock; the last sample takes the spacing before it.
        if i + 1 < len(recording):
            hi = recording[i + 1][clock]
        else:
            hi = lo + (lo - recording[i - 1][clock] if i else 0)
        a = bisect.bisect_left(clocks, lo)
        b = bisect.bisect_right(clocks, hi)
        window = trace[a:min(b + 1, len(trace))]      # up to the next sample's clock, and one frame more
        if not window:
            uncovered += 1
            continue
        for u in units:
            key = ' '.join(u) if len(u) > 1 else u[0]
            if any(f not in sample for f in u):
                continue
            r = report[key]
            r['samples'] += 1
            if any(all(s.get(f) == sample[f] for f in u) for s in window):
                r['passed'] += 1
            elif len(r['failures']) < show:
                r['failures'].append({'n': sample.get('n', i), clock: lo,
                                      'recorded': {f: sample[f] for f in u},
                                      'port': [{f: s.get(f) for f in (clock,) + u} for s in window[:4]]})
    for key, r in report.items():
        r['rate'] = r['passed'] / r['samples'] if r['samples'] else 1.0
        r['required'] = require.get(key, 1.0)
    report['_uncovered'] = uncovered
    return report


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__.strip())
        return 0 if argv else 2
    pos, o = parse(argv)
    if len(pos) != 2 or not o['clock']:
        print(__doc__.strip())
        return 2
    rep = check(load(pos[0]), load(pos[1]), o['clock'], o['fields'], o['blocks'], o['require'], o['show'])
    uncovered = rep.pop('_uncovered')
    bad = [k for k, r in rep.items() if r['rate'] < r['required']]
    if o['json']:
        print(json.dumps({'values': rep, 'uncovered': uncovered, 'failed': bad}, indent=1))
    else:
        for k, r in rep.items():
            mark = 'FAIL' if k in bad else 'ok  '
            print(f'{mark} {k}: {r["passed"]} of {r["samples"]} samples ({100 * r["rate"]:.1f} %, needs {100 * r["required"]:.0f} %)')
            if k in bad:
                for f in r['failures']:
                    print(f'       sample {f["n"]} at {o["clock"]} {f[o["clock"]]}: recorded {f["recorded"]}, port {f["port"]}')
        if uncovered:
            print(f'FAIL {uncovered} samples have no port state in their window: the trace does not cover the run')
    return 1 if bad or uncovered else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
