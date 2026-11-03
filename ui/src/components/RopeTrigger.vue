<template>
  <div
    class="rope-trigger"
    data-hit
    :class="{ 'is-hovered': isHovered }"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @click="handleClick"
  >
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
      <path d="M12 2v20M2 12h20" stroke-linecap="round"/>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits<{
  (e: 'toggle'): void;
}>();

const isHovered = ref(false);

const handleClick = () => {
  emit('toggle');
};
</script>

<style scoped>
.rope-trigger {
  position: fixed;
  top: 8px;
  left: 8px;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: rgba(168, 85, 247, 0.6);
  border: 1.5px solid rgba(200, 170, 255, 0.7);
  color: rgba(255, 255, 255, 0.95);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 1000;
  transition: all 0.25s ease;
  backdrop-filter: blur(8px);
  box-shadow: 0 2px 12px rgba(168, 85, 247, 0.45);
}

.rope-trigger:hover {
  background: rgba(168, 85, 247, 0.85);
  transform: scale(1.12);
  box-shadow: 0 4px 18px rgba(168, 85, 247, 0.6);
}

.rope-trigger.is-hovered {
  background: rgba(168, 85, 247, 0.75);
}
</style>
