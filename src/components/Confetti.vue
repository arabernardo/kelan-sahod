<script setup lang="ts">
// CSS-only emoji burst for payday. Pieces are computed once; the animation runs once.
const EMOJIS = ['🎉', '💸', '💰', '🪙', '✨']
const COUNT = 20

const pieces = Array.from({ length: COUNT }, (_, i) => {
  const angle = (i / COUNT) * Math.PI * 2
  const distance = 110 + (i % 4) * 38
  return {
    emoji: EMOJIS[i % EMOJIS.length],
    style: {
      '--dx': `${Math.round(Math.cos(angle) * distance)}px`,
      '--dy': `${Math.round(Math.sin(angle) * distance - 40)}px`,
      '--rot': `${(i % 2 === 0 ? 1 : -1) * (120 + i * 17)}deg`,
      '--delay': `${(i % 5) * 70}ms`,
    },
  }
})
</script>

<template>
  <div class="confetti" aria-hidden="true">
    <span v-for="(piece, i) in pieces" :key="i" class="confetti__piece" :style="piece.style">
      {{ piece.emoji }}
    </span>
  </div>
</template>
