<template>
  <div class="pin-view" @wheel.prevent="handleWheel" @dblclick="close">
    <img
      v-if="src"
      :src="src"
      class="pin-image"
      :style="{ transform: `scale(${scale})` }"
      draggable="false"
      data-tauri-drag-region
    />
    <div class="pin-toolbar">
      <button class="tool-btn" title="缩小" @click="zoom(-0.1)">−</button>
      <span class="zoom-label">{{ Math.round(scale * 100) }}%</span>
      <button class="tool-btn" title="放大" @click="zoom(0.1)">+</button>
      <button class="tool-btn close" title="取消贴图" @click="close">✕</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const params = new URLSearchParams(window.location.search);
const path = params.get('pin') ?? '';
const pinId = params.get('id') ?? '';
const src = computed(() => (path ? convertFileSrc(path) : ''));

const scale = ref(1);

const zoom = (delta: number) => {
  scale.value = Math.min(5, Math.max(0.2, Number((scale.value + delta).toFixed(2))));
};

const handleWheel = (e: WheelEvent) => {
  zoom(e.deltaY > 0 ? -0.1 : 0.1);
};

const close = async () => {
  if (pinId) {
    await invoke('unpin_image', { id: pinId }).catch(() => undefined);
  }
  await getCurrentWindow().close().catch(() => undefined);
};
</script>

<style scoped>
.pin-view {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: transparent;
  user-select: none;
}

.pin-image {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  transition: transform 0.15s var(--ease-out-expo, ease-out);
  -webkit-user-drag: none;
}

.pin-toolbar {
  position: absolute;
  bottom: 8px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 999px;
  background: rgba(10, 10, 20, 0.75);
  border: 1px solid rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(12px);
  opacity: 0;
  transition: opacity 0.2s ease;
}

.pin-view:hover .pin-toolbar {
  opacity: 1;
}

.tool-btn {
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: none;
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tool-btn:hover {
  background: rgba(168, 85, 247, 0.8);
}

.tool-btn.close:hover {
  background: #ef4444;
}

.zoom-label {
  color: rgba(255, 255, 255, 0.8);
  font-size: 11px;
  min-width: 36px;
  text-align: center;
}
</style>
