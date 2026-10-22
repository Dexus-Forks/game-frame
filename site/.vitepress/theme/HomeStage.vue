<!-- The home page's stage under the title: the animated picture of a game's bytes going
     through decompiling, tracing and porting into a rebuilt game (site/diagrams/banners.py). -->
<script setup lang="ts">
import src from '../../../docs/assets/diagrams/banner-hero.svg'
</script>

<template>
  <figure class="home-stage">
    <img :src="src" alt="Bytes from a game's executable flow through decompiling, tracing and porting, and a game scene is rebuilt tile by tile." />
  </figure>
</template>

<style scoped>
.home-stage {
  max-width: 1152px;
  margin: 8px auto 8px;
  padding: 0 24px;
}
.home-stage img {
  display: block;
  width: 100%;
  height: auto;
  border-radius: 16px;
  border: 1px solid var(--vp-c-divider);
  box-shadow: var(--gf-stage-shadow);
}
@media (max-width: 640px) {
  .home-stage { padding: 0 16px; }
  .home-stage img { border-radius: 10px; }
}
</style>
