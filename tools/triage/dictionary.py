#!/usr/bin/env python3
"""Check a project's data dictionary (labels/dictionary.tsv) and print it as a table.

Usage: dictionary.py [FILE] [--fields labels/fields.tsv] [--strict] [--markdown]

The data dictionary has one row per value the game's rules use, with ten tab-separated columns:
name, where, type, kind, unit, range, meaning, written by, read by, evidence. GameFrame's
docs/DATA-DICTIONARY.md describes each column, and docs/TAXONOMIES.md lists the kinds. Lines
that start with # are comments. FILE defaults to labels/dictionary.tsv.

Errors (the exit status is 1):
- a row without ten columns, or without a name;
- a `where` that is not one of the forms: Struct+0x0c, 0x14002f000, export NAME,
  file:NAME+OFFSET, save:BLOCK+OFFSET, msg:NAME+OFFSET, script:NAME;
- a `kind` that is not one of the kinds of values;
- an `evidence` that does not start with observed, measured, inferred or unknown;
- the same name and place in two rows.

Warnings (an error with --strict):
- an empty unit, range or meaning (write - when there is none);
- with --fields, a Struct+0x0c row whose structure and offset are not in fields.tsv, or whose
  type differs from the type there.

With --fields, the tool also counts the fields in fields.tsv that have no row yet.
--markdown prints the rows as a Markdown table, sorted by name, for a findings document.
Standard library only.
"""
import re
import sys
from pathlib import Path

COLUMNS = ['name', 'where', 'type', 'kind', 'unit', 'range', 'meaning', 'written by', 'read by', 'evidence']
KINDS = {'position', 'angle', 'time', 'count', 'ratio', 'identifier', 'flags', 'enumeration', 'random state',
         'reference', 'text'}
EVIDENCE = ('observed', 'measured', 'inferred', 'unknown')
NAME = re.compile(r'^[A-Za-z_][\w.\[\]]*$')
PLACES = [
    ('field', re.compile(r'^(?P<struct>[A-Za-z_][\w:]*)\+(?P<offset>0x[0-9A-Fa-f]+|\d+)$')),
    ('global', re.compile(r'^(0x)?[0-9A-Fa-f]{4,16}$|^[0-9A-Fa-f]{4}:[0-9A-Fa-f]{4}$')),
    ('export', re.compile(r'^export [A-Za-z_]\w*$')),
    ('file', re.compile(r'^file:[^+\s]+\+(0x[0-9A-Fa-f]+|\d+)$')),
    ('save', re.compile(r'^save:[^+\s]+\+(0x[0-9A-Fa-f]+|\d+)$')),
    ('message', re.compile(r'^msg:[^+\s]+\+(0x[0-9A-Fa-f]+|\d+)$')),
    ('script', re.compile(r'^script:[\w.\[\]]+$')),
]


def rows(path):
    """Yield (line number, columns) for each row of a tab-separated labels file."""
    for n, line in enumerate(Path(path).read_text(encoding='utf-8').splitlines(), 1):
        if line.strip() and not line.startswith('#'):
            yield n, line.split('\t')


def place(where):
    for kind, pattern in PLACES:
        m = pattern.match(where)
        if m:
            return kind, m
    return None, None


def offset(text):
    text = text.strip().lstrip('+')
    return int(text, 16) if text.lower().startswith('0x') else int(text)


def load_fields(path):
    """{(struct, offset): type} from labels/fields.tsv (struct, offset, name, type, evidence)."""
    fields = {}
    for n, cols in rows(path):
        if len(cols) >= 2:
            try:
                fields[(cols[0].strip(), offset(cols[1]))] = cols[3].strip() if len(cols) > 3 else ''
            except ValueError:
                pass
    return fields


def check(path, fields=None):
    """Return (entries, errors, warnings, covered): messages are 'line N: text'."""
    entries, errors, warnings, seen, covered = [], [], [], {}, set()
    for n, cols in rows(path):
        if len(cols) != len(COLUMNS):
            errors.append(f'line {n}: {len(cols)} columns; a row has {len(COLUMNS)}')
            continue
        row = dict(zip(COLUMNS, (c.strip() for c in cols)))
        entries.append(row)
        if not NAME.match(row['name']):
            errors.append(f'line {n}: the name "{row["name"]}" is empty or not a plain name')
        kind, m = place(row['where'])
        if kind is None:
            errors.append(f'line {n}: "{row["where"]}" is not a known form of place')
        if row['kind'] not in KINDS:
            errors.append(f'line {n}: "{row["kind"]}" is not a kind of value ({", ".join(sorted(KINDS))})')
        if not row['evidence'].split(':', 1)[0].strip() in EVIDENCE:
            errors.append(f'line {n}: the evidence must start with {", ".join(EVIDENCE)}')
        key = (row['name'], row['where'])
        if key in seen:
            errors.append(f'line {n}: {row["name"]} at {row["where"]} is already on line {seen[key]}')
        seen[key] = n
        for col in ('unit', 'range', 'meaning'):
            if not row[col]:
                warnings.append(f'line {n}: {row["name"]} has no {col} (write - when there is none)')
        if fields is not None and kind == 'field':
            spot = (m['struct'], offset(m['offset']))
            if spot not in fields:
                warnings.append(f'line {n}: {row["where"]} is not in fields.tsv')
            else:
                covered.add(spot)
                if fields[spot] and row['type'] and fields[spot] != row['type']:
                    warnings.append(f'line {n}: {row["where"]} is {row["type"]} here and {fields[spot]} in fields.tsv')
    return entries, errors, warnings, covered


def markdown(entries):
    cols = ['name', 'where', 'type', 'kind', 'unit', 'range', 'meaning', 'evidence']
    out = ['| ' + ' | '.join(cols) + ' |', '|' + '---|' * len(cols)]
    for row in sorted(entries, key=lambda r: (r['name'], r['where'])):
        cells = [f'`{row[c]}`' if c in ('name', 'where', 'type') else row[c].replace('|', '\\|') for c in cols]
        out.append('| ' + ' | '.join(cells) + ' |')
    return '\n'.join(out)


def main(argv):
    args, path, fields_path = list(argv), 'labels/dictionary.tsv', None
    strict = '--strict' in args
    as_table = '--markdown' in args
    args = [a for a in args if a not in ('--strict', '--markdown')]
    if '--fields' in args:
        i = args.index('--fields')
        fields_path = args[i + 1]
        del args[i:i + 2]
    if args and args[0] in ('-h', '--help'):
        print(__doc__)
        return 0
    if args:
        path = args[0]
    if not Path(path).exists():
        print(f'{path}: no such file', file=sys.stderr)
        return 2
    fields = load_fields(fields_path) if fields_path else None
    entries, errors, warnings, covered = check(path, fields)
    if as_table:
        print(markdown(entries))
    for msg in errors:
        print(f'{path}: error: {msg}', file=sys.stderr)
    for msg in warnings:
        print(f'{path}: warning: {msg}', file=sys.stderr)
    summary = f'{path}: {len(entries)} rows, {len(errors)} errors, {len(warnings)} warnings'
    if fields is not None:
        summary += f'; {len(covered)} of {len(fields)} fields in fields.tsv have a row'
    print(summary, file=sys.stderr if as_table else sys.stdout)
    return 1 if errors or (strict and warnings) else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
