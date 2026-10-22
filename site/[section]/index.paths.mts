// The index pages /skills/, /agents/ and /tools/, made from the same files as their pages.
import { SECTIONS, indexPage } from '../.vitepress/catalog.mts'

export default {
  paths() {
    return SECTIONS.map(section => ({ params: { section, source: 'site/.vitepress/catalog.mts' }, content: indexPage(section) }))
  },
}
