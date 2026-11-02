<template>
  <div class="panel-header">
    <div class="header-left">
      <slot name="icon"></slot>
      <h2 class="panel-title">
        <slot name="title"></slot>
      </h2>
      <span v-if="badge" class="badge">{{ badge }}</span>
    </div>
    <button v-if="closable" class="close-btn" @click="$emit('close')">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  badge?: string | number;
  closable?: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
}>();
</script>

<style scoped>
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.panel-title {
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
  color: var(--text-primary);
  margin: 0;
}

.badge {
  font-size: var(--text-xs);
  color: var(--text-muted);
  background: var(--bg-glass-light);
  padding: 2px 8px;
  border-radius: var(--radius-full);
}

.close-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-full);
  background: transparent;
  border: 1px solid var(--border-hairline);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.close-btn:hover {
  background: var(--bg-glass-light);
  color: var(--text-primary);
}
</style>
