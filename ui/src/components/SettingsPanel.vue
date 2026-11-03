<template>
  <div class="settings-panel">
      <section class="section">
        <h3 class="section-title">挂绳</h3>
        <div class="slider-row">
          <div class="slider-label">
            <span>图片大小</span>
            <span class="slider-value">{{ Math.round(draft.card_scale * 100) }}%</span>
          </div>
          <input
            class="size-slider"
            type="range"
            min="20"
            max="130"
            step="1"
            :value="Math.round(draft.card_scale * 100)"
            @input="previewCardScale"
            @change="commitCardScale"
          />
        </div>
        <div class="slider-row">
          <div class="slider-label">
            <span>绳索弧度</span>
            <span class="slider-value">{{ Math.round(draft.rope_sag) }}%</span>
          </div>
          <input
            class="size-slider"
            type="range"
            min="0"
            max="100"
            step="1"
            :value="draft.rope_sag"
            @input="previewRopeSag"
            @change="commitRopeSag"
          />
        </div>
        <div class="swing-row">
          <div class="slider-label">
            <span>摇晃幅度</span>
            <span class="slider-value">{{ draft.card_swing }}°</span>
          </div>
          <div class="swing-presets">
            <button
              v-for="p in swingPresets"
              :key="p.value"
              class="swing-preset"
              :class="{ active: draft.card_swing === p.value }"
              @click="setSwing(p.value)"
            >
              {{ p.label }}
            </button>
          </div>
          <div class="swing-input-row">
            <input
              class="swing-input"
              type="number"
              min="0"
              max="15"
              step="0.5"
              :value="draft.card_swing"
              @input="previewSwing"
              @change="commitSwing"
            />
            <span class="swing-unit">°</span>
          </div>
        </div>
        <ToggleSwitch
          :model-value="draft.rope_always_visible"
          title="绳索常驻显示"
          description="关闭后需点击触发按钮才显示绳索"
          @update:model-value="update('rope_always_visible', $event)"
        />
        <div v-if="!draft.rope_always_visible" class="auto-hide-row">
          <div class="slider-label">
            <span>自动隐藏时间</span>
            <span class="slider-value">{{ draft.rope_auto_hide_seconds }}秒</span>
          </div>
          <div class="swing-input-row">
            <input
              class="swing-input"
              type="number"
              min="1"
              max="60"
              step="1"
              :value="draft.rope_auto_hide_seconds"
              @input="update('rope_auto_hide_seconds', Number(($event.target as HTMLInputElement).value))"
            />
            <span class="swing-unit">秒</span>
          </div>
        </div>
        <div class="animation-style-row">
          <div class="slider-label">
            <span>显示动画风格</span>
          </div>
          <div class="swing-presets">
            <button
              v-for="style in animationStyles"
              :key="style.value"
              class="swing-preset"
              :class="{ active: draft.rope_animation_style === style.value }"
              @click="update('rope_animation_style', style.value)"
            >
              {{ style.label }}
            </button>
          </div>
        </div>
        <div class="slider-row">
          <div class="slider-label">
            <span>动画时长</span>
            <span class="slider-value">{{ draft.rope_animation_duration }}ms</span>
          </div>
          <input
            class="size-slider"
            type="range"
            min="200"
            max="2000"
            step="100"
            :value="draft.rope_animation_duration"
            @input="update('rope_animation_duration', Number(($event.target as HTMLInputElement).value))"
          />
        </div>
        <ToggleSwitch
          :model-value="draft.always_on_top"
          title="窗口置顶"
          description="置物架始终显示在其他窗口之上"
          @update:model-value="update('always_on_top', $event)"
        />
      </section>

      <section class="section">
        <h3 class="section-title">外观</h3>
        <div class="choice-options">
          <button
            v-for="option in themes"
            :key="option.id"
            class="choice-option"
            :class="{ active: draft.theme === option.id }"
            @click="update('theme', option.id)"
          >
            <span class="theme-swatch" :class="`swatch-${option.id}`"></span>
            {{ option.label }}
          </button>
        </div>
      </section>

      <section class="section">
        <h3 class="section-title">收集</h3>
        <ToggleSwitch
          :model-value="draft.collect_images"
          title="自动收集截图"
          description="监听截图文件夹，新截图自动上绳"
          @update:model-value="update('collect_images', $event)"
        />
        <ToggleSwitch
          :model-value="draft.auto_hang"
          title="自动挂上绳子"
          description="新图片直接出现在绳子最右侧"
          @update:model-value="update('auto_hang', $event)"
        />
        <ToggleSwitch
          :model-value="draft.collect_texts"
          title="收集剪贴板文字"
          description="复制文字后自动存入文字信封"
          @update:model-value="update('collect_texts', $event)"
        />
      </section>

      <section class="section">
        <h3 class="section-title">系统</h3>
        <ToggleSwitch
          :model-value="draft.start_on_boot"
          title="开机自动启动"
          description="登录 Windows 后自动在托盘运行"
          @update:model-value="update('start_on_boot', $event)"
        />
      </section>

      <section class="section">
        <h3 class="section-title">截图文件夹</h3>
        <p class="folder-path">{{ draft.screenshots_folder || '未设置' }}</p>
        <GlassButton variant="secondary" :disabled="!draft.screenshots_folder" @click="openFolder">
          打开文件夹
        </GlassButton>
      </section>

      <section class="section">
        <h3 class="section-title">快捷键</h3>
        <div class="shortcut-row">
          <span>截图</span>
          <kbd>{{ captureHotkey }}</kbd>
        </div>
        <div class="shortcut-row">
          <span>显示 / 隐藏</span>
          <kbd>{{ toggleHotkey }}</kbd>
        </div>
      </section>

      <div class="meta">
        <span>SnapCord v{{ version }}</span>
        <span>MIT License</span>
      </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import GlassButton from './GlassButton.vue';
import ToggleSwitch from './ToggleSwitch.vue';
import { api, isTauri } from '../api';
import type { Settings } from '../types';

const props = defineProps<{
  settings: Settings;
}>();

const emit = defineEmits<{
  (e: 'update', settings: Settings): void;
  (e: 'preview-scale', scale: number): void;
  (e: 'preview-rope-sag', percent: number): void;
  (e: 'preview-swing', degrees: number): void;
}>();

const draft = ref<Settings>({ ...props.settings });
const version = ref('0.1.0');

const themes = [
  { id: 'classic', label: '经典紫' },
  { id: 'neon', label: '霓虹青' },
  { id: 'light', label: '浅色' },
];

const previewCardScale = (event: Event) => {
  const percent = Number((event.target as HTMLInputElement).value);
  draft.value = { ...draft.value, card_scale: percent / 100 };
  emit('preview-scale', percent / 100);
};

const commitCardScale = () => {
  update('card_scale', draft.value.card_scale);
};

const previewRopeSag = (event: Event) => {
  const percent = Number((event.target as HTMLInputElement).value);
  draft.value = { ...draft.value, rope_sag: percent };
  emit('preview-rope-sag', percent);
};

const commitRopeSag = () => {
  update('rope_sag', draft.value.rope_sag);
};

const swingPresets = [
  { label: '关闭', value: 0 },
  { label: '小', value: 2 },
  { label: '中', value: 4 },
  { label: '大', value: 7 },
];

const animationStyles = [
  { label: '弧度展开', value: 0 },
  { label: '拉绳抖动', value: 1 },
];

const setSwing = (value: number) => {
  draft.value = { ...draft.value, card_swing: value };
  emit('preview-swing', value);
  update('card_swing', value);
};

const previewSwing = (event: Event) => {
  const value = Number((event.target as HTMLInputElement).value);
  draft.value = { ...draft.value, card_swing: value };
  emit('preview-swing', value);
};

const commitSwing = () => {
  update('card_swing', draft.value.card_swing);
};

watch(
  () => props.settings,
  (value) => {
    draft.value = { ...value };
  },
  { deep: true },
);

const update = <K extends keyof Settings>(key: K, value: Settings[K]) => {
  draft.value = { ...draft.value, [key]: value };
  emit('update', { ...draft.value });
};

const formatHotkey = (modifiers: number, key: number) => {
  const parts: string[] = [];
  if (modifiers & 0x0002) parts.push('Ctrl');
  if (modifiers & 0x0001) parts.push('Alt');
  if (modifiers & 0x0004) parts.push('Shift');
  if (modifiers & 0x0008) parts.push('Win');
  if ((key >= 0x41 && key <= 0x5a) || (key >= 0x30 && key <= 0x39)) {
    parts.push(String.fromCharCode(key));
  } else if (key >= 0x70 && key <= 0x7b) {
    parts.push(`F${key - 0x6f}`);
  } else if (key === 0x20) {
    parts.push('Space');
  }
  return parts.join('+');
};

const captureHotkey = computed(() =>
  formatHotkey(draft.value.capture_hotkey_modifiers, draft.value.capture_hotkey_key),
);
const toggleHotkey = computed(() =>
  formatHotkey(draft.value.toggle_hotkey_modifiers, draft.value.toggle_hotkey_key),
);

const openFolder = async () => {
  if (!draft.value.screenshots_folder) return;
  await api.openInExplorer(draft.value.screenshots_folder).catch(() => undefined);
};

onMounted(async () => {
  if (!isTauri()) return;
  version.value = await api.getAppVersion().catch(() => '0.1.0');
});
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding-bottom: var(--space-4);
  border-bottom: 1px solid var(--border-hairline);
}

.section:last-of-type {
  border-bottom: none;
}

.section-title {
  margin: 0 0 var(--space-1);
  font-size: var(--text-xs);
  font-weight: var(--font-semibold);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--text-muted);
}

.slider-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0 var(--space-1);
}

.slider-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.slider-value {
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.size-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 5px;
  border-radius: 3px;
  background: linear-gradient(90deg, var(--accent-purple), var(--accent-emerald));
  outline: none;
  cursor: pointer;
}

.size-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  border: 2px solid var(--accent-purple);
  box-shadow: 0 0 10px rgba(168, 85, 247, 0.65);
  cursor: pointer;
}

.size-slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  border: 2px solid var(--accent-purple);
  box-shadow: 0 0 10px rgba(168, 85, 247, 0.65);
  cursor: pointer;
}

.swing-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0 var(--space-1);
}

.swing-presets {
  display: flex;
  gap: var(--space-1);
}

.swing-preset {
  flex: 1;
  padding: 6px 8px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-hairline);
  background: var(--bg-glass-light);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.swing-preset:hover {
  color: var(--text-primary);
  border-color: var(--border-glow);
}

.swing-preset.active {
  border-color: var(--accent-purple);
  color: var(--text-primary);
  box-shadow: var(--shadow-glow);
}

.swing-input-row {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.swing-input {
  width: 80px;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-hairline);
  background: var(--bg-glass-light);
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-variant-numeric: tabular-nums;
  outline: none;
  transition: border-color var(--duration-fast) var(--ease-out-expo);
}

.swing-input:focus {
  border-color: var(--accent-purple);
}

.swing-unit {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.auto-hide-row,
.animation-style-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-2) 0 var(--space-1);
}

.choice-options {
  display: flex;
  gap: var(--space-2);
}

.choice-option {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-hairline);
  background: var(--bg-glass-light);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.choice-option:hover {
  color: var(--text-primary);
  border-color: var(--border-glow);
}

.choice-option.active {
  border-color: var(--accent-purple);
  color: var(--text-primary);
  box-shadow: var(--shadow-glow);
}

.theme-swatch {
  width: 14px;
  height: 14px;
  border-radius: var(--radius-full);
  border: 1px solid rgba(255, 255, 255, 0.25);
}

.swatch-classic {
  background: linear-gradient(135deg, #a855f7, #10b981);
}

.swatch-neon {
  background: linear-gradient(135deg, #22d3ee, #34d399);
}

.swatch-light {
  background: linear-gradient(135deg, #7c3aed, #f4f5fb);
}

.folder-path {
  margin: 0;
  font-size: var(--text-xs);
  color: var(--text-secondary);
  word-break: break-all;
  line-height: 1.5;
}

.shortcut-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

kbd {
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-hairline);
  background: var(--bg-glass-light);
  font-size: var(--text-xs);
  color: var(--text-primary);
}

.meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: var(--text-xs);
  color: var(--text-muted);
}
</style>
