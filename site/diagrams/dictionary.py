"""Diagrams for the taxonomies (docs/TAXONOMIES.md) and the data dictionary
(docs/DATA-DICTIONARY.md). site/diagrams/make.py writes them.
"""
from engines import arrow, box, lines
from svg import AMBER, BLUE, DIM, GREEN, GREY, PURPLE, RED, TEAL, TEXT, Svg


def taxonomies_overview():
    s = Svg(900, 360, 'The five taxonomies GameFrame uses: kinds of code, kinds of data, kinds of values, kinds of '
                      'evidence and kinds of difference between the port and the original')
    s.heading('Five sets of shared names',
              'Every function, file, value, claim and difference gets one name from its set.')
    sets = [('Code', TEAL, ['game rules', 'interface', 'engine', 'middleware', 'runtime library', 'compiler helpers',
                            'protection', 'debug and dead code']),
            ('Data', BLUE, ['assets', 'rule tables', 'scripts', 'levels and maps', 'saves', 'settings', 'derived data',
                            'messages']),
            ('Values', PURPLE, ['position', 'angle', 'time', 'count', 'ratio', 'identifier', 'flags', 'enumeration',
                                'random state', 'reference', 'text']),
            ('Evidence', AMBER, ['observed', 'measured', 'inferred', 'unknown']),
            ('Differences', RED, ['not ported', 'deviation', 'not yet verified', 'known difference',
                                  'original bug kept'])]
    for k, (title, colour, items) in enumerate(sets):
        x = 24 + k * 172
        s.rect(x, 84, 162, 256, colour, rx=6, extra=' fill-opacity="0.12"')
        s.text(x + 12, 108, title, 13, colour, weight='600')
        for j, item in enumerate(items):
            s.rect(x + 10, 120 + j * 21, 142, 18, colour, rx=3, extra=' fill-opacity="0.22"')
            s.text(x + 18, 133 + j * 21, item, 10.5, TEXT)
    return s.render()


def dictionary_entry():
    s = Svg(900, 400, 'One row of the data dictionary, for the growth stage of a tree in the sample game, with the '
                      'source that fills each column: the field labels, the arithmetic, the checks, the list of '
                      'functions that touch the field, and the running game')
    s.heading('One entry of the data dictionary',
              'Each column comes from a different source. The evidence label rises from inferred to measured.')
    rows = [('name', 'tree.stage'), ('where', 'Entity+0x0c'), ('type', 'int32'), ('kind', 'enumeration'),
            ('unit', '-'), ('range', '0 to 3; stops at 3'), ('meaning', 'The growth stage of a tree.'),
            ('written by', 'tree_update'), ('read by', 'tree_update, tree_value'),
            ('evidence', 'measured: rises with each stage')]
    x0, y0 = 300, 84
    for k, (col, val) in enumerate(rows):
        y = y0 + k * 28
        s.rect(x0, y, 310, 24, GREY, rx=3, extra=' fill-opacity="0.14"')
        s.text(x0 + 10, y + 16, col, 11, AMBER, mono=True)
        s.text(x0 + 96, y + 16, val, 11, TEXT, mono=True)
    sources = [('labels/fields.tsv', 'the field\'s name, place, type', 0, 2, TEAL),
               ('the arithmetic', 'the kind and the unit', 3, 4, BLUE),
               ('clamps and checks', 'the range at each edge', 5, 5, PURPLE),
               ('decomp_query.py layout', 'the functions that touch it', 7, 8, GREEN),
               ('the running game', 'confirms the meaning', 9, 9, AMBER)]
    for k, (title, sub, a, b, colour) in enumerate(sources):
        y = 88 + k * 56
        box(s, 24, y, 220, 44, colour, title, sub, op=0.22, size=11.5)
        ty = y0 + (a + b) / 2 * 28 + 12
        arrow(s, 246, y + 22, x0 - 4, ty, colour)
    lines(s, 630, 100, ['The meaning comes from all', 'of them together.', '', 'The file holds names,',
                        'meanings and evidence only,', 'never the game\'s data.'], 11.5, DIM, gap=18)
    s.text(24, 380, 'A port names each field after its row, and tests take their edge cases from the range column.', 11.5, DIM)
    return s.render()


DIAGRAMS = {
    'taxonomies-overview.svg': taxonomies_overview, 'data-dictionary-entry.svg': dictionary_entry,
}
