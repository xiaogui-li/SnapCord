<template>
  <div
    class="image-card double-bezel"
    :class="{ 'is-hovered': isHovered, 'is-dragging': isDragging }"
    :style="cardStyle"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @click="handleClick"
    @dragstart="handleDragStart"
  >
    <div class="double-bezel-inner">
      <!-- Clip connecting to rope -->
      <div class="clip"></div>

      <!-- Image preview -->
      <div class="image-wrapper">
        <img :src="image.thumbnail" :alt="image.id" class="preview-image" />
      </div>

      <!-- Action buttons (hover) -->
      <div class="actions" v-show="isHovered">
        <button class="action-btn" @click.stop="handleCopy" title="复制">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
        </button>
        <button class="action-btn" @click.stop="handlePin" title="钉在桌面">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 2L12 22"></path>
            <path d="M17 7L12 2L7 7"></path>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface Image {
  id: string;
  path: string;
  thumbnail: string;
  timestamp: Date;
}

const props = defineProps<{
  image: Image;
  index: number;
  isExpanded: boolean;
}>();

const emit = defineEmits<{
  (e: 'click', image: Image): void;
  (e: 'drag-start', image: Image): void;
  (e: 'pin', image: Image): void;
}>();

const isHovered = ref(false);
const isDragging = ref(false);

const cardStyle = computed(() => ({
  animationDelay: `${props.index * 50}ms`,
  transform: isDragging.value ? 'scale(0.95)' : 'scale(1)',
}));

const handleClick = () => {
  emit('click', props.image);
};

const handleDragStart = (e: DragEvent) => {
  isDragging.value = true;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'copy';
    e.dataTransfer.setData('text/plain', props.image.path);
  }
  emit('drag-start', props.image);
  
  setTimeout(() => {
    isDragging.value = false;
  }, 100);
};

const handleCopy = () => {
  // TODO: Implement copy via Tauri command
  console.log('Copy image:', props.image.path);
};

const handlePin = () => {
  emit('pin', props.image);
};
</script>

<style scoped>
.image-card {
  width: 100px;
  height: 80px;
  flex-shrink: 0;
  cursor: pointer;
  transition: all var(--duration-normal) var(--ease-out-expo);
  position: relative;
  animation: cardEnter var(--duration-slower) var(--ease-out-expo) backwards;
}

@keyframes cardEnter {
  from {
    opacity: 0;
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.image-card:hover {
  transform: translateY(-8px);
  box-shadow: var(--shadow-glow);
}

.image-card.is-dragging {
  opacity: 0.5;
}

.double-bezel-inner {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.clip {
  position: absolute;
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  width: 12px;
  height: 16px;
  background: linear-gradient(180deg, #C0C0C0 0%, #808080 100%);
  border-radius: 2px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  z-index: 2;
}

.clip::before {
  content: '';
  position: absolute;
  top: 4px;
  left: 2px;
  right: 2px;
  height: 2px;
  background: rgba(0, 0, 0, 0.2);
}

.image-wrapper {
  width: 100%;
  height: 100%;
  padding: 4px;
}

.preview-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: calc(var(--radius-3xl) - 10px);
  pointer-events: none;
}

.actions {
  position: absolute;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: var(--space-2);
  opacity: 0;
  transition: opacity var(--duration-fast) var(--ease-out-expo);
}

.image-card:hover .actions {
  opacity: 1;
}

.action-btn {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  background: var(--bg-glass);
  border: 1px solid var(--border-hairline);
  color: var(--text-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
  backdrop-filter: blur(12px);
}

.action-btn:hover {
  background: var(--accent-purple);
  border-color: var(--accent-purple);
  transform: scale(1.1);
}
</style>
