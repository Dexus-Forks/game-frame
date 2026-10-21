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

To add a page, write the Markdown file in `docs/` and link it from its folder's `README.md`
(or from `docs/README.md`): it appears in the menu in that place. Links between files stay
relative, as GitHub needs them; the site turns them into page URLs, and links to tools and
scripts into links to GitHub (`.vitepress/links.mts`). The build fails on a link to a page
that does not exist.

| File | What |
|---|---|
| `.vitepress/config.mts` | The site's settings |
| `.vitepress/repo.mts` | Which files are pages, and the URL of each |
| `.vitepress/sidebar.mts` | The menu, read from `docs/README.md` and the folder READMEs |
| `.vitepress/links.mts` | Relative links resolved for the site |
| `skills/[skill].md`, `skills/[skill].paths.mts` | One page per skill |
