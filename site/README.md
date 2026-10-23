# The documentation site

A [VitePress](https://vitepress.dev) site built from the repository's own files. It holds no
copies of the documentation: the pages are read where they are.

```bash
npm --prefix site install
npm --prefix site run dev      # http://localhost:5173
npm --prefix site run build    # site/.vitepress/dist
```

| On the site | Comes from |
|---|---|
| `/` | `site/index.md`, the only page written for the site |
| `/overview` | `README.md` |
| `/guide` and the menu's sections | `docs/README.md`: each `## ` section is a menu group, in order |
| `/games/`, `/observe/`, `/case-studies/` and their pages | `docs/<folder>/README.md` and the files it links to, in the order it links them |
| `/game-patterns`, `/dos`, `/abi`, `/setup`, `/legal` | `docs/*.md` (URLs in lower case) |
| `/skills/<name>` | `skills/<name>/SKILL.md`, through the dynamic route `site/skills/[skill].md`, followed by the agents that preload it and the tools it names |
| `/agents/<name>` | `agents/<name>.md`, through `site/agents/[agent].md` |
| `/tools/<name>` | The tool's own help (its docstring or header comment, as `./gameframe tool NAME` prints it), through `site/tools/[tool].md` |
| `/skills/`, `/agents/`, `/tools/` | Index pages made from the same files, through `site/[section]/index.md` |
| `/examples/<name>` | `examples/<name>/README.md` |

`.github/workflows/site.yml` builds it on every push to `main` that changes the docs, the
skills, the examples or the site, and publishes it to GitHub Pages
(https://tanepiper.github.io/game-frame/), with `SITE_BASE` set to the Pages path.

To add a page, write the Markdown file in `docs/` and link it from its folder's `README.md`
(or from `docs/README.md`): it appears in the menu in that place. A subfolder's `README.md`
linked from a folder's `README.md` becomes a group of its own, one level deep
(`docs/preservation/law/`).

To group a folder's pages in the menu, list them in its `README.md` under two or more `## `
headings, one list item per page (as `docs/computers/README.md` does). Each heading becomes a
group; a heading with one page shows that page. A `docs/README.md` section that links several
folders shows each folder as its own group.

Links between files stay
relative, as GitHub needs them; the site turns them into page URLs (`.vitepress/links.mts`).
A link to a tool or a script, and inline code that starts with a tool's path
(`tools/triage/memscan.py`), lead to that tool's page. Other files link to GitHub.
`.vitepress/catalog.mts` builds the skills', agents' and tools' pages.

The guides' diagrams are SVG files in `docs/assets/diagrams/`, drawn by `site/diagrams/make.py`
from the numbers they illustrate. Some are animated with SVG's own animation, which plays on
GitHub and on the site with no script. Change a diagram in `make.py`, then run
`python3 site/diagrams/make.py`. The tests fail when a file differs from what the generator
draws. The build fails on a link to a page
that does not exist.

After the build, `.vitepress/llms.mts` writes text for AI agents into the output, with no extra
dependency:

- a plain Markdown copy of every page beside its HTML (`/games/interaction.md`,
  `/games/index.md`, `/skills/re-start.md`): front matter removed, relative links pointing at
  the other `.md` copies, skill pages from their `SKILL.md`;
- `/llms.txt`, an index in the [llms.txt](https://llmstxt.org) format: every page with its first
  sentence, in the order of the menu;
- `/llms-full.txt`, every page in one file.

Each page names its copy in a `<link rel="alternate" type="text/markdown">` and in a small link
under the page. The workflow sets `SITE_URL` so the links in `llms.txt` are full URLs; locally
they are paths.

Each page shows the date of the last commit to its source file, so the workflow checks out the
whole history. A skill page takes the date of its `SKILL.md`. The home page shows the date of
the newest commit to any page.

## Pictures

Pictures live in `docs/assets/`, and pages link them with relative paths, so they show on
GitHub too. Keep them small: SVG for diagrams and terminal output, compressed WebP for
screenshots. Never use a picture or data from a commercial game. The case-study screenshots of
our own ports are the one exception (see `AGENTS.md`).

| Picture | What |
|---|---|
| `docs/assets/method-loop.svg` | The method as a loop, drawn by hand; edit the SVG text directly |
| `docs/assets/captures/*.svg` | Terminal output of the tools on the sample game. After `tests/run.sh --ghidra`, run `site/captures/make.sh` to remake them |
| `docs/assets/case-studies/*.webp` | Frames of the case-study ports |

The home page's feature icons are inline SVG in `site/index.md`, drawn with `currentColor`, so
they take the brand colour in both modes.

| File | What |
|---|---|
| `.vitepress/config.mts` | The site's settings |
| `.vitepress/repo.mts` | Which files are pages, and the URL of each |
| `.vitepress/sidebar.mts` | The menu, read from `docs/README.md` and the folder READMEs |
| `.vitepress/links.mts` | Relative links resolved for the site |
| `.vitepress/llms.mts` | The Markdown copies, `llms.txt` and `llms-full.txt` |
| `.vitepress/theme/` | The default theme, plus the home page's "Last updated" line, its hero picture, each page's Markdown link, and `custom.css`: brand colours, heading rules, image frames, the feature icons' tiles |
| `captures/make.sh`, `captures/term_svg.py` | Remake the terminal pictures in `docs/assets/captures/` from the tools, run on the sample game |
| `skills/[skill].md`, `skills/[skill].paths.mts` | One page per skill |
