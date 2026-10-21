// The default theme, plus a "Last updated" line under the home page's features.
// Doc pages already show the date from git (lastUpdated in config.mts).
import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import HomeLastUpdated from './HomeLastUpdated.vue'

export default {
  extends: DefaultTheme,
  Layout: () => h(DefaultTheme.Layout, null, {
    'home-features-after': () => h(HomeLastUpdated),
  }),
}
