<template>
  <div class="toggle-row" :class="{ 'is-disabled': disabled }">
    <div class="toggle-text">
      <span class="toggle-title">{{ title }}</span>
      <span v-if="description" class="toggle-desc">{{ description }}</span>
    </div>
    <button
      type="button"
      role="switch"
      class="toggle"
      :class="{ 'is-on': modelValue }"
      :aria-checked="modelValue"
      :disabled="disabled"
      @click="$emit('update:modelValue', !modelValue)"
    >
      <span class="knob"></span>
    </button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  modelValue: boolean;
  title: string;
  description?: string;
  disabled?: boolean;
}>();

defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();
</script>

<style scoped>
.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-3) 0;
}

.toggle-row.is-disabled {
  opacity: 0.5;
}

.toggle-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toggle-title {
  font-size: var(--text-sm);
  color: var(--text-primary);
}

.toggle-desc {
  font-size: var(--text-xs);
  color: var(--text-muted);
}

.toggle {
  flex-shrink: 0;
  width: 40px;
  height: 22px;
  padding: 2px;
  border-radius: var(--radius-full);
  border: 1px solid var(--border-hairline);
  background: var(--bg-glass-light);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.toggle.is-on {
  background: var(--gradient-primary);
  border-color: transparent;
}

.knob {
  display: block;
  width: 16px;
  height: 16px;
  border-radius: var(--radius-full);
  background: #fff;
  box-shadow: var(--shadow-sm);
  transition: transform var(--duration-fast) var(--ease-out-expo);
}

.toggle.is-on .knob {
  transform: translateX(18px);
}
</style>
