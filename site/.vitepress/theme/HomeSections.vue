<!-- The home page's gallery of the guides: one illustrated card for each section, with the
     section's banner (site/diagrams/banners.py and art.py) and a line on what it covers. -->
<script setup lang="ts">
import { withBase } from 'vitepress'
import games from '../../../docs/assets/diagrams/art-games.svg'
import preservation from '../../../docs/assets/diagrams/banner-preservation.svg'
import hardware from '../../../docs/assets/diagrams/banner-hardware.svg'
import consoles from '../../../docs/assets/diagrams/banner-consoles.svg'
import handhelds from '../../../docs/assets/diagrams/banner-handhelds.svg'
import computers from '../../../docs/assets/diagrams/banner-computers.svg'
import arcade from '../../../docs/assets/diagrams/banner-arcade.svg'
import engines from '../../../docs/assets/diagrams/banner-engines.svg'
import media from '../../../docs/assets/diagrams/banner-media.svg'
import observe from '../../../docs/assets/diagrams/banner-observe.svg'
import working from '../../../docs/assets/diagrams/banner-working.svg'

const sections = [
  { title: 'Kinds of games', text: 'Ten families of games, who drives each one, and where the work is.', link: '/games/', img: games, wide: true },
  { title: 'Hardware of the era', text: 'CPUs, VGA, 3D cards and sound cards, as games used them.', link: '/hardware/', img: hardware },
  { title: 'Console games', text: 'From the NES to the seventh generation, and the routes to a port.', link: '/consoles/', img: consoles },
  { title: 'Handhelds', text: 'Game Boy to Switch, LCD games and phones.', link: '/handhelds/', img: handhelds },
  { title: 'Home computers', text: '8-bit and 16-bit machines, their chips and their disks and tapes.', link: '/computers/', img: computers },
  { title: 'Arcade games', text: 'Boards, protection and the emulators that run them.', link: '/arcade/', img: arcade },
  { title: 'Engines and middleware', text: 'id Tech, Build, Unreal, Flash and the libraries games shipped with.', link: '/engines/', img: engines },
  { title: 'Media and files', text: 'Floppies, discs, tapes, packers and copy protection.', link: '/media/', img: media },
  { title: 'Observe the original', text: 'Memory, inputs, frames and calls, captured from the running game.', link: '/observe/', img: observe },
  { title: 'Working on a project', text: 'Sessions, evidence, gates, versions and building the port.', link: '/working/', img: working },
  { title: 'Game preservation', text: 'Why games disappear, and how a project keeps one.', link: '/preservation/', img: preservation },
]
</script>

<template>
  <section class="home-sections">
    <h2 id="explore-the-guides">Explore the guides</h2>
    <div class="grid">
      <a v-for="s in sections" :key="s.link" :href="withBase(s.link)" class="card" :class="{ wide: s.wide }">
        <img :src="s.img" alt="" loading="lazy" />
        <span class="body">
          <span class="title">{{ s.title }}</span>
          <span class="text">{{ s.text }}</span>
        </span>
      </a>
    </div>
  </section>
</template>

<style scoped>
.home-sections {
  max-width: 1152px;
  margin: 24px auto 48px;
  padding: 0 24px;
}
.home-sections h2 {
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.01em;
  margin: 0 0 18px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 18px;
}
.card {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 14px;
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg-soft);
  color: inherit;
  text-decoration: none;
  transition: transform 0.2s, border-color 0.2s, box-shadow 0.2s;
}
.card:hover {
  transform: translateY(-3px);
  border-color: var(--vp-c-brand-1);
  box-shadow: var(--gf-stage-shadow);
}
.card img {
  display: block;
  width: 100%;
  aspect-ratio: 16 / 7;
  object-fit: cover;
  background: #0d1b1e;
}
.card.wide {
  grid-column: span 3;
}
.card.wide img {
  aspect-ratio: 3 / 1;
}
.body {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 14px 16px 16px;
}
.title {
  font-weight: 700;
  font-size: 16px;
  color: var(--vp-c-text-1);
}
.text {
  font-size: 14px;
  line-height: 1.5;
  color: var(--vp-c-text-2);
}
@media (max-width: 959px) {
  .grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .card.wide { grid-column: span 2; }
}
@media (max-width: 640px) {
  .home-sections { padding: 0 16px; }
  .grid { grid-template-columns: minmax(0, 1fr); }
  .card.wide { grid-column: span 1; }
  .card.wide img { aspect-ratio: 16 / 7; }
}
</style>
