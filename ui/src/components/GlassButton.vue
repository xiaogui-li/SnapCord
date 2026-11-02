<template>
  <button
    class="glass-button"
    :class="{
      'is-primary': variant === 'primary',
      'is-secondary': variant === 'secondary',
      'is-ghost': variant === 'ghost',
      'is-icon': iconOnly,
      'is-disabled': disabled,
    }"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot></slot>
    <span v-if="trailingIcon" class="trailing-icon">
      <slot name="trailing"></slot>
    </span>
  </button>
</template>

<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'ghost';
  iconOnly?: boolean;
  disabled?: boolean;
  trailingIcon?: boolean;
}>();

defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();
</script>

<style scoped>
.glass-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  border-radius: var(--radius-full);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
  border: none;
  outline: none;
  position: relative;
}

.glass-button:active:not(:disabled) {
  transform: scale(0.98);
}

/* Primary variant */
.glass-button.is-primary {
  background: var(--gradient-primary);
  color: white;
  padding: var(--space-3) var(--space-6);
  box-shadow: 0 4px 16px rgba(168, 85, 247, 0.3);
}

.glass-button.is-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(168, 85, 247, 0.4);
}

/* Secondary variant */
.glass-button.is-secondary {
  background: var(--bg-glass-light);
  color: var(--text-secondary);
  border: 1px solid var(--border-hairline);
  padding: var(--space-3) var(--space-6);
}

.glass-button.is-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-primary);
}

/* Ghost variant */
.glass-button.is-ghost {
  background: transparent;
  color: var(--text-secondary);
  padding: var(--space-2);
}

.glass-button.is-ghost:hover:not(:disabled) {
  background: var(--bg-glass-light);
  color: var(--text-primary);
}

/* Icon only */
.glass-button.is-icon {
  width: 32px;
  height: 32px;
  padding: 0;
}

/* Disabled */
.glass-button.is-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Trailing icon */
.trailing-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: var(--radius-full);
  background: rgba(0, 0, 0, 0.2);
  margin-left: var(--space-2);
}

.glass-button.is-primary .trailing-icon {
  background: rgba(255, 255, 255, 0.2);
}

.glass-button.is-primary:hover:not(:disabled) .trailing-icon {
  transform: translateX(2px) translateY(-1px);
}
</style>
