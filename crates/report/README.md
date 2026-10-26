# gameframe-report

Reports from a running port. A port marks each piece it leaves out with a `not ported` comment.
Where the game can reach such a piece while it runs, the port calls `unported!` there too. The
first time the game gets there, this crate writes a report. A crash becomes a report the same way.
Each report holds:

- the place in the port (file, line, column) and the original's address or function;
- the values at the place, given to the macro;
- the game's state, kept current by the game (the frame, the track, the save);
- the launch description and the devices chosen in the frame, so the run can be repeated;
- how many times the game came there, the platform, and the stack at the first time.

A report is written as one JSON file and one Markdown file, named by its id, such as
`unported-3f2a19c0`. The id is the same for every hit at one place. The Markdown is a GitHub
issue's body, laid out for an agent to debug from: where, what, the state, the launch to start
it again, and the steps to follow (trace, measure, port, verify, land).

```rust
use gameframe_report as report;

report::install(report::Meta {
    game: "Hardwar".into(),
    version: env!("CARGO_PKG_VERSION").into(),
    issues: "tanepiper/hardwar".into(),
    ..Default::default()
});
report::set_dir("reports");
report::set_launch(&launch);
// In the game loop:
report::set_state("frame", frame);
// At a path the port has not ported:
report::unported!("0x5898b4", "the material of a hit's other side", material = m);
// Where the port checks itself against the original:
report::mismatch!("0x489c1c", "fire control's stream A", ours = a, original = b);
// When the game ends:
report::flush();
```

## Dev and Production

| Mode | When | What a new report does |
|---|---|---|
| Dev | a debug build, or `GAMEFRAME_REPORT=dev` | Writes the folder's two files and one line on standard error. No link. An agent reads the folder, and `tools/triage/report_issue.py file` files one when you choose |
| Production | a release build, or `GAMEFRAME_REPORT=production` | Writes the home folder as `~` everywhere in the report, and sets `issue`: a link to GitHub's new-issue form for `Meta::issues`, with the title, the body and the labels `gameframe-report` and the kind filled in. The frame shows it as "Open a GitHub issue". The player reads the issue and sends it from the browser |

`set_mode` chooses in code. Nothing is sent over the network in either mode. A link only opens
the form, and the program holds no token. A link carries at most 6,000 characters of body: the
stack is left out first, and the JSON file holds everything.

## With the frame

`gameframe_frame::Link::send_reports(devices)` sends each new report to the frame as
`report.new`. The frame's Play tab lists them. In Dev each shows its file name. In Production
each gets one button per repository in `library.json`'s `issues`, such as the game's and
GameFrame's ([the library page](../../templates/library-page/README.md#links-and-reports)), each
opening that repository's form through `Report::issue_url_for`.

## Files

`<id>.json` has these fields: `id`, `kind` (`unported`, `panic`, `mismatch`), `what`,
`original`, `file`, `line`, `column`, `context`, `state`, `launch`, `devices`, `meta` (`game`,
`version`, `commit`, `issues`), `count`, `first`, `last` (seconds since 1970), `platform`,
`backtrace`, `mode` and `issue`. Any other tool may write a report in this form, such as an
emulator that meets an instruction it lacks. `tools/triage/report_issue.py` reads it.

## Strict mode

With `GAMEFRAME_STRICT=1` or `set_strict(true)`, `unported!` and `mismatch!` panic after
writing the report. Use it in tests that must not reach an unported path.

## What it does not do

It does not stop the game. An unported path goes on as the port wrote it. It does not find
unported paths by itself: the port names each one. It does not file issues: the player does in
Production, and you do in Dev with `report_issue.py`.

## Tests

`cargo test --release -p gameframe-report` in `crates/`. One report per place, counts, the
files, the sink, panics, strict mode, Dev and Production, and the issue's title, body and link.
