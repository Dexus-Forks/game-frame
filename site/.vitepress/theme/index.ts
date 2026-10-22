// The default theme, plus the home page's animated stage under its title, the setup prompt
// (from README.md), a gallery of the guides' sections before its features, a "Last updated"
// line under them, and a light layer of brand colours and image frames (custom.css).
// Doc pages already show the date from git (lastUpdated in config.mts), and a link to their
// plain Markdown copy.
import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import HomeLastUpdated from './HomeLastUpdated.vue'
import HomeSections from './HomeSections.vue'
import HomeSetupPrompt from './HomeSetupPrompt.vue'
import HomeStage from './HomeStage.vue'
import MarkdownLink from './MarkdownLink.vue'
import FileExplorer from './lab/FileExplorer.vue'
import PixelExplorer from './lab/PixelExplorer.vue'
import RecordsExplorer from './lab/RecordsExplorer.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    // The browser tools of /lab/, used in its pages.
    app.component('FileExplorer', FileExplorer)
    app.component('PixelExplorer', PixelExplorer)
    app.component('RecordsExplorer', RecordsExplorer)
  },
  Layout: () => h(DefaultTheme.Layout, null, {
    'home-hero-after': () => [h(HomeStage), h(HomeSetupPrompt)],
    'home-features-before': () => h(HomeSections),
    'home-features-after': () => h(HomeLastUpdated),
    'doc-footer-before': () => h(MarkdownLink),
  }),
}
