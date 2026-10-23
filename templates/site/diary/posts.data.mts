// The diary entries for the index page, newest first.
import { createContentLoader } from 'vitepress'

export default createContentLoader('diary/2*.md', {
  transform: (pages) =>
    pages
      .map((p) => ({
        url: p.url,
        title: p.frontmatter.title as string,
        date: p.url.match(/(\d{4}-\d{2}-\d{2})/)?.[1] ?? '',
        summary: (p.frontmatter.summary as string) ?? '',
      }))
      .sort((a, b) => b.date.localeCompare(a.date)),
})
