<template>
  <div class="settings-shell">
    <div class="settings-frame">
      <header class="settings-header" data-tauri-drag-region>
        <div class="header-main" data-tauri-drag-region>
          <svg
            class="header-icon"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="3"></circle>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.09a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.09a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
          </svg>
          <h1>设置</h1>
        </div>
        <button class="close-btn" title="关闭设置" @click="closeWindow">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </header>

      <div class="settings-body">
        <SettingsPanel
          :settings="settings"
          @update="handleUpdate"
          @preview-scale="events.emitCardScalePreview"
          @preview-rope-sag="events.emitRopeSagPreview"
          @preview-swing="events.emitCardSwingPreview"
        />
        <p v-if="error" class="error-text">{{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import SettingsPanel from './components/SettingsPanel.vue';
import { api, events, isTauri } from './api';
import type { Settings } from './types';

const defaultSettings: Settings = {
  screenshots_folder: '',
  auto_hang: true,
  collect_images: true,
  collect_texts: false,
  capture_hotkey_modifiers: 0x0001 | 0x0002,
  capture_hotkey_key: 0x53,
  toggle_hotkey_modifiers: 0x0001 | 0x0002,
  toggle_hotkey_key: 0x54,
  display_screen: 0,
  theme: 'classic',
  start_on_boot: false,
  auto_update: true,
  card_scale: 1,
  rope_sag: 35,
  card_swing: 4,
  rope_always_visible: false,
  rope_auto_hide_seconds: 5,
  rope_animation_style: 0,
  rope_animation_duration: 1500,
  always_on_top: true,
};

const settings = ref<Settings>({ ...defaultSettings });
const error = ref('');

const applyTheme = (theme: string) => {
  document.documentElement.dataset.theme = theme || 'classic';
};

watch(
  () => settings.value.theme,
  (theme) => applyTheme(theme),
  { immediate: true },
);

const handleUpdate = async (next: Settings) => {
  const previous = settings.value;
  settings.value = next;
  try {
    await api.updateSettings(next);
    error.value = '';
  } catch (err) {
    settings.value = previous;
    error.value = `保存失败: ${err}`;
  }
};

const closeWindow = () => {
  void getCurrentWindow().close().catch(() => undefined);
};

onMounted(async () => {
  if (!isTauri()) return;
  settings.value = await api.getSettings().catch(() => ({ ...defaultSettings }));
});
</script>

<style scoped>
.settings-shell {
  height: 100vh;
  padding: 10px;
  box-sizing: border-box;
}

.settings-frame {
  height: 100%;
  display: flex;
  flex-direction: column;
  border-radius: 18px;
  overflow: hidden;
  background: linear-gradient(180deg, rgba(18, 18, 26, 0.94) 0%, rgba(8, 8, 13, 0.96) 100%);
  border: 1px solid var(--border-hairline);
  box-shadow:
    0 28px 70px rgba(0, 0, 0, 0.72),
    0 0 32px rgba(168, 85, 247, 0.16);
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 14px 10px 20px;
  flex-shrink: 0;
  cursor: grab;
}

.header-main {
  display: flex;
  align-items: center;
  gap: 10px;
  pointer-events: none;
}

.header-icon {
  color: var(--accent-purple);
}

h1 {
  margin: 0;
  font-size: 16px;
  font-weight: var(--font-semibold);
  color: var(--text-primary);
}

.close-btn {
  width: 30px;
  height: 30px;
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

.settings-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: 2px 20px 20px;
  scrollbar-width: thin;
  scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
}

.error-text {
  margin-top: var(--space-3);
  font-size: var(--text-xs);
  color: #f87171;
}
</style>

<style>
html.settings-window,
html.settings-window body,
html.settings-window #app {
  height: 100%;
  background: transparent !important;
}
</style>
