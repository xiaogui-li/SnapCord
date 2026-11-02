<template>
  <div class="double-bezel-card" :class="{ 'is-hovered': isHovered }" @mouseenter="isHovered = true" @mouseleave="isHovered = false">
    <div class="double-bezel-inner">
      <slot></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const isHovered = ref(false);
</script>

<style scoped>
.double-bezel-card {
  background: var(--bg-glass-light);
  border: 1px solid rgba(255, 255, 255, 0.05);
  padding: 6px;
  border-radius: var(--radius-3xl);
  transition: all var(--duration-normal) var(--ease-out-expo);
}

.double-bezel-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--shadow-glow);
}

.double-bezel-inner {
  background: rgba(20, 20, 20, 0.9);
  border-radius: calc(var(--radius-3xl) - 6px);
  box-shadow: inset 0 1px 1px rgba(255, 255, 255, 0.1);
  overflow: hidden;
}
</style>
