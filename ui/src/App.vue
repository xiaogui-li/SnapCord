<template>
  <div class="app-container">
    <!-- Main rope bar -->
    <Rope
      :images="images"
      @image-click="handleImageClick"
      @drag-start="handleDragStart"
      @pin="handlePin"
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
      :items="textItems"
      :is-visible="showTextEnvelope"
      @close="showTextEnvelope = false"
    />

    <!-- Floating action buttons -->
    <div class="fab-container">
      <button class="fab" @click="toggleBasket" :class="{ active: showBasket }">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"></path>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <path d="M16 10a4 4 0 0 1-8 0"></path>
        </svg>
        <span v-if="basketImages.length > 0" class="fab-badge">{{ basketImages.length }}</span>
      </button>

      <button class="fab" @click="toggleTextEnvelope" :class="{ active: showTextEnvelope }">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"></path>
          <polyline points="22,6 12,13 2,6"></polyline>
        </svg>
        <span v-if="textItems.length > 0" class="fab-badge">{{ textItems.length }}</span>
      </button>
    </div>

    <!-- Status toast -->
    <Transition name="toast">
      <div v-if="toast.message" class="toast" :class="toast.type">
        {{ toast.message }}
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import Rope from './components/Rope.vue';
import BasketPanel from './components/BasketPanel.vue';
import TextEnvelope from './components/TextEnvelope.vue';

interface Image {
  id: string;
  path: string;
  thumbnail: string;
  timestamp: Date;
}

interface TextItem {
  id: string;
  content: string;
  timestamp: Date;
}

// Mock data for development
const images = ref<Image[]>([
  {
    id: '1',
    path: '/mock/screenshot1.png',
    thumbnail: 'https://picsum.photos/seed/snap1/200/150',
    timestamp: new Date(),
  },
  {
    id: '2',
    path: '/mock/screenshot2.png',
    thumbnail: 'https://picsum.photos/seed/snap2/200/150',
    timestamp: new Date(),
  },
  {
    id: '3',
    path: '/mock/screenshot3.png',
    thumbnail: 'https://picsum.photos/seed/snap3/200/150',
    timestamp: new Date(),
  },
]);

const basketImages = ref<Image[]>([
  {
    id: '4',
    path: '/mock/basket1.png',
    thumbnail: 'https://picsum.photos/seed/basket1/200/150',
    timestamp: new Date(),
  },
  {
    id: '5',
    path: '/mock/basket2.png',
    thumbnail: 'https://picsum.photos/seed/basket2/200/150',
    timestamp: new Date(),
  },
]);

const textItems = ref<TextItem[]>([
  {
    id: '1',
    content: '这是一段示例文字，来自剪贴板收集。',
    timestamp: new Date(),
  },
  {
    id: '2',
    content: 'SnapCord 是一个截图暂存工具，支持拖拽、复制和桌面贴图。',
    timestamp: new Date(Date.now() - 3600000),
  },
]);

const showBasket = ref(false);
const showTextEnvelope = ref(false);

const toast = reactive({
  message: '',
  type: 'success',
});

const showToast = (message: string, type: 'success' | 'error' = 'success') => {
  toast.message = message;
  toast.type = type;
  setTimeout(() => {
    toast.message = '';
  }, 2000);
};

const handleImageClick = (_image: Image) => {
  // TODO: Copy image to clipboard via Tauri
  showToast('已复制到剪贴板');
};

const handleDragStart = (image: Image) => {
  console.log('Drag start:', image.path);
};

const handlePin = (_image: Image) => {
  // TODO: Create pinned window via Tauri
  showToast('已钉在桌面');
};

const toggleBasket = () => {
  showBasket.value = !showBasket.value;
  if (showBasket.value) showTextEnvelope.value = false;
};

const toggleTextEnvelope = () => {
  showTextEnvelope.value = !showTextEnvelope.value;
  if (showTextEnvelope.value) showBasket.value = false;
};

const handleHangImages = (ids: string[]) => {
  // TODO: Move images from basket to rope via Tauri
  showToast(`已挂上 ${ids.length} 张图片`);
};
</script>

<style scoped>
.app-container {
  min-height: 100vh;
  background: var(--bg-void);
  position: relative;
  overflow: hidden;
}

.fab-container {
  position: fixed;
  right: var(--space-8);
  bottom: var(--space-8);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  z-index: var(--z-sticky);
}

.fab {
  width: 56px;
  height: 56px;
  border-radius: var(--radius-full);
  background: var(--bg-glass);
  border: 1px solid var(--border-hairline);
  color: var(--text-primary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all var(--duration-normal) var(--ease-out-expo);
  backdrop-filter: blur(12px);
}

.fab:hover {
  transform: scale(1.1);
  background: rgba(255, 255, 255, 0.1);
}

.fab.active {
  background: var(--gradient-primary);
  border-color: transparent;
  box-shadow: var(--shadow-glow);
}

.fab-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  background: var(--accent-purple);
  color: white;
  font-size: 11px;
  font-weight: var(--font-semibold);
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--shadow-sm);
}

.toast {
  position: fixed;
  bottom: var(--space-8);
  left: 50%;
  transform: translateX(-50%);
  padding: var(--space-3) var(--space-6);
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
