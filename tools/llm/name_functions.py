#!/usr/bin/env python3
"""Suggest a name, a signature and a summary for every unnamed function.

Usage:
  name_functions.py prepare [--export DIR] [--binary NAME] [--min-size N] [--max N] [--all]
  name_functions.py submit  [--binary NAME] [--model ID] [--size N]
  name_functions.py status  [--binary NAME]
  name_functions.py fetch   [--binary NAME]
  name_functions.py cancel  [--binary NAME]

  prepare   picks the functions and writes one request each to
            analysis/llm/names/<binary>/requests.jsonl, then prints how many and their size.
            Nothing is sent. Read the count before you submit.
  submit    sends them as batches of --size requests each to APIs with batch support. With a
            compatible local or OpenAI API, it runs requests synchronously and saves results
            beside them. Small hosted batches end sooner and can be fetched one by one
  status    prints batch progress or the local synchronous results
  fetch     writes the answers of every batch that has ended to
            analysis/llm/names/<binary>/suggested.tsv: address, name, confidence
            (low/medium/high), signature, summary, evidence. Run it again as more batches end
  cancel    cancels hosted batches that have not ended; local requests run synchronously

  --export    the Ghidra export (default: analysis/export in the project)
  --binary    the binary in the export (default: the only one, or the first .exe)
  --min-size  skip functions smaller than N bytes (default 16)
  --max       send at most N functions, the largest first (default 2000)
  --all       include functions that already have a name (to check the names)
  --model     the model (default: selected provider profile; GAMEFRAME_MODEL overrides one run)
  --size      requests per hosted batch (default 200; ignored by synchronous local APIs)

Each request holds the function's pseudocode, the names of its callers and callees, and the
strings it uses. Functions named in labels/names.tsv are skipped.

The suggestions are guesses from pseudocode, so they stay in analysis/ (ignored by git).
Confirm a suggestion as re-trace does before you copy it to labels/names.tsv, and label it
*inferred* until you have.

What it does not do: spend control. --max limits the number of requests. Set provider spend
limits for hosted APIs. Submitting model requests requires a typed confirmation; status, fetch
and cancel do not run inference. It never writes labels/ and never applies names in Ghidra.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(os.path.dirname(HERE), 'triage'))
from client import MODEL, PROJECT, api, text_of  # noqa: E402
from decomp_query import FUN, Export, fun_addr  # noqa: E402

SYSTEM = """You name functions in a decompiled game binary. You get one function's Ghidra \
pseudocode, the names of its callers and callees, and the strings it uses. Give the name the \
original programmer most likely used, in the style the binary's own names show. Give a C \
signature with useful parameter names. Write one plain sentence on what it does. Name the \
evidence: the string, constant, call or field access that supports the name. Use confidence \
"low" when the code alone does not show the purpose."""

SCHEMA = {'type': 'object', 'additionalProperties': False,
          'required': ['name', 'confidence', 'signature', 'summary', 'evidence'],
          'properties': {'name': {'type': 'string'}, 'signature': {'type': 'string'},
                         'confidence': {'type': 'string', 'enum': ['low', 'medium', 'high']},
                         'summary': {'type': 'string'}, 'evidence': {'type': 'string'}}}
MAX_BODY = 24000  # characters of pseudocode sent; longer bodies are cut and marked


def folder(binary):
    return os.path.join(PROJECT, 'analysis', 'llm', 'names', binary)


WORD = re.compile(r'\b([A-Za-z_][\w:]*)\s*\(')


def callees(ex, addr):
    """The functions a body names: FUN_ references, then calls by name."""
    seen = []
    body = ex.bodies.get(addr, '')
    found = [fun_addr(m) for m in FUN.finditer(body)] + [ex.by_name.get(m.group(1)) for m in WORD.finditer(body)]
    for a in found:
        if a is not None and a != addr and a not in seen:
            seen.append(a)
    return seen


def caller_index(ex):
    """{callee: [callers]} from one pass over every body."""
    index = {}
    for a in ex.bodies:
        for c in callees(ex, a):
            index.setdefault(c, []).append(a)
    return index


def prepare(ex, min_size, limit, everything):
    strings = []
    path = os.path.join(ex.dir, 'strings.txt')
    if os.path.exists(path):
        strings = [l.rstrip('\n').partition('\t')[2] for l in open(path, encoding='utf-8', errors='replace')]
    strings = [s for s in strings if len(s) >= 4]
    picked = [(a, n, size) for a, (n, size) in ex.functions.items()
              if a in ex.bodies and size >= min_size and a not in ex.labels
              and (everything or n.startswith('FUN_'))]
    picked.sort(key=lambda t: -t[2])
    picked = picked[:limit]
    index = caller_index(ex)
    out = folder(ex.binary)
    os.makedirs(out, exist_ok=True)
    total = 0
    with open(os.path.join(out, 'requests.jsonl'), 'w', encoding='utf-8') as f:
        for a, n, size in picked:
            body = ex.label(ex.bodies[a])
            if len(body) > MAX_BODY:
                body = body[:MAX_BODY] + '\n/* ... cut: the function is longer */\n'
            used = [s for s in strings if s[:60] in body][:20]
            text = (f'Binary: {ex.binary}\nFunction: {n} at {ex.fmt(a)}, {size} bytes\n'
                    f'Called by: {", ".join(ex.name(c) for c in index.get(a, [])[:20]) or "(no direct callers)"}\n'
                    f'Calls: {", ".join(ex.name(c) for c in callees(ex, a)[:30]) or "(nothing)"}\n'
                    f'Strings: {json.dumps(used)}\n\n```c\n{body}```')
            total += len(text)
            f.write(json.dumps({'custom_id': ex.fun(a), 'text': text}) + '\n')
    print(f'{len(picked)} functions, {total:,} characters (about {total // 4:,} input tokens) '
          f'in {os.path.relpath(out, PROJECT)}/requests.jsonl')
    print('Nothing was sent. Next: name_functions.py submit')


def submit(binary, model, size):
    if size < 1:
        sys.exit('--size must be at least 1')
    out = folder(binary)
    rows = [json.loads(l) for l in open(os.path.join(out, 'requests.jsonl'), encoding='utf-8')]
    reqs = [{'custom_id': r['custom_id'], 'params': {
        'model': model, 'max_tokens': 4000, 'system': SYSTEM,
        'output_config': {'effort': 'low', 'format': {'type': 'json_schema', 'schema': SCHEMA}},
        'messages': [{'role': 'user', 'content': r['text']}]}} for r in rows]
    # Small batches: one large batch can sit for most of the 24 hours the API allows, and
    # nothing in it can be fetched until it ends.
    ids, c = [], api()
    if not c.supports_batches:
        path = os.path.join(out, 'local-results.jsonl')
        completed = {}
        if os.path.exists(path):
            for line in open(path, encoding='utf-8'):
                try:
                    result = json.loads(line)
                    completed[result['custom_id']] = result
                except (json.JSONDecodeError, KeyError, TypeError):
                    continue
        remaining = [r for r in rows if completed.get(r['custom_id'], {}).get('type') != 'succeeded']
        params_by_id = {r['custom_id']: req['params'] for r, req in zip(rows, reqs)}
        succeeded = sum(r.get('type') == 'succeeded' for r in completed.values())
        with open(path, 'a', encoding='utf-8') as f:
            for index, r in enumerate(remaining, 1):
                try:
                    response = c.messages.create(**params_by_id[r['custom_id']])
                    result = {'custom_id': r['custom_id'], 'type': 'succeeded', 'text': text_of(response)}
                    succeeded += 1
                except Exception as error:  # keep the failed request retryable on the next submit
                    result = {'custom_id': r['custom_id'], 'type': 'errored', 'error': str(error)}
                f.write(json.dumps(result, ensure_ascii=False) + '\n')
                f.flush()
                print(f'local {index}/{len(remaining)}: {r["custom_id"]} {result["type"]}')
        print(f'{succeeded} local results in {os.path.relpath(path, PROJECT)}; '
              f'{max(0, len(rows) - succeeded)} pending or failed')
        return
    with open(os.path.join(out, 'batch.txt'), 'w') as f:
        for i in range(0, len(reqs), size):
            batch = c.messages.batches.create(requests=reqs[i:i + size])
            ids.append(batch.id)
            f.write(batch.id + '\n')
            f.flush()
            print(f'batch {batch.id}: {len(reqs[i:i + size])} requests, {batch.processing_status}')
    print(f'{len(ids)} batches, {len(reqs)} requests')


def batch_ids(binary):
    path = os.path.join(folder(binary), 'batch.txt')
    if not os.path.exists(path):
        sys.exit('no batch yet: name_functions.py submit')
    return [l.strip() for l in open(path) if l.strip()]


def status(binary):
    c = api(confirm=False)
    if not c.supports_batches:
        path = os.path.join(folder(binary), 'local-results.jsonl')
        latest = {}
        if os.path.exists(path):
            for line in open(path, encoding='utf-8'):
                try:
                    result = json.loads(line)
                    latest[result['custom_id']] = result
                except (json.JSONDecodeError, KeyError, TypeError):
                    continue
        succeeded = sum(r.get('type') == 'succeeded' for r in latest.values())
        failed = sum(r.get('type') == 'errored' for r in latest.values())
        request_path = os.path.join(folder(binary), 'requests.jsonl')
        with open(request_path, encoding='utf-8') as f:
            count = sum(1 for _ in f)
        print(f'local: {succeeded} succeeded, {failed} failed, '
              f'{max(0, count - succeeded - failed)} pending')
        return
    total = {'processing': 0, 'succeeded': 0, 'errored': 0, 'expired': 0, 'canceled': 0}
    ended, client = 0, c
    ids = batch_ids(binary)
    for i in ids:
        b = client.messages.batches.retrieve(i)
        c = b.request_counts
        for k in total:
            total[k] += getattr(c, k)
        ended += b.processing_status == 'ended'
        print(f'{b.id}: {b.processing_status}; {c.processing} processing, {c.succeeded} succeeded, '
              f'{c.errored} errored, {c.expired} expired')
    state = 'ended' if ended == len(ids) else 'in_progress'
    print(f'all {len(ids)} batches: {state}; {ended} ended; ' + ', '.join(f'{v} {k}' for k, v in total.items()))


def cancel(binary):
    c = api(confirm=False)
    if not c.supports_batches:
        print('local requests run synchronously; nothing remains to cancel')
        return
    for i in batch_ids(binary):
        b = c.messages.batches.retrieve(i)
        if b.processing_status == 'in_progress':
            b = c.messages.batches.cancel(i)
        print(f'{b.id}: {b.processing_status}')


def clean(text):
    return re.sub(r'[\t\n]+', ' ', str(text)).strip()


def fetch(binary):
    rows, failed, waiting = [], 0, 0
    c = api(confirm=False)
    if not c.supports_batches:
        path = os.path.join(folder(binary), 'local-results.jsonl')
        results = {}
        if os.path.exists(path):
            for line in open(path, encoding='utf-8'):
                try:
                    result = json.loads(line)
                    results[result['custom_id']] = result
                except (json.JSONDecodeError, KeyError, TypeError):
                    failed += 1
        iterable = []
        for custom_id, result in results.items():
            if result.get('type') != 'succeeded':
                failed += 1
                continue
            iterable.append((custom_id, result.get('text', '')))
    else:
        # Keep the client alive while each batch result stream is read.
        results = []
        for i in batch_ids(binary):
            if c.messages.batches.retrieve(i).processing_status != 'ended':
                waiting += 1
                continue
            results.extend(c.messages.batches.results(i))
        iterable = []
        for r in results:
            if r.result.type != 'succeeded':
                failed += 1
                continue
            iterable.append((r.custom_id, text_of(r.result.message)))
    for custom_id, text in iterable:
        try:
            d = json.loads(text)
        except json.JSONDecodeError:
            failed += 1
            continue
        addr = custom_id.removeprefix('FUN_').replace('_', ':')
        rows.append([addr] + [clean(d.get(k, '')) for k in ('name', 'confidence', 'signature', 'summary', 'evidence')])
    order = {'high': 0, 'medium': 1, 'low': 2}
    rows.sort(key=lambda r: (order.get(r[2], 3), r[0]))
    path = os.path.join(folder(binary), 'suggested.tsv')
    with open(path, 'w', encoding='utf-8') as f:
        f.write('# address\tname\tconfidence\tsignature\tsummary\tevidence (suggestions: confirm before labelling)\n')
        f.writelines('\t'.join(r) + '\n' for r in rows)
    print(f'{len(rows)} suggestions in {os.path.relpath(path, PROJECT)}; {failed} failed or unreadable'
          + (f'; {waiting} batches not ended yet, fetch again later' if waiting else ''))


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__); return 0
    cmd, argv = argv[0], argv[1:]
    opts = {'--export': os.path.join(PROJECT, 'analysis', 'export'), '--binary': None,
            '--min-size': '16', '--max': '2000', '--model': MODEL, '--size': '200'}
    everything = False
    while argv:
        if argv[0] == '--all':
            everything, argv = True, argv[1:]
        elif argv[0] in opts and len(argv) > 1:
            opts[argv[0]], argv = argv[1], argv[2:]
        else:
            sys.exit(f'unknown option {argv[0]}')
    ex = Export(opts['--export'], opts['--binary'])
    if cmd == 'prepare':
        prepare(ex, int(opts['--min-size']), int(opts['--max']), everything)
    elif cmd == 'submit':
        submit(ex.binary, opts['--model'], int(opts['--size']))
    elif cmd == 'status':
        status(ex.binary)
    elif cmd == 'fetch':
        fetch(ex.binary)
    elif cmd == 'cancel':
        cancel(ex.binary)
    else:
        sys.exit(f'unknown command {cmd}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
