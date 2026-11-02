<template>
  <GlassPanel :is-visible="isVisible" position="right" @close="$emit('close')">
    <template #default>
      <PanelHeader :badge="images.length" closable @close="$emit('close')">
        <template #icon>
          <svg class="header-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"></path>
            <line x1="3" y1="6" x2="21" y2="6"></line>
            <path d="M16 10a4 4 0 0 1-8 0"></path>
          </svg>
        </template>
        <template #title>图片篮子</template>
      </PanelHeader>

      <div class="image-grid">
        <DoubleBezelCard
          v-for="image in images"
          :key="image.id"
          class="grid-item"
          :class="{ 'is-selected': selectedIds.has(image.id) }"
          @click="toggleSelect(image.id)"
        >
          <img :src="image.thumbnail" :alt="image.id" class="grid-image" />
          <div class="checkbox" v-show="isHovered || selectedIds.has(image.id)">
            <svg v-if="selectedIds.has(image.id)" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          </div>
        </DoubleBezelCard>
      </div>

      <div class="panel-footer">
        <GlassButton variant="secondary" @click="clearSelection">
          取消选择
        </GlassButton>
        <GlassButton variant="primary" :disabled="selectedIds.size === 0" @click="hangSelected">
          挂上绳子 ({{ selectedIds.size }})
        </GlassButton>
      </div>
    </template>
  </GlassPanel>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import GlassPanel from './GlassPanel.vue';
import PanelHeader from './PanelHeader.vue';
import DoubleBezelCard from './DoubleBezelCard.vue';
import GlassButton from './GlassButton.vue';

interface Image {
  id: string;
  path: string;
  thumbnail: string;
  timestamp: Date;
}

defineProps<{
  images: Image[];
  isVisible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'hang', ids: string[]): void;
}>();

const selectedIds = ref(new Set<string>());
const isHovered = ref(false);

const toggleSelect = (id: string) => {
  if (selectedIds.value.has(id)) {
    selectedIds.value.delete(id);
  } else {
    selectedIds.value.add(id);
  }
};

const clearSelection = () => {
  selectedIds.value.clear();
};

const hangSelected = () => {
  emit('hang', Array.from(selectedIds.value));
  selectedIds.value.clear();
};
</script>

<style scoped>
.image-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-3);
  overflow-y: auto;
  flex: 1;
  padding-right: var(--space-2);
}

.grid-item {
  aspect-ratio: 4/3;
  cursor: pointer;
  position: relative;
}

.grid-item.is-selected {
  box-shadow: 0 0 0 2px var(--accent-purple);
}

.grid-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: calc(var(--radius-3xl) - 10px);
}

.checkbox {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: var(--radius-full);
  background: var(--accent-purple);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: var(--shadow-sm);
}

.panel-footer {
  display: flex;
  gap: var(--space-3);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-hairline);
}

.panel-footer :deep(.glass-button) {
  flex: 1;
}

.header-icon {
  color: var(--accent-purple);
}
</style>
