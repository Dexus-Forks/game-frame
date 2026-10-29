// The diary entries for the index page, newest first: the day posts, and the live entries
// (YYYY-MM-DD-HHMM-slug), which show their time. A past day's live entries stay on the site,
// linked from that day's post, and leave this list once the day post exists.
import { createContentLoader } from 'vitepress'

const LIVE = /\/(\d{4}-\d{2}-\d{2})-([01]\d|2[0-3])([0-5]\d)-[^/]*$/
const DAY = /\/(\d{4}-\d{2}-\d{2})-[^/]*$/

export default createContentLoader('diary/2*.md', {
  transform: (pages) => {
    const url = (p: { url: string }) => p.url.replace(/\.html$/, '')
    const dayPosts = new Set(pages.filter((p) => !LIVE.test(url(p))).map((p) => DAY.exec(url(p))?.[1]))
    return pages
      .filter((p) => !(LIVE.test(url(p)) && dayPosts.has(LIVE.exec(url(p))![1])))
      .map((p) => {
        const live = LIVE.exec(url(p))
        const day = DAY.exec(url(p))?.[1] ?? ''
        return {
          url: p.url,
          title: p.frontmatter.title as string,
          date: live ? `${live[1]} ${live[2]}:${live[3]}` : day,
          summary: (p.frontmatter.summary as string) ?? '',
        }
      })
      .sort((a, b) => b.url.localeCompare(a.url))
  },
})
