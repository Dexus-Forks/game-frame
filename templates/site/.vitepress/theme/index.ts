// The site's theme: VitePress's default theme with the project's own colours (custom.css).
// Give the site the look of its game: take the colours from the game's palette or its
// interface, as hex values typed here. No fonts are bundled and nothing from the game is
// loaded: custom.css uses the reader's system fonts.
import type { Theme } from 'vitepress'
import DefaultTheme from 'vitepress/theme'
import './custom.css'

export default {
  extends: DefaultTheme,
} satisfies Theme
