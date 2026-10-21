<!-- The setup prompt under the home page's title. The text is README.md's "setup-prompt"
     region, read when the site builds, so the overview and the home page show the same prompt.
     The markup is VitePress's own code block, so its copy button works. -->
<script setup lang="ts">
import readme from '../../../README.md?raw'

const region = readme.match(/<!-- #region setup-prompt -->([\s\S]*?)<!-- #endregion setup-prompt -->/)
const code = region?.[1].match(/```text\n([\s\S]*?)```/)?.[1] ?? ''
</script>

<template>
  <section v-if="code" class="home-setup vp-doc">
    <h2 id="set-it-up-with-your-agent">Set it up with your agent</h2>
    <p>
      Copy this prompt into your coding agent (Claude Code, Codex, Cursor or another). It sets
      up GameFrame and starts a project for your game. It asks before it installs anything.
    </p>
    <div class="language-text vp-adaptive-theme">
      <button title="Copy prompt" class="copy"></button>
      <span class="lang">prompt</span>
      <pre class="shiki"><code>{{ code }}</code></pre>
    </div>
  </section>
</template>

<style scoped>
.home-setup {
  max-width: 1152px;
  margin: 8px auto 40px;
  padding: 0 24px;
}
@media (min-width: 640px) { .home-setup { padding: 0 48px; } }
@media (min-width: 960px) { .home-setup { padding: 0 64px; } }
.home-setup h2 {
  margin-top: 0;
  border-top: none;
  padding-top: 0;
}
.home-setup pre code {
  white-space: pre-wrap;
}
</style>
