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
| `/skills/<name>` | `.claude/skills/<name>/SKILL.md`, through the dynamic route `site/skills/[skill].md` |
| `/examples/<name>` | `examples/<name>/README.md` |

`.github/workflows/site.yml` builds it on every push to `main` that changes the docs, the
skills, the examples or the site, and publishes it to GitHub Pages
(https://tanepiper.github.io/game-frame/), with `SITE_BASE` set to the Pages path.

To add a page, write the Markdown file in `docs/` and link it from its folder's `README.md`
(or from `docs/README.md`): it appears in the menu in that place. A subfolder's `README.md`
linked from a folder's `README.md` becomes a group of its own, one level deep
(`docs/preservation/research/`). Links between files stay
relative, as GitHub needs them; the site turns them into page URLs, and links to tools and
scripts into links to GitHub (`.vitepress/links.mts`). The build fails on a link to a page
that does not exist.

Each page shows the date of the last commit to its source file, so the workflow checks out the
whole history. A skill page takes the date of its `SKILL.md`. The home page shows the date of
the newest commit to any page.

| File | What |
|---|---|
| `.vitepress/config.mts` | The site's settings |
| `.vitepress/repo.mts` | Which files are pages, and the URL of each |
| `.vitepress/sidebar.mts` | The menu, read from `docs/README.md` and the folder READMEs |
| `.vitepress/links.mts` | Relative links resolved for the site |
| `.vitepress/theme/` | The default theme, plus the home page's "Last updated" line |
| `skills/[skill].md`, `skills/[skill].paths.mts` | One page per skill |
