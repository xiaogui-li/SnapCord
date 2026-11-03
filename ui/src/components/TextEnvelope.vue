<template>
  <GlassPanel :is-visible="isVisible" position="right" @close="$emit('close')">
    <template #default>
      <PanelHeader closable @close="$emit('close')">
        <template #icon>
          <svg class="header-icon" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
            <polyline points="22,6 12,13 2,6"></polyline>
          </svg>
        </template>
        <template #title>文字信封</template>
      </PanelHeader>

      <div class="search-box">
        <svg class="search-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          v-model="searchQuery"
          placeholder="搜索文字..."
          class="search-input"
        />
      </div>

      <div class="text-list">
        <div
          v-for="item in filteredItems"
          :key="item.id"
          class="text-item"
          @click="copyText(item.content)"
        >
          <p class="text-preview">{{ item.content }}</p>
          <div class="item-meta">
            <span class="timestamp">{{ formatTime(item.timestamp) }}</span>
            <GlassButton variant="ghost" icon-only @click.stop="copyText(item.content)">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
            </GlassButton>
          </div>
        </div>
      </div>

      <div class="panel-footer">
        <GlassButton variant="secondary" @click="clearAll">
          清空
        </GlassButton>
      </div>
    </template>
  </GlassPanel>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import GlassPanel from './GlassPanel.vue';
import PanelHeader from './PanelHeader.vue';
import GlassButton from './GlassButton.vue';
import type { TextItem } from '../types';
import { api, isTauri } from '../api';

const props = defineProps<{
  items: TextItem[];
  isVisible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'clear'): void;
}>();

const searchQuery = ref('');

const filteredItems = computed(() => {
  if (!searchQuery.value) return props.items;
  const query = searchQuery.value.toLowerCase();
  return props.items.filter(item =>
    item.content.toLowerCase().includes(query)
  );
});

const copyText = async (text: string) => {
  try {
    if (isTauri()) {
      await api.copyText(text);
    } else {
      await navigator.clipboard.writeText(text);
    }
  } catch (err) {
    console.error('Failed to copy:', err);
  }
};

const formatTime = (timestamp: number) => {
  const diff = Date.now() - timestamp * 1000;
  const minutes = Math.floor(diff / 60000);

  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes}分钟前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}小时前`;
  return new Date(timestamp * 1000).toLocaleDateString('zh-CN');
};

const clearAll = () => {
  emit('clear');
};
</script>

<style scoped>
.search-box {
  position: relative;
}

.search-icon {
  position: absolute;
  left: var(--space-4);
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-muted);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: var(--space-3) var(--space-4) var(--space-3) 40px;
  background: var(--bg-glass-light);
  border: 1px solid var(--border-hairline);
  border-radius: var(--radius-full);
  color: var(--text-primary);
  font-size: var(--text-sm);
  outline: none;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.search-input:focus {
  border-color: var(--accent-purple);
  box-shadow: 0 0 0 3px rgba(168, 85, 247, 0.1);
}

.search-input::placeholder {
  color: var(--text-muted);
}

.text-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding-right: var(--space-2);
}

.text-item {
  padding: var(--space-4);
  background: var(--bg-glass-light);
  border: 1px solid var(--border-hairline);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.text-item:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.15);
  transform: translateY(-2px);
}

.text-preview {
  font-size: var(--text-sm);
  color: var(--text-primary);
  line-height: 1.5;
  margin: 0 0 var(--space-2) 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.item-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.timestamp {
  font-size: var(--text-xs);
  color: var(--text-muted);
}

.panel-footer {
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-hairline);
}

.header-icon {
  color: var(--accent-emerald);
}
</style>
