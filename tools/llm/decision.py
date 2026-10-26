#!/usr/bin/env python3
"""Ask a configured decision model bounded choice, yes-or-no and scoring questions.

Usage:
  decision.py REQUEST.json [--model ID] [--url URL] [--timeout SECONDS]
  decision.py - [options] < REQUEST.json

REQUEST.json contains `state` and `questions`. State may be text, an object or an array.
Each question has `type` (`choice`, `noul` or `score`), `instructions`, and optional or
required `criteria`. Choice and score questions need 2 to 24 options.

The tool calls the System One decision endpoint at `/v1/systemone`. Set
GAMEFRAME_DECISION_ENDPOINT and GAMEFRAME_DECISION_MODEL to route it to a decision model. If no
decision endpoint is set, local mode builds one from GAMEFRAME_LLM_ENDPOINT. `--url` and
`--model` select a route for one call.
The endpoint and model are user choices. Local model use is the default; the tool never
switches to a paid provider. It asks before sending to a non-loopback endpoint and stops if no
confirmation is available. It does not start a server or download a model.

Example:
  {"state":"Static code only; no runtime comparison.","questions":{
    "next":{"type":"choice","instructions":"What should happen next?",
      "criteria":{"measure":"Run and compare the original.","none":"No listed step fits."}},
    "parity":{"type":"noul","instructions":"Is runtime parity proven?"},
    "evidence":{"type":"score","instructions":"How strong is the evidence?",
      "criteria":["Static code","Original measured","Parity test passed"]}}}

What it does not do: decide whether an agent should ask a person, write code, verify a claim or
treat confidence as the chance an answer is correct. Output is advisory. Check it against the
game's evidence.
"""
import argparse
import json
import math
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

from client import BASE_URL, MODEL, PROVIDER, confirm_remote_request  # noqa: E402

MAX_REQUEST_BYTES = 64 * 1024
MAX_RESPONSE_BYTES = 1024 * 1024
QUESTION_TYPES = {'choice', 'noul', 'score'}


def _validate(value):
    if not isinstance(value, dict) or set(value) != {'state', 'questions'}:
        raise ValueError('request must contain only state and questions')
    if not isinstance(value['state'], (str, dict, list)):
        raise ValueError('state must be text, an object or an array')
    questions = value['questions']
    if not isinstance(questions, dict) or not 1 <= len(questions) <= 64:
        raise ValueError('questions must be an object with 1 to 64 named questions')
    for name, question in questions.items():
        if not isinstance(name, str) or not name or not isinstance(question, dict):
            raise ValueError('each question needs a non-empty name and an object value')
        kind = question.get('type')
        if not isinstance(kind, str) or kind not in QUESTION_TYPES:
            raise ValueError(f'{name}: type must be choice, noul or score')
        if not isinstance(question.get('instructions'), str) or not question['instructions'].strip():
            raise ValueError(f'{name}: instructions must be non-empty text')
        criteria = question.get('criteria')
        if kind == 'choice':
            if not isinstance(criteria, dict) or not 2 <= len(criteria) <= 24:
                raise ValueError(f'{name}: choice criteria must have 2 to 24 options')
            if any(not isinstance(option, str) or not option
                   or description is not None and not isinstance(description, str)
                   for option, description in criteria.items()):
                raise ValueError(f'{name}: choice criteria map options to text or null')
        elif kind == 'score':
            if not isinstance(criteria, list) or not 2 <= len(criteria) <= 24:
                raise ValueError(f'{name}: score criteria must have 2 to 24 levels')
            if any(not isinstance(level, str) or not level for level in criteria):
                raise ValueError(f'{name}: each score level must be non-empty text')
        elif criteria is not None:
            if not isinstance(criteria, dict) or not set(criteria) <= {'true', 'false'}:
                raise ValueError(f'{name}: noul criteria may describe true and false')
            if any(not isinstance(description, str) for description in criteria.values()):
                raise ValueError(f'{name}: noul criteria descriptions must be text')
    return value


def _read_request(path):
    if path == '-':
        raw = sys.stdin.buffer.read(MAX_REQUEST_BYTES + 1)
    else:
        with open(path, 'rb') as source:
            raw = source.read(MAX_REQUEST_BYTES + 1)
    if len(raw) > MAX_REQUEST_BYTES:
        raise ValueError('request exceeds decision API limit of 64 KiB')
    try:
        return _validate(json.loads(raw))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f'request is not valid UTF-8 JSON: {error}') from error


def _default_endpoint():
    endpoint = os.environ.get('GAMEFRAME_DECISION_ENDPOINT')
    if endpoint:
        return _endpoint(endpoint)
    if PROVIDER != 'local':
        raise ValueError('Set GAMEFRAME_DECISION_ENDPOINT or pass --url to select a decision server.')
    if not BASE_URL:
        raise ValueError('Set GAMEFRAME_DECISION_ENDPOINT or GAMEFRAME_LLM_ENDPOINT to a decision server.')
    parsed = urllib.parse.urlsplit(BASE_URL)
    path = parsed.path.rstrip('/')
    if path.endswith('/v1'):
        path += '/systemone'
    else:
        path += '/v1/systemone'
    return _endpoint(urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, path, '', '')))


def _endpoint(url):
    parsed = urllib.parse.urlsplit(url)
    if parsed.scheme not in ('http', 'https') or not parsed.hostname or parsed.username or parsed.password:
        raise ValueError('url must be an HTTP(S) endpoint without embedded credentials')
    if parsed.query or parsed.fragment:
        raise ValueError('url must not have a query string or fragment')
    if not parsed.path.endswith('/v1/systemone'):
        raise ValueError('url must end with /v1/systemone')
    try:
        parsed.port
    except ValueError as error:
        raise ValueError(f'invalid endpoint port: {error}') from error
    return url


def _send(value, url, model, timeout, api_key=None):
    if not model or not isinstance(model, str):
        raise ValueError('choose a decision model with --model or GAMEFRAME_DECISION_MODEL')
    request_body = json.dumps({'model': model, **value}, ensure_ascii=False,
                              allow_nan=False, separators=(',', ':')).encode('utf-8')
    if len(request_body) > MAX_REQUEST_BYTES:
        raise ValueError('request exceeds decision API limit of 64 KiB')
    headers = {'Content-Type': 'application/json'}
    if api_key:
        headers['Authorization'] = f'Bearer {api_key}'
    request = urllib.request.Request(url, data=request_body, headers=headers, method='POST')
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read(MAX_RESPONSE_BYTES + 1)
            if len(body) > MAX_RESPONSE_BYTES:
                raise ValueError('decision provider response exceeds 1 MiB')
    except urllib.error.HTTPError as error:
        detail = error.read(4096).decode('utf-8', errors='replace')
        raise RuntimeError(f'decision provider returned HTTP {error.code}: {detail}') from error
    except urllib.error.URLError as error:
        raise RuntimeError(f'cannot reach decision provider at {url}: {error.reason}') from error
    try:
        result = json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RuntimeError(f'decision provider returned invalid JSON: {error}') from error
    if not isinstance(result, dict) or not isinstance(result.get('answers'), dict):
        raise RuntimeError('decision response must contain an answers object')
    if set(result['answers']) != set(value['questions']):
        raise RuntimeError('decision response question names do not match the request')
    return result


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0
    parser = argparse.ArgumentParser(description=__doc__, add_help=False)
    parser.add_argument('request', help='JSON request file, or - to read JSON from stdin')
    parser.add_argument('--model', default=(os.environ.get('GAMEFRAME_DECISION_MODEL') or MODEL),
                        help='decision model identifier')
    parser.add_argument('--url', default=None, help='System One endpoint; overrides configured endpoint')
    parser.add_argument('--timeout', type=float, default=120, help='request timeout in seconds (default: 120)')
    try:
        args = parser.parse_args(argv)
        if not math.isfinite(args.timeout) or args.timeout <= 0:
            raise ValueError('timeout must be a positive finite number')
        endpoint = _endpoint(args.url) if args.url else _default_endpoint()
        if args.request == '-':
            confirm_remote_request(endpoint, 'decision request')
            value = _read_request(args.request)
        else:
            value = _read_request(args.request)
            confirm_remote_request(endpoint, 'decision request')
        api_key = os.environ.get('GAMEFRAME_DECISION_API_KEY')
        result = _send(value, endpoint, args.model, args.timeout, api_key)
        print(json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False))
        return 0
    except (OSError, ValueError, RuntimeError) as error:
        print(f'error: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
