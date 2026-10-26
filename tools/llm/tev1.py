#!/usr/bin/env python3
"""Ask a local Ollama Tev1 model bounded questions about supplied state.

Usage:
  tev1.py REQUEST.json [--model ID] [--url URL] [--timeout SECONDS]
  tev1.py - [options] < REQUEST.json

REQUEST.json is an object with `state` and `questions`. State is text, an object or an array.
Questions is an object with 1 to 64 named questions. Each question has `type` (`choice`,
`noul` or `score`), `instructions`, and the type's optional or required `criteria` described
by Ollama's Tev1 API. Choice and score questions need 2 to 24 criteria entries.

Example request:
  {"state":"Static code only; no runtime comparison.","questions":{
    "next":{"type":"choice","instructions":"What should happen next?",
      "criteria":{"measure":"Run and compare the original.","none":"No listed step fits."}},
    "parity":{"type":"noul","instructions":"Is runtime parity proven?"},
    "evidence":{"type":"score","instructions":"How strong is the evidence?",
      "criteria":["Static code","Original measured","Parity test passed"]}}}

The tool posts the request to Ollama's `/v1/systemone` endpoint and prints its JSON answer.
The default endpoint is local at http://127.0.0.1:11434/v1/systemone; use `--url` only when
you have chosen another Ollama server. Install a model separately, for example with
`ollama pull tev1:4b`.

What it does not do: start Ollama, download a model, choose whether the agent should ask a
question, write code, verify a claim, or treat Tev1's confidence as the chance that an answer
is correct. Its answer is advisory. Review it against the game's evidence.
"""
import argparse
import json
import math
import sys
import urllib.error
import urllib.parse
import urllib.request

DEFAULT_URL = 'http://127.0.0.1:11434/v1/systemone'
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
        if kind not in QUESTION_TYPES:
            raise ValueError(f'{name}: type must be choice, noul or score')
        if (not isinstance(question.get('instructions'), str)
                or not question['instructions'].strip()):
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
        raise ValueError('request exceeds Ollama Tev1 API limit of 64 KiB')
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f'request is not valid UTF-8 JSON: {error}') from error
    return _validate(value)


def _endpoint(url):
    parsed = urllib.parse.urlsplit(url)
    if (parsed.scheme not in ('http', 'https') or not parsed.hostname
            or parsed.username or parsed.password):
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


def _send(value, url, model, timeout):
    request_body = json.dumps({'model': model, **value}, ensure_ascii=False,
                              allow_nan=False, separators=(',', ':')).encode('utf-8')
    if len(request_body) > MAX_REQUEST_BYTES:
        raise ValueError('request exceeds Ollama Tev1 API limit of 64 KiB')
    request = urllib.request.Request(url, data=request_body,
                                     headers={'Content-Type': 'application/json'}, method='POST')
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = response.read(MAX_RESPONSE_BYTES + 1)
            if len(body) > MAX_RESPONSE_BYTES:
                raise ValueError('Ollama response exceeds 1 MiB')
    except urllib.error.HTTPError as error:
        detail = error.read(4096).decode('utf-8', errors='replace')
        raise RuntimeError(f'Ollama returned HTTP {error.code}: {detail}') from error
    except urllib.error.URLError as error:
        raise RuntimeError(f'cannot reach Ollama at {url}: {error.reason}') from error
    try:
        result = json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RuntimeError(f'Ollama returned invalid JSON: {error}') from error
    if not isinstance(result, dict) or not isinstance(result.get('answers'), dict):
        raise RuntimeError('Ollama response must contain an answers object')
    expected, received = set(value['questions']), set(result['answers'])
    if expected != received:
        raise RuntimeError('Ollama response question names do not match the request')
    return result


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0
    parser = argparse.ArgumentParser(description=__doc__, add_help=False)
    parser.add_argument('request', help='JSON request file, or - to read JSON from stdin')
    parser.add_argument('--model', default='tev1:4b',
                        help='installed Ollama model (default: tev1:4b)')
    parser.add_argument('--url', default=DEFAULT_URL,
                        help=f'Ollama API endpoint (default: {DEFAULT_URL})')
    parser.add_argument('--timeout', type=float, default=120,
                        help='request timeout in seconds (default: 120)')
    try:
        args = parser.parse_args(argv)
        if not args.model or not math.isfinite(args.timeout) or args.timeout <= 0:
            raise ValueError('model must be non-empty and timeout must be a positive finite number')
        value = _read_request(args.request)
        url = _endpoint(args.url)
        result = _send(value, url, args.model, args.timeout)
        print(json.dumps(result, ensure_ascii=False, indent=2, allow_nan=False))
        return 0
    except (OSError, ValueError, RuntimeError) as error:
        print(f'error: {error}', file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
