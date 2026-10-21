<!-- The home page's hero picture: real output of tests/run.sh, made by site/captures/make.sh. -->
<script setup lang="ts">
import src from '../../../docs/assets/captures/hero.svg'
</script>

<template>
  <img class="home-hero-capture" :src="src" alt="tests/run.sh --ghidra: the port matches the original bit for bit, draws random numbers in the original's order, and the naive port is caught. 104 passed, 0 failed." />
</template>

<style scoped>
.home-hero-capture {
  position: relative;
  width: 100%;
  max-width: 520px;
  border-radius: 10px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.25);
}
</style>
