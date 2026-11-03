<template>
  <div class="glass-panel" :class="{ 'is-visible': isVisible }" :style="panelStyle">
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  isVisible: boolean;
  position?: 'left' | 'right' | 'center';
  width?: string;
  maxHeight?: string;
}>();

const panelStyle = computed(() => ({
  width: props.width || '360px',
  maxHeight: props.maxHeight || '70vh',
  [props.position || 'right']: 'var(--space-8)',
}));
</script>

<style scoped>
.glass-panel {
  position: fixed;
  top: 50%;
  transform: translateY(-50%);
  border-radius: var(--radius-2xl);
  padding: var(--space-6);
  z-index: var(--z-modal);
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
  overflow-y: auto;
  overscroll-behavior: contain;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;

  /* Glass effect */
  background: var(--bg-glass);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border: 1px solid var(--border-hairline);

  /* Animation */
  opacity: 0;
  pointer-events: none;
  transition: all var(--duration-normal) var(--ease-out-expo);
}

.glass-panel.is-visible {
  opacity: 1;
  pointer-events: auto;
}

.glass-panel[position="right"].is-visible {
  animation: slideInRight var(--duration-normal) var(--ease-out-expo);
}

.glass-panel[position="left"].is-visible {
  animation: slideInLeft var(--duration-normal) var(--ease-out-expo);
}

.glass-panel[position="center"].is-visible {
  animation: fadeIn var(--duration-normal) var(--ease-out-expo);
}

@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateY(-50%) translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateY(-50%) translateX(0);
  }
}

@keyframes slideInLeft {
  from {
    opacity: 0;
    transform: translateY(-50%) translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(-50%) translateX(0);
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-50%) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(-50%) scale(1);
  }
}
</style>
