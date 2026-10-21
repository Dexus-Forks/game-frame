// The default theme, plus a picture of the tests beside the home page's title, the setup prompt
// under it (from README.md), a "Last updated"
// line under its features, and a light layer of brand colours and image frames (custom.css).
// Doc pages already show the date from git (lastUpdated in config.mts), and a link to their
// plain Markdown copy.
import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import HomeHeroImage from './HomeHeroImage.vue'
import HomeLastUpdated from './HomeLastUpdated.vue'
import HomeSetupPrompt from './HomeSetupPrompt.vue'
import MarkdownLink from './MarkdownLink.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, {
    'home-hero-image': () => h(HomeHeroImage),
    'home-hero-after': () => h(HomeSetupPrompt),
    'home-features-after': () => h(HomeLastUpdated),
    'doc-footer-before': () => h(MarkdownLink),
  }),
}
