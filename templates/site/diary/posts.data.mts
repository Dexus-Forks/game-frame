// The diary entries for the index page, newest first: the day posts, and today's live entries
// (YYYY-MM-DD-HHMM-slug), which show their time.
import { createContentLoader } from 'vitepress'

export default createContentLoader('diary/2*.md', {
  transform: (pages) =>
    pages
      .map((p) => ({
        url: p.url,
        title: p.frontmatter.title as string,
        date: p.url.match(/(\d{4}-\d{2}-\d{2})(?:-([01]\d|2[0-3])([0-5]\d)-)?/)?.slice(1).reduce((d, x, i) => x ? d + (i === 1 ? ' ' : i === 2 ? ':' : '') + x : d, '') ?? '',
        summary: (p.frontmatter.summary as string) ?? '',
      }))
      .sort((a, b) => b.url.localeCompare(a.url)),
})
