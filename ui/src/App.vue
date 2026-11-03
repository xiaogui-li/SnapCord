<template>
  <div class="app-container">
    <!-- Trigger button -->
    <RopeTrigger v-if="!settings.rope_always_visible" @toggle="toggleRope" />

    <!-- Main rope bar -->
    <Rope
      :images="ropeImages"
      :card-scale="effectiveCardScale"
      :rope-sag="effectiveRopeSag"
      :card-swing="effectiveCardSwing"
      :is-visible="isRopeVisible"
      :animation-style="settings.rope_animation_style"
      :animation-duration="settings.rope_animation_duration"
      @image-click="handleImageClick"
      @drag-start="handleDragStart"
      @pin="handlePin"
      @copy="handleImageClick"
      @basket="handleMoveToBasket"
      @edit="handleEdit"
    />

    <!-- Basket panel -->
    <BasketPanel
      :images="basketImages"
      :is-visible="showBasket"
      @close="showBasket = false"
      @hang="handleHangImages"
    />

    <!-- Text envelope -->
    <TextEnvelope
      :items="texts"
      :is-visible="showTextEnvelope"
      @close="showTextEnvelope = false"
      @clear="handleClearTexts"
    />

    <!-- Status toast -->
    <Transition name="toast">
      <div v-if="toast.message" class="toast" :class="toast.type">
        {{ toast.message }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Rope from './components/Rope.vue';
import RopeTrigger from './components/RopeTrigger.vue';
import BasketPanel from './components/BasketPanel.vue';
import TextEnvelope from './components/TextEnvelope.vue';
import { api, events, isTauri } from './api';
import type { ImageItem, Settings, TextItem } from './types';

const images = ref<ImageItem[]>([]);
const texts = ref<TextItem[]>([]);

const ropeImages = computed(() => images.value.filter((img) => img.source !== 'basket'));
const basketImages = computed(() => images.value.filter((img) => img.source === 'basket'));

const showBasket = ref(false);
const showTextEnvelope = ref(false);

const overlayExpanded = ref(false);

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
// 滑块拖动过程中的实时预览，松手保存后回到 settings 里的值。
const cardScalePreview = ref<number | null>(null);
const effectiveCardScale = computed(() => cardScalePreview.value ?? settings.value.card_scale);
const ropeSagPreview = ref<number | null>(null);
const effectiveRopeSag = computed(() => ropeSagPreview.value ?? settings.value.rope_sag);
const cardSwingPreview = ref<number | null>(null);
const effectiveCardSwing = computed(() => cardSwingPreview.value ?? settings.value.card_swing);

// 绳索显示/隐藏控制
const isRopeVisible = ref(settings.value.rope_always_visible);
let hideTimer: number | undefined;

const toggleRope = () => {
  if (isRopeVisible.value) {
    hideRope();
  } else {
    showRope();
  }
};

const showRope = () => {
  isRopeVisible.value = true;
  if (hideTimer) clearTimeout(hideTimer);
  if (!settings.value.rope_always_visible && settings.value.rope_auto_hide_seconds > 0) {
    hideTimer = window.setTimeout(() => {
      hideRope();
    }, settings.value.rope_auto_hide_seconds * 1000);
  }
};

const hideRope = () => {
  isRopeVisible.value = false;
  if (hideTimer) clearTimeout(hideTimer);
};

watch(
  () => settings.value.rope_always_visible,
  (alwaysVisible) => {
    if (alwaysVisible) {
      showRope();
    } else {
      hideRope();
    }
  },
);

watch(
  () => settings.value.theme,
  (theme) => {
    document.documentElement.dataset.theme = theme || 'classic';
  },
  { immediate: true },
);

const toast = reactive({
  message: '',
  type: 'success' as 'success' | 'error',
});

watch(
  () => settings.value.always_on_top,
  async (value) => {
    if (!isTauri()) return;
    await getCurrentWindow().setAlwaysOnTop(value).catch(() => undefined);
  },
  { immediate: true },
);

let unlistenImage: UnlistenFn | undefined;
let unlistenText: UnlistenFn | undefined;
let unlistenPanel: UnlistenFn | undefined;
let unlistenSettings: UnlistenFn | undefined;
let unlistenScalePreview: UnlistenFn | undefined;
let unlistenSagPreview: UnlistenFn | undefined;
let unlistenSwingPreview: UnlistenFn | undefined;

const showToast = (message: string, type: 'success' | 'error' = 'success') => {
  toast.message = message;
  toast.type = type;
  setTimeout(() => {
    toast.message = '';
  }, 2000);
};

const refresh = async () => {
  const [loadedImages, loadedTexts] = await Promise.all([api.getImages(), api.getTexts()]);
  images.value = loadedImages;
  texts.value = loadedTexts;
};

const handleImageClick = async (image: ImageItem) => {
  try {
    await api.copyImage(image.path);
    showToast('已复制到剪贴板');
  } catch (err) {
    showToast(`复制失败: ${err}`, 'error');
  }
};

const handleDragStart = async (image: ImageItem) => {
  try {
    await api.startFileDrag(image.id);
  } catch (err) {
    showToast(`拖拽失败: ${err}`, 'error');
  }
};

const handleMoveToBasket = async (image: ImageItem) => {
  try {
    await api.moveToBasket(image.id);
    showToast('已收进篮子');
    await refresh();
  } catch (err) {
    showToast(`操作失败: ${err}`, 'error');
  }
};

const handlePin = async (image: ImageItem) => {
  try {
    await api.pinImage(image.id);
    showToast('已钉在桌面');
    await refresh();
  } catch (err) {
    showToast(`贴图失败: ${err}`, 'error');
  }
};

const handleEdit = async (image: ImageItem) => {
  try {
    await api.openImageEditor(image.id);
  } catch (err) {
    showToast(`打开编辑器失败: ${err}`, 'error');
  }
};

const openPanel = (panel: 'basket' | 'texts') => {
  showBasket.value = panel === 'basket';
  showTextEnvelope.value = panel === 'texts';
};

const syncOverlayHeight = async (expanded: boolean) => {
  if (expanded === overlayExpanded.value) return;
  overlayExpanded.value = expanded;
  if (isTauri()) {
    await api.setOverlayExpanded(expanded).catch(() => undefined);
  }
};

watch(
  [showBasket, showTextEnvelope],
  ([basket, envelope]) => {
    void syncOverlayHeight(basket || envelope);
  },
  { immediate: true },
);

// 透明区域点击穿透依赖后端拿到的可交互矩形，这里定期上报最新布局。
let hitTimer: number | undefined;
let lastHitPayload = '';

const reportHitRegions = () => {
  if (!isTauri()) return;
  const regions: number[][] = [];
  document
    .querySelectorAll<HTMLElement>('[data-hit], .glass-panel.is-visible')
    .forEach((el) => {
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0) return;
      regions.push([rect.left, rect.top, rect.right, rect.bottom]);
    });
  const payload = JSON.stringify(regions);
  if (payload === lastHitPayload) return;
  lastHitPayload = payload;
  void api.setHitRegions(regions).catch(() => {
    lastHitPayload = '';
  });
};

const handleHangImages = async (ids: string[]) => {
  try {
    await Promise.all(ids.map((id) => api.moveToRope(id)));
    showToast(`已挂上 ${ids.length} 张图片`);
    await refresh();
  } catch (err) {
    showToast(`挂图失败: ${err}`, 'error');
  }
};

const handleClearTexts = async () => {
  try {
    await api.clearTexts();
    await refresh();
    showToast('已清空文字');
  } catch (err) {
    showToast(`清空失败: ${err}`, 'error');
  }
};

onMounted(async () => {
  if (!isTauri()) {
    console.info('Running outside Tauri, IPC disabled');
    return;
  }
  // setup 阶段拿不到显示器信息，等 webview 就绪后再铺满屏幕顶边。
  await api
    .setOverlayExpanded(showBasket.value || showTextEnvelope.value)
    .catch(() => undefined);
  reportHitRegions();
  hitTimer = window.setInterval(reportHitRegions, 200);
  await refresh();
  settings.value = await api.getSettings().catch(() => ({ ...defaultSettings }));
  unlistenPanel = await events.onOpenPanel(openPanel);
  unlistenSettings = await events.onSettingsUpdated((next) => {
    settings.value = next;
    cardScalePreview.value = null;
    ropeSagPreview.value = null;
    cardSwingPreview.value = null;
  });
  unlistenScalePreview = await events.onCardScalePreview((scale) => {
    cardScalePreview.value = scale;
  });
  unlistenSagPreview = await events.onRopeSagPreview((percent) => {
    ropeSagPreview.value = percent;
  });
  unlistenSwingPreview = await events.onCardSwingPreview((degrees) => {
    cardSwingPreview.value = degrees;
  });
  unlistenImage = await events.onImageCollected((image) => {
    const index = images.value.findIndex((existing) => existing.id === image.id);
    if (index >= 0) {
      images.value[index] = image;
    } else {
      images.value.push(image);
    }
    showToast('收集到新截图');
  });
  unlistenText = await events.onTextCollected((item) => {
    texts.value.push(item);
  });
});

onUnmounted(() => {
  unlistenImage?.();
  unlistenText?.();
  unlistenPanel?.();
  unlistenSettings?.();
  unlistenScalePreview?.();
  unlistenSagPreview?.();
  unlistenSwingPreview?.();
  if (hitTimer) window.clearInterval(hitTimer);
  if (hideTimer) clearTimeout(hideTimer);
});
</script>

<style scoped>
.app-container {
  height: 100vh;
  position: relative;
  overflow: hidden;
  background: transparent;
}

.toast {
  position: fixed;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  padding: var(--space-2) var(--space-5);
  background: var(--bg-glass);
  border: 1px solid var(--border-hairline);
  border-radius: var(--radius-full);
  color: var(--text-primary);
  font-size: var(--text-sm);
  backdrop-filter: blur(12px);
  z-index: var(--z-tooltip);
  box-shadow: var(--shadow-md);
}

.toast.success {
  border-color: var(--accent-emerald);
}

.toast.error {
  border-color: #EF4444;
}

.toast-enter-active,
.toast-leave-active {
  transition: all var(--duration-normal) var(--ease-out-expo);
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(20px);
}
</style>

<style>
html.overlay-window,
html.overlay-window body,
html.overlay-window #app {
  background: transparent !important;
}
</style>
