#!/usr/bin/env python3
"""Check opt-in MCP configuration and project-local provider launch settings.

Usage: python3 tests/mcp_provider_config.py
Inputs: GameFrame's provider catalog and a temporary project directory.
Outputs: fails when provider opt-in, generated client settings, or local build overrides break.
It does not install or connect to any external MCP server.
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GAMEFRAME = ROOT / 'gameframe'
CATALOG = json.loads((ROOT / 'tools/gameframe/mcp_providers.json').read_text())


def run(project, *args):
    env = {**os.environ, 'GAMEFRAME_PROJECT': str(project)}
    return subprocess.run([str(GAMEFRAME), *args], cwd=ROOT, env=env,
                          capture_output=True, text=True)


def executable(path, text):
    path.write_text(text)
    path.chmod(0o755)


def main():
    with tempfile.TemporaryDirectory(prefix='gameframe-mcp-providers-') as temp:
        project = Path(temp)
        listed = run(project, 'mcp', '--list')
        assert listed.returncode == 0, listed.stderr
        for name, info in CATALOG.items():
            assert f'{name:<12} off' in listed.stdout, (name, listed.stdout)
            assert info['version'] in listed.stdout, (name, listed.stdout)

        # The generator refreshes only managed MCP entries and preserves local settings.
        (project / '.gemini').mkdir()
        (project / '.gemini/settings.json').write_text(json.dumps({
            'mcpServers': {'custom': {'command': 'custom-server'}}, 'other': True}))
        (project / '.cursor').mkdir()
        (project / '.cursor/mcp.json').write_text(json.dumps({
            'mcpServers': {'custom': {'command': 'custom-server'}}, 'inputs': []}))
        (project / '.vscode').mkdir()
        (project / '.vscode/mcp.json').write_text(json.dumps({
            'servers': {'custom': {'command': 'custom-server'}}, 'inputs': []}))
        (project / 'opencode.json').write_text(json.dumps({
            'mcp': {'custom': {'type': 'local', 'command': ['custom-server']}}}))
        codex = project / '.codex/config.toml'
        codex.parent.mkdir()
        codex.write_text('model = "keep-me"\n')
        copilot = project / '.github/agents'
        copilot.mkdir(parents=True)
        for name in ('re-accelerator.agent.md', 're-cataloguer.agent.md'):
            shutil.copyfile(ROOT / '.github/agents' / name, copilot / name)
        customized = copilot / 're-cataloguer.agent.md'
        customized.write_text(customized.read_text() + '\n<!-- project customization -->\n')
        emitted = run(project, 'mcp', '--emit', 'all')
        assert emitted.returncode == 0, emitted.stderr

        enabled = run(project, 'mcp', '--enable', 'rea', 'mcp-emulate', '86box')
        assert enabled.returncode == 0, enabled.stderr
        assert 'rea (6.7.0)' in enabled.stdout and 'mcp-emulate (0.1.0)' in enabled.stdout
        assert 'Refreshed Copilot MCP access in 1 generated agent files.' in enabled.stdout, enabled.stdout
        assert 'Kept customized Copilot agent files:' in enabled.stdout, enabled.stdout
        config = json.loads((project / '.mcp.json').read_text())['mcpServers']
        assert set(config) == {'rea', 'mcp-emulate', '86box'}
        for name in config:
            assert config[name]['command'] == './gameframe'
            assert config[name]['args'][:2] == ['mcp-provider', name]

        codex_config = tomllib.loads(codex.read_text())
        assert codex_config['model'] == 'keep-me'
        for name in config:
            assert codex_config['mcp_servers'][name]['command'] == './gameframe'
        assert json.loads((project / '.gemini/settings.json').read_text())['mcpServers']['custom']
        assert json.loads((project / '.cursor/mcp.json').read_text())['mcpServers']['custom']
        assert json.loads((project / '.vscode/mcp.json').read_text())['servers']['custom']
        assert json.loads((project / 'opencode.json').read_text())['mcp']['custom']
        copilot_tools = json.loads(re.search(
            r'^tools: (.*)$', (copilot / 're-accelerator.agent.md').read_text(), re.M).group(1))
        assert {'rea/*', 'mcp-emulate/*', '86box/*', 'gameframe/*'} <= set(copilot_tools), copilot_tools
        assert '<!-- project customization -->' in customized.read_text()
        assert 'rea/*' not in customized.read_text()

        disabled = run(project, 'mcp', '--disable', 'rea')
        assert disabled.returncode == 0, disabled.stderr
        assert 'rea' not in json.loads((project / '.mcp.json').read_text())['mcpServers']
        assert 'rea' not in tomllib.loads(codex.read_text())['mcp_servers']
        assert 'mcp-emulate' in tomllib.loads(codex.read_text())['mcp_servers']
        refreshed_tools = (copilot / 're-accelerator.agent.md').read_text()
        assert '"mcp-emulate/*"' in refreshed_tools and '"86box/*"' in refreshed_tools
        assert '"rea/*"' not in refreshed_tools

        # Adding a client after provider opt-in generates both its server config and Copilot allowlist.
        late = project / 'late-copilot'
        late.mkdir()
        (late / '.mcp.json').write_text(json.dumps({'mcpServers': {'rea': CATALOG['rea']['server']}}))
        added = subprocess.run([sys.executable, str(ROOT / 'scripts/clients.py'), 'write', str(late),
                                '--agent', 'copilot', '--plugin'], cwd=ROOT,
                               capture_output=True, text=True)
        assert added.returncode == 0, added.stderr
        late_agent = (late / '.github/agents/re-accelerator.agent.md').read_text()
        assert '"rea/*"' in late_agent, late_agent.splitlines()[3]
        late_servers = json.loads((late / '.vscode/mcp.json').read_text())['servers']
        assert 'rea' in late_servers, late_servers

        # MCP launch wrappers load .env so local custom server and emulator paths work in clients.
        fake_server = project / 'fake-dosbox-mcp'
        executable(fake_server, '#!/bin/sh\nprintf "custom-server:%s\\n" "$DOSBOX_X_BINARY"\n')
        fake_emulator = project / 'custom-dosbox-x'
        executable(fake_emulator,
                   '#!/bin/sh\nif [ "${1:-}" = "-version" ]; then echo "DOSBox-X version 2026.10.01"; exit; fi\n'
                   'printf "%s\\n" "$@" > "$FAKE_DOSBOX_LOG"\n')
        (project / '.env').write_text(
            f'DOSBOX_X_MCP_SERVER={fake_server}\nDOSBOX_X_BINARY={fake_emulator}\n')
        launched = run(project, 'mcp-provider', 'dosbox', 'v0.1.2')
        assert launched.returncode == 0, launched.stderr
        assert f'custom-server:{fake_emulator}' in launched.stdout, launched.stdout

        checked = subprocess.run(['bash', '-c', 'source scripts/lib.sh; dosbox_version'], cwd=ROOT,
                                 env={**os.environ, 'GAMEFRAME_PROJECT': str(project)},
                                 capture_output=True, text=True)
        assert checked.returncode == 0 and checked.stdout.strip() == '2026.10.01', checked.stderr
        setup = subprocess.run(['bash', 'scripts/setup.sh'], cwd=ROOT,
                               env={**os.environ, 'GAMEFRAME_PROJECT': str(project)},
                               capture_output=True, text=True)
        assert 'dosbox-x       ok       DOSBox-X version 2026.10.01' in setup.stdout, setup.stdout
        game = project / 'game'
        game.mkdir()
        (game / 'SAMPLE.EXE').write_bytes(b'sample')
        call_log = project / 'dosbox-call.txt'
        runner = subprocess.run(['bash', 'scripts/dosbox-run.sh', '--out', str(project / 'out'),
                                 '--time', '1', str(game), 'SAMPLE.EXE'], cwd=ROOT,
                                env={**os.environ, 'GAMEFRAME_PROJECT': str(project),
                                     'FAKE_DOSBOX_LOG': str(call_log)},
                                capture_output=True, text=True)
        assert runner.returncode == 0 and 'exit code 0' in runner.stdout, runner.stderr
        assert '-conf' in call_log.read_text(), call_log.read_text()

        # 86Box requires the selected source revision and an executable local custom build.
        source = project / '86box-mcp'
        source.mkdir()
        subprocess.run(['git', 'init', '-q', str(source)], check=True)
        subprocess.run(['git', '-C', str(source), 'config', 'user.email', 'test@example.invalid'], check=True)
        subprocess.run(['git', '-C', str(source), 'config', 'user.name', 'GameFrame test'], check=True)
        (source / 'README').write_text('fixture\n')
        subprocess.run(['git', '-C', str(source), 'add', 'README'], check=True)
        subprocess.run(['git', '-C', str(source), 'commit', '-qm', 'fixture'], check=True)
        revision = subprocess.run(['git', '-C', str(source), 'rev-parse', 'HEAD'], check=True,
                                  capture_output=True, text=True).stdout.strip()
        fake_box_server = project / 'fake-86box-mcp'
        executable(fake_box_server, '#!/bin/sh\nprintf "custom-86box:%s\\n" "$BOX86_BINARY"\n')
        fake_box = project / '86Box'
        executable(fake_box, '#!/bin/sh\nexit 0\n')
        (project / '.env').write_text(
            f'BOX86_MCP_DIR={source}\nBOX86_MCP_REVISION={revision}\n'
            f'BOX86_MCP_SERVER={fake_box_server}\nBOX86_BINARY={fake_box}\n')
        launched = run(project, 'mcp-provider', '86box', CATALOG['86box']['source_revision'])
        assert launched.returncode == 0, launched.stderr
        assert f'custom-86box:{fake_box}' in launched.stdout, launched.stdout

    print('MCP providers: opt-in, generated clients, version pins, project .env and local builds')


if __name__ == '__main__':
    main()
