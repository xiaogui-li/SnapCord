<template>
  <div class="rope-container" @mouseenter="isExpanded = true" @mouseleave="isExpanded = false">
    <!-- Rope wire -->
    <div class="rope-wire"></div>

    <!-- Image cards -->
    <div class="rope-cards">
      <ImageCard
        v-for="(image, index) in images"
        :key="image.id"
        :image="image"
        :index="index"
        :is-expanded="isExpanded"
        @click="handleImageClick"
        @drag-start="handleDragStart"
        @pin="handlePin"
      />
    </div>

    <!-- Empty state -->
    <div v-if="images.length === 0" class="empty-state">
      <p class="empty-text">拖拽截图到此处</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import ImageCard from './ImageCard.vue';

interface Image {
  id: string;
  path: string;
  thumbnail: string;
  timestamp: Date;
}

defineProps<{
  images: Image[];
}>();

const emit = defineEmits<{
  (e: 'image-click', image: Image): void;
  (e: 'drag-start', image: Image): void;
  (e: 'pin', image: Image): void;
}>();

const isExpanded = ref(false);

const handleImageClick = (image: Image) => {
  emit('image-click', image);
};

const handleDragStart = (image: Image) => {
  emit('drag-start', image);
};

const handlePin = (image: Image) => {
  emit('pin', image);
};
</script>

<style scoped>
.rope-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 120px;
  z-index: var(--z-sticky);
  transition: height var(--duration-normal) var(--ease-out-expo);
}

.rope-container:hover {
  height: 160px;
}

.rope-wire {
  position: absolute;
  top: 20px;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, 
    transparent 0%, 
    var(--accent-purple) 20%, 
    var(--accent-emerald) 80%, 
    transparent 100%
  );
  opacity: 0.6;
  box-shadow: 0 0 12px rgba(168, 85, 247, 0.3);
}

.rope-cards {
  position: absolute;
  top: 24px;
  left: var(--space-8);
  right: var(--space-8);
  display: flex;
  gap: var(--space-4);
  overflow-x: auto;
  overflow-y: visible;
  padding: var(--space-2) 0;
  scrollbar-width: none;
}

.rope-cards::-webkit-scrollbar {
  display: none;
}

.empty-state {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
}

.empty-text {
  color: var(--text-muted);
  font-size: var(--text-sm);
  font-weight: var(--font-light);
}
</style>
