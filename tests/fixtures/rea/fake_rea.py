"""A stand-in REA MCP server for tests/run.sh: the tools rea_session.py calls, with fixed answers.

It prints a non-JSON-RPC line first, as npx does before REA, and keeps REA's rules that the
tools depend on: a snapshot replays the dossiers saved in it, and a session that renamed
anything refuses to save a snapshot. Answers name the callee FUN_<addr> so the tests can see
the labels applied by the summary.
"""
import json
import sys

state = {'names': {}, 'renamed': False, 'cache': {}, 'calls': 0}


def dossier(addr):
    return {
        'evidence_id': 'ev_' + addr[2:].rjust(64, '0'),
        'normalized_result': {
            'procedure': {'address': addr, 'name': 'FUN_' + addr[2:],
                          'body': {'total_bytes': 16}},
            'basic_blocks': [{}, {}],
            'incoming_references': [{'source_address': '0x140100000', 'source_procedure': None}],
            'callees': [{'address': '0x140001100', 'name': 'FUN_140001100'}],
            'native_value_flow': {'effects': [{'kind': 'memory_read'}, {'kind': 'memory_write'}]},
        },
    }


def call(name, args):
    if name == 'open_binary':
        if args.get('snapshot_path'):
            state['cache'] = json.load(open(args['snapshot_path']))
        return {'path': args['path'], 'sha256': '0' * 64}
    if name == 'analyze_function':
        addr = args['procedure']
        if addr not in state['cache'] or state['renamed']:
            state['calls'] += 1
            state['cache'][addr] = dossier(addr)
        return state['cache'][addr]
    if name == 'set_addresses_names':
        state['renamed'] = True
        state['names'].update(args['names'])
        return {'normalized_result': {a: True for a in args['names']}}
    if name == 'close_binary':
        if args.get('snapshot_path'):
            if state['renamed']:
                raise ValueError('metadata edits invalidate immutable analysis snapshots')
            json.dump(state['cache'], open(args['snapshot_path'], 'w'))
        print(f'fake-rea: {state["calls"]} analysed', file=sys.stderr)
        return None
    raise ValueError(f'unknown tool {name}')


print(json.dumps({'type': 'message', 'message': 'printing as NDJSON'}), flush=True)
for line in sys.stdin:
    msg = json.loads(line)
    if 'id' not in msg:
        continue
    if msg['method'] == 'initialize':
        result = {'protocolVersion': msg['params']['protocolVersion'], 'capabilities': {}}
    else:
        try:
            value = call(msg['params']['name'], msg['params']['arguments'])
            result = {'content': [{'type': 'text', 'text': json.dumps({'result': value})}]}
        except ValueError as error:
            result = {'isError': True, 'content': [{'type': 'text', 'text': str(error)}]}
    print(json.dumps({'jsonrpc': '2.0', 'id': msg['id'], 'result': result}), flush=True)
