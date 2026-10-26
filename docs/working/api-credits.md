# Model providers and API credits

Max and Team plans come with monthly credits for the Claude Platform. Max 5x gets $100 a
month and Max 20x gets $200. A Team plan gets $20 for each Standard seat and $100 for each
Premium seat, up to $500 in all. Credits that are not used expire at the end of each billing
cycle. Source: [Monthly API credits for Max and Team plans](https://support.claude.com/en/articles/17154008-monthly-api-credits-for-max-and-team-plans).

The credits pay for the Claude API, the Message Batches API, the Agent SDK and `claude -p` run
with an API key. They do not pay for interactive Claude Code in a terminal, an IDE, the
desktop app or the web. They do not pay for the Claude Code GitHub Action either.

The model tools in `tools/llm/` default to the local provider. They support Anthropic Messages
and OpenAI-compatible Chat Completions. Set `GAMEFRAME_LLM_ENDPOINT` for a local server or a
custom OpenAI-compatible service. To use a named hosted service, set
`GAMEFRAME_LLM_PROVIDER=anthropic` or `openai` and that provider's API key. Those providers use
their standard endpoints and ignore `GAMEFRAME_LLM_ENDPOINT`, so a local URL saved in `.env`
does not receive a hosted key. A saved key alone does not select a hosted service. The generic
decision tool can use a server's separate native decision endpoint. See
[working with agents](agents.md#generic-decision-tool).

These tools are optional. A task being self-contained, suitable for automation or covered by
available credits is not a request to run it. Continue in the current session by default. Ask the
user before making a paid API call, starting an unattended agent or sending project material to
the provider. Hosted model calls and inference to non-loopback endpoints stop at a confirmation
prompt and require the user to type `yes`; they fail closed when no answer is available. Batch
status, fetch and cancel operations do not run inference. Ask separately before setting up
recurring runs.

## Set up

1. On claude.ai, open Settings, then Billing. In the API credits section, link a Console
   organization. A Team plan's owner does this in Organization settings.
2. In the Console, create an API key in a workspace of that organization.
3. Put the key in the project's `.env` as `GAMEFRAME_ANTHROPIC_API_KEY=...`. The file is ignored by git.
   A key that is not scoped to a workspace is refused with "must include the
   anthropic-workspace-id header". Use a workspace's key instead, or add the workspace's ID to
   `.env` as `GAMEFRAME_ANTHROPIC_WORKSPACE_ID=...`. The key alone does not select Anthropic.
4. In the Console, set a spend limit on the key's workspace. The tools have their own limits,
   and the workspace limit is the one that cannot be bypassed.
5. Install the SDK: `scripts/setup.sh --install`, or `.venv/bin/pip install -r requirements-llm.txt`.
6. Check the setup with `./gameframe run tools/llm/client.py`.

GameFrame passes the configured API key to `claude -p --bare`. It does not reuse your
interactive Claude Code login for a headless run.

The `claude` command must be recent enough for the model. An old one fails with "does not
support this model": run `claude update`, or set `GAMEFRAME_CLAUDE` to a newer one. A run
started from inside a Claude Code session would inherit that session's `CLAUDE*` variables,
which mark it as part of that session; the credits would then refuse it with "Credit balance
is too low". `client.py` leaves those variables out of every run it starts.

When Anthropic is selected, the tools can send the game's decompiled code and data to Claude, the
same as an interactive session does. The [legal notes](../LEGAL.md) cover what that can mean for
you.

## Use a local model

`tools/llm/client.py` reads `GAMEFRAME_LLM_PROVIDER`, `GAMEFRAME_LLM_ENDPOINT`,
`GAMEFRAME_LLM_PROTOCOL`, the selected provider's model setting and provider-specific API keys
from the environment or `.env`. Local is the default provider. Its
endpoint can point to a local server or any compatible server. Local mode sends only
`GAMEFRAME_LOCAL_LLM_API_KEY`, or the value `local` if that key is unset. It never sends the
hosted provider keys.

For an Ollama, LM Studio or llama.cpp server with an Anthropic-compatible endpoint, set:

```dotenv
GAMEFRAME_LLM_PROTOCOL=anthropic
GAMEFRAME_LLM_PROVIDER=local
GAMEFRAME_LLM_ENDPOINT=http://127.0.0.1:11434
GAMEFRAME_LOCAL_MODEL=qwen3-coder
GAMEFRAME_LOCAL_LLM_API_KEY=local
```

Use the server's address and a model it has loaded. LM Studio commonly listens on port 1234;
llama.cpp's port depends on how it was started. This setup supports the direct-call tools and
the Claude Code runner used by `headless.py` and `skillcheck.py`, provided the model can use
tools. These servers document Anthropic-compatible Messages APIs; see [Ollama](https://docs.ollama.com/api/anthropic-compatibility),
[LM Studio](https://lmstudio.ai/docs/developer/anthropic-compat) and
[llama.cpp](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md).

For any OpenAI-compatible Chat Completions endpoint, use:

```dotenv
GAMEFRAME_LLM_PROTOCOL=openai
GAMEFRAME_LLM_PROVIDER=local
GAMEFRAME_LLM_ENDPOINT=http://127.0.0.1:1234/v1
GAMEFRAME_LOCAL_MODEL=your-loaded-model-id
GAMEFRAME_LOCAL_LLM_API_KEY=local
```

For OpenAI's hosted API, set `GAMEFRAME_LLM_PROVIDER=openai`,
`GAMEFRAME_OPENAI_API_KEY=...` and optionally `GAMEFRAME_OPENAI_MODEL` to a model available to
your account. It uses OpenAI's standard endpoint. For Anthropic's hosted API, set
`GAMEFRAME_LLM_PROVIDER=anthropic`, `GAMEFRAME_ANTHROPIC_API_KEY` and optionally
`GAMEFRAME_ANTHROPIC_MODEL`; it uses Anthropic's standard endpoint. Both named providers ignore
`GAMEFRAME_LLM_ENDPOINT`, so no endpoint clearing is needed
when a local URL is saved in `.env`.
A custom OpenAI-compatible service uses `GAMEFRAME_LLM_PROVIDER=openai-compatible` with
`GAMEFRAME_LLM_ENDPOINT`, `GAMEFRAME_OPENAI_COMPATIBLE_API_KEY` and optionally
`GAMEFRAME_OPENAI_COMPATIBLE_MODEL` set for that service. `GAMEFRAME_MODEL` overrides the
selected model for one run.

For a one-off frontier task after local work, select the hosted service in that command's
environment. For example:

```bash
GAMEFRAME_LLM_PROVIDER=anthropic \
  ./gameframe run tools/llm/ask.py "<question>"
```

Store the key in `.env` as `GAMEFRAME_ANTHROPIC_API_KEY` instead of putting it directly in a shell
history. The client reads it for this command because it names the Anthropic provider. Keep
`GAMEFRAME_LOCAL_MODEL`, `GAMEFRAME_ANTHROPIC_MODEL`, the local endpoint and both keys in `.env`.
Changing `GAMEFRAME_LLM_PROVIDER` selects the saved provider settings; the tool asks before a
hosted request. `GAMEFRAME_MODEL` is a one-run override and is not loaded from `.env`.

This protocol supports `ask.py`, `locate.py` and synchronous function naming. The selected
model must support image input to use `locate.py`. `headless.py` and `skillcheck.py` use Claude
Code's tool loop, so configure an Anthropic-compatible endpoint for those agent runs. Model
servers can differ in schema, image and tool support; choose a model and endpoint that provide
the features the task needs.

Anthropic-compatible hosted batches are used only when no custom endpoint is set. Local
function naming runs one request at a time, saves progress under `analysis/`, and retries failed
requests when `submit` is run again. To use batches through another server, set
`GAMEFRAME_LLM_BATCHES=1` only when it implements the Messages Batches API.

## Route decision questions

`decision.py` sends choice, yes-or-no and score questions to a native System One endpoint. The
model server and decision model are selected separately from the chat model, so a server can
serve different models for each job. Set these in the project `.env` or in the command's
environment:

```dotenv
GAMEFRAME_DECISION_ENDPOINT=http://127.0.0.1:PORT/v1/systemone
GAMEFRAME_DECISION_MODEL=your-decision-model-id
```

`GAMEFRAME_DECISION_API_KEY` is optional and is sent only to this endpoint. If
`GAMEFRAME_DECISION_ENDPOINT` is omitted, a local run appends `/v1/systemone` to
`GAMEFRAME_LLM_ENDPOINT`. Hosted decision routes must be selected with the endpoint or `--url`;
the command never derives a hosted decision route from the chat endpoint. See
`./gameframe tool decision` for request types and limits. The server must implement the System
One request format and have a decision model loaded; a regular chat model that only supports
chat completions is not enough. See the [LM Studio System One guide](https://lmstudio.ai/docs/developer/jev-compat/systemone)
and the [llama.cpp server API](https://github.com/ggml-org/llama.cpp/tree/master/tools/server).

The tool returns the selected server's response. It does not calibrate confidence or establish
evidence. Keep explicit corrections with `decision_feedback.py`; its export can prepare local
training examples but does not change model weights or upload data.

## The tools

### Name every unnamed function

`name_functions.py` sends each unnamed function of a Ghidra export to the configured model. Each
request holds the pseudocode, the callers, the callees and the strings the function uses.
The answer is a name, a signature, a summary, a confidence and the evidence.

```bash
./gameframe run tools/llm/name_functions.py prepare --binary GAME.EXE --max 2000
./gameframe run tools/llm/name_functions.py submit --binary GAME.EXE
./gameframe run tools/llm/name_functions.py status --binary GAME.EXE
./gameframe run tools/llm/name_functions.py fetch --binary GAME.EXE
./gameframe run tools/llm/name_functions.py cancel --binary GAME.EXE
```

`prepare` sends nothing. It prints the number of functions and their size in tokens, so you
can decide before you submit. The Batch API costs half the normal price.

With the hosted Anthropic API, `submit` sends the requests as batches of 200 (`--size N`). One batch of thousands can sit
for most of the 24 hours the API allows, and none of its answers can be fetched until it
ends. Small batches end sooner, mostly within an hour. `fetch` reads every batch that has
ended, so run it again as more end. `cancel` stops the ones still running. Compatible local
and OpenAI APIs run synchronously and keep each result as it completes; `status` and `fetch`
read that local progress file, and `--size` is ignored.

The results go to `analysis/llm/names/<binary>/suggested.tsv`, sorted by confidence. They are
guesses from pseudocode. Confirm each one as `re-trace` does before you add it to
`labels/names.tsv`, and label it *inferred* until you have. The tool never writes `labels/`.

### Check that each skill works on its own

`skillcheck.py` tests rule 4 of `AGENTS.md`: an agent that loads one skill must be able to do
that step from it alone. Each task in `tests/skill-tasks.json` names a skill, a task on the
sample game and the strings the answer must hold. The tool gives an agent the skill and an
empty folder with the sample's binaries. It passes when the agent's `answer.txt` holds the
ground truth.

```bash
./gameframe run tools/llm/skillcheck.py --budget 1 --report analysis/llm/skillcheck.md
```

Add a task when a skill changes. Take the expected strings from the sample's own ground truth.

### Run an agent unattended

`headless.py` runs one of the agents in `agents/` with its skills and a budget. It saves the
agent's final report to `analysis/llm/runs/`.

Use it only after the user asks or agrees. A self-contained task or a good fit for unattended
work is not permission to start the run. Ask before creating a branch or worktree for it.

```bash
./gameframe run tools/llm/headless.py re-decoder "Decode every .PAK file and prove the decoder with validate.py" --write --budget 10
```

Without `--write` the agent can read files and run commands, and it changes nothing. Before a
`--write` run, agree with the user where changes should go. Use the branch or worktree they
choose, and read the diff before merging. Do not create an isolated checkout by default.

`headless.py docs-check` runs `re-checker` and then `re-editor` over the documents. It changes
nothing. The report lists missing evidence, addresses and files that do not exist, gaps that
are already covered, and sentences that break [the writing rules](writing.md).

```bash
./gameframe run tools/llm/headless.py docs-check docs skills GAPS.md --budget 5
```

### Ask the know-how

`ask.py` answers a question from the know-how entries. It finds them with the search that
`gameframe advise` uses, sends the best eight, and names each entry it uses.

```bash
./gameframe run tools/llm/ask.py "the game crashes with a privileged instruction under Wine"
./gameframe run tools/llm/ask.py --show "a privileged instruction under Wine"
```

`--show` lists the entries it would send and spends nothing.

### Find something on the screen

`locate.py` asks a model where an element is in a capture, or where a reference screen sits in
it. Use it when `tools/live/align.py` reports a weak match because the screen has changed. It
gives positions as fractions of the picture, so they survive the API's resizing. One call
uses one model request. A vision-capable model is required; with a local server, images stay on
that server.

```bash
./gameframe run tools/llm/locate.py derived/captures/now.png --find "the Exit button" --units 2
./gameframe run tools/llm/locate.py derived/captures/now.png --ref derived/port-menu.png --save derived/align.json
```

## A month of credits

If the user chooses to spend credits, a budget can help keep the work within the limit. This is
an optional example for a month of $200, not a schedule to enact:

| Possible timing | What | Limit |
|---|---|---|
| A new game | `name_functions.py` over its main binaries | `--max` from the `prepare` count |
| After skill edits | `skillcheck.py` | `--budget 1` a task |
| Each week | `headless.py docs-check` | `--budget 5` |
| The last days | `headless.py` with a long task the project has put off | what is left |

Only set up recurring runs with cron or launchd when the user asks for them. Each tool stops at
its own limit. The workspace spend limit in the Console stops everything.

## Tests

`tests/run.sh` checks the OpenAI-compatible client against a local stand-in and the Anthropic
tools against `tools/llm/fake_api.py`, a stand-in for that API and for the `claude` command.
The tests send nothing to a real model and spend nothing.
