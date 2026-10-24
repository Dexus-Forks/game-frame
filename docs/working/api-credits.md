# API credits

Max and Team plans come with monthly credits for the Claude Platform. Max 5x gets $100 a
month and Max 20x gets $200. A Team plan gets $20 for each Standard seat and $100 for each
Premium seat, up to $500 in all. Credits that are not used expire at the end of each billing
cycle. Source: [Monthly API credits for Max and Team plans](https://support.claude.com/en/articles/17154008-monthly-api-credits-for-max-and-team-plans).

The credits pay for the Claude API, the Message Batches API, the Agent SDK and `claude -p` run
with an API key. They do not pay for interactive Claude Code in a terminal, an IDE, the
desktop app or the web. They do not pay for the Claude Code GitHub Action either.

The tools in `tools/llm/` use the credits for work that suits them: large batches, unattended
runs and checks that repeat each month.

## Set up

1. On claude.ai, open Settings, then Billing. In the API credits section, link a Console
   organization. A Team plan's owner does this in Organization settings.
2. In the Console, create an API key in that organization.
3. Put the key in the project's `.env` as `ANTHROPIC_API_KEY=...`. The file is ignored by git.
4. In the Console, set a spend limit on the key's workspace. The tools have their own limits,
   and the workspace limit is the one that cannot be bypassed.
5. Install the SDK: `scripts/setup.sh --install`, or `.venv/bin/pip install -r requirements-llm.txt`.
6. Check the setup with `./gameframe run tools/llm/client.py`.

`claude -p --bare` reads only `ANTHROPIC_API_KEY` and never your plan's login, so a headless
run cannot spend your plan's usage by mistake.

These tools send the game's decompiled code and data to the Claude API, the same as an
interactive session does. The [legal notes](../LEGAL.md) cover what that can mean for you.

## The tools

### Name every unnamed function

`name_functions.py` sends each unnamed function of a Ghidra export to the Batch API. Each
request holds the pseudocode, the callers, the callees and the strings the function uses.
The answer is a name, a signature, a summary, a confidence and the evidence.

```bash
./gameframe run tools/llm/name_functions.py prepare --binary GAME.EXE --max 2000
./gameframe run tools/llm/name_functions.py submit --binary GAME.EXE
./gameframe run tools/llm/name_functions.py status --binary GAME.EXE
./gameframe run tools/llm/name_functions.py fetch --binary GAME.EXE
```

`prepare` sends nothing. It prints the number of functions and their size in tokens, so you
can decide before you submit. The Batch API costs half the normal price. Most batches end
within an hour.

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

```bash
./gameframe run tools/llm/headless.py re-decoder "Decode every .PAK file and prove the decoder with validate.py" --write --budget 10
```

Without `--write` the agent can read files and run commands, and it changes nothing. With
`--write`, run it on a branch or a worktree of its own and read the diff before you merge.

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

## A month of credits

Credits expire at the end of each cycle, so plan work that spends them before then. One way
to plan a month of $200:

| When | What | Limit |
|---|---|---|
| A new game | `name_functions.py` over its main binaries | `--max` from the `prepare` count |
| After skill edits | `skillcheck.py` | `--budget 1` a task |
| Each week | `headless.py docs-check` | `--budget 5` |
| The last days | `headless.py` with a long task the project has put off | what is left |

Schedule the repeating runs with cron or launchd. Each tool stops at its own limit. The
workspace spend limit in the Console stops everything.

## Tests

`tests/run.sh` checks every tool against `tools/llm/fake_api.py`, a stand-in for the API and
for the `claude` command. The tests send nothing and spend nothing.
