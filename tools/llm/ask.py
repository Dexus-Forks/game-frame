#!/usr/bin/env python3
"""Ask a question of GameFrame's know-how in plain words, and get an answer that cites entries.

Usage:
  ask.py [--top N] [--model ID] [--show] QUESTION ...

  QUESTION  a question, an error message or a description of what you see
  --top     how many entries to send (default 8)
  --show    print the entries that would be sent and stop, spending nothing

It finds the entries with `gameframe advise`'s own search (tools/gameframe/advise.py: error
signatures, facets, then words; GameFrame's entries and the project's), sends the best N
whole, and asks Claude to answer from them alone, naming each entry it uses. When no entry
fits, the answer says so, and the question may be a new gap (re-land, "Know-how").

One question costs one Messages API call with a few thousand tokens. It uses the API key
(client.py).

What it does not do: search the web, the code or the docs outside docs/knowhow/. An answer
is a pointer to entries: read the entry before you act on it.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'gameframe'))
from client import MODEL, api, text_of  # noqa: E402
import advise  # noqa: E402

SYSTEM = """You answer questions from people who reverse engineer games with GameFrame. \
Answer only from the know-how entries given. Name each entry you use by its id in backticks. \
If no entry covers the question, say so in one sentence, and say what to record if they solve \
it. Be short: the fix or the fact first, then the command."""


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    top, model, show, words = 8, MODEL, False, []
    while argv:
        a = argv.pop(0)
        if a == '--top' and argv:
            top = int(argv.pop(0))
        elif a == '--model' and argv:
            model = argv.pop(0)
        elif a == '--show':
            show = True
        else:
            words.append(a)
    question = ' '.join(words)
    hits = advise.search(question, cwd=os.getcwd())[:top]
    if show:
        for h in hits:
            print(f'{h.score:4d}  {h.entry.id}: {h.entry.title}')
        return 0
    if not hits:
        print('No know-how entry matches. If you solve it, record it (re-land, "Know-how").')
        return 0
    context = '\n\n'.join(f'<entry id="{h.entry.id}" status="{h.entry.status}">\n# {h.entry.title}\n'
                          f'{h.entry.body}\n</entry>' for h in hits)
    msg = api().messages.create(model=model, max_tokens=4000, system=SYSTEM,
                                messages=[{'role': 'user', 'content': f'{context}\n\nQuestion: {question}'}])
    print(text_of(msg))
    print(f'\n(entries sent: {", ".join(h.entry.id for h in hits)})')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
