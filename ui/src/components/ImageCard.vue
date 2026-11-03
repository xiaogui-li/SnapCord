<template>
  <div
    class="image-card"
    :class="{ 'is-hovered': isHovered, 'is-dragging': isDragging, 'is-compact': isCompact }"
    :style="cardStyle"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
    @click="handleClick"
    @mousedown="handleMouseDown"
    @dragstart.prevent
  >
    <div class="card-hanger"><span class="clip"></span></div>

    <div class="card-frame">
      <div class="card-body">
        <div class="image-wrapper">
          <img :src="image.thumbnail" :alt="image.id" class="preview-image" draggable="false" />
          <span class="image-sheen"></span>

          <div class="actions" v-show="isHovered">
            <button class="action-btn" @click.stop="handleEdit" title="编辑">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M6 20h12a2 2 0 0 0 2-2V9l-5-5H6a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2z"></path>
                <path d="M14 4v5h5"></path>
                <path d="M9 14l2 2 4-4"></path>
              </svg>
            </button>
            <button class="action-btn" @click.stop="handleCopy" title="复制">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
              </svg>
            </button>
            <button class="action-btn" @click.stop="handlePin" title="钉在桌面">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2L12 22"></path>
                <path d="M17 7L12 2L7 7"></path>
              </svg>
            </button>
            <button class="action-btn" @click.stop="handleBasket" title="收进篮子">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z"></path>
                <line x1="3" y1="6" x2="21" y2="6"></line>
                <path d="M16 10a4 4 0 0 1-8 0"></path>
              </svg>
            </button>
          </div>
        </div>

        <div class="caption">
          <div class="caption-text">
            <span class="caption-title">{{ captionTitle }}</span>
            <span class="caption-time">{{ relativeTime }}</span>
          </div>
          <button class="open-btn" @click.stop="handleClick" title="打开图片">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M15 3h6v6"></path>
              <path d="M10 14 21 3"></path>
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue';
import type { ImageItem } from '../types';
import type { ImageSource } from '../types';

const props = defineProps<{
  image: ImageItem;
  index: number;
  isExpanded: boolean;
  cardScale?: number;
  cardSwing?: number;
}>();

// 缩到小尺寸时只保留缩略图，说明文字和悬停按钮会挤成一团。
const isCompact = computed(() => (props.cardScale ?? 1) < 0.6);

const emit = defineEmits<{
  (e: 'click', image: ImageItem): void;
  (e: 'drag-start', image: ImageItem): void;
  (e: 'pin', image: ImageItem): void;
  (e: 'copy', image: ImageItem): void;
  (e: 'basket', image: ImageItem): void;
  (e: 'edit', image: ImageItem): void;
}>();

const isHovered = ref(false);
const isDragging = ref(false);

// 每张卡片独立相位与周期，避免整齐划一的机械感。
const swingDelay = ref(Math.random() * -3);
const swingDuration = ref(2.5 + Math.random() * 1.5);

const GLOW_COLORS = ['168, 85, 247', '56, 189, 248', '16, 185, 129'];

const cardStyle = computed(() => ({
  '--index': props.index,
  '--glow-rgb': GLOW_COLORS[props.index % GLOW_COLORS.length],
  '--swing-angle': `${props.cardSwing ?? 0}deg`,
  '--swing-delay': `${swingDelay.value}s`,
  '--swing-duration': `${swingDuration.value}s`,
}));

const SOURCE_LABELS: Record<ImageSource, string> = {
  screenshot: '截图',
  clipboard: '剪贴板',
  manual: '导入',
  basket: '篮子',
};

const captionTitle = computed(() => {
  const date = new Date(props.image.timestamp * 1000);
  const clock = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
  return `${SOURCE_LABELS[props.image.source] ?? '图片'} · ${clock}`;
});

const relativeTime = computed(() => {
  const seconds = Math.max(0, Math.floor(Date.now() / 1000 - props.image.timestamp));
  if (seconds < 60) return '刚刚';
  if (seconds < 3600) return `${Math.floor(seconds / 60)} 分钟前`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} 小时前`;
  if (seconds < 86400 * 7) return `${Math.floor(seconds / 86400)} 天前`;
  const date = new Date(props.image.timestamp * 1000);
  return `${date.getMonth() + 1}月${date.getDate()}日`;
});

const handleClick = () => {
  emit('click', props.image);
};

let dragOrigin: { x: number; y: number } | null = null;

const clearDragListeners = () => {
  dragOrigin = null;
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('mouseup', clearDragListeners);
};

const handleMouseMove = (event: MouseEvent) => {
  if (!dragOrigin) return;
  if (Math.hypot(event.clientX - dragOrigin.x, event.clientY - dragOrigin.y) < 6) return;
  clearDragListeners();
  isDragging.value = true;
  emit('drag-start', props.image);
  setTimeout(() => {
    isDragging.value = false;
  }, 200);
};

const handleMouseDown = (event: MouseEvent) => {
  if (event.button !== 0) return;
  event.preventDefault();
  dragOrigin = { x: event.clientX, y: event.clientY };
  window.addEventListener('mousemove', handleMouseMove);
  window.addEventListener('mouseup', clearDragListeners);
};

const handleCopy = () => {
  emit('copy', props.image);
};

const handleEdit = () => {
  emit('edit', props.image);
};

const handleBasket = () => {
  emit('basket', props.image);
};

onBeforeUnmount(clearDragListeners);

const handlePin = () => {
  emit('pin', props.image);
};
</script>

<style scoped>
.image-card {
  width: calc(240px * var(--card-scale, 1));
  flex-shrink: 0;
  margin-top: 14px;
  position: relative;
  cursor: pointer;
  user-select: none;
  animation:
    cardEnter var(--duration-slower) var(--ease-out-expo) backwards,
    cardSwing var(--swing-duration, 3s) ease-in-out infinite;
  animation-delay:
    calc(var(--index, 0) * 60ms),
    var(--swing-delay, 0s);
  transform-origin: 50% 0;
  transition: transform 0.35s var(--ease-out-expo);
}

@keyframes cardEnter {
  from {
    opacity: 0;
    transform: translateY(-26px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes cardSwing {
  0%, 100% {
    transform: rotate(calc(-1 * var(--swing-angle, 0deg)));
  }
  50% {
    transform: rotate(var(--swing-angle, 0deg));
  }
}

.image-card:hover {
  transform: translateY(-10px);
}

.image-card.is-dragging {
  opacity: 0.55;
  transform: scale(0.97);
}

/* 挂点相对绳子是固定的：缩放卡片时只缩小夹子，夹子与绳子的接触位置不能动。 */
.card-hanger {
  position: absolute;
  top: -30px;
  left: 50%;
  width: 36px;
  height: 48px;
  transform: translateX(-50%);
  z-index: 3;
  pointer-events: none;
}

.clip {
  position: absolute;
  inset: 0;
  transform: scale(clamp(0.7, var(--card-scale, 1), 1));
  /* 以吊环中心为缩放原点，保证缩小后吊环仍卡在绳子上。 */
  transform-origin: 50% 19%;
}

/* 金属夹的吊环 */
.clip::before {
  content: '';
  position: absolute;
  left: 50%;
  top: 0;
  width: 20px;
  height: 18px;
  transform: translateX(-50%);
  border: 2.6px solid #d6dbe4;
  border-bottom: none;
  border-radius: 11px 11px 0 0;
  box-shadow:
    0 0 10px rgba(196, 214, 255, 0.55),
    inset 0 2px 2px rgba(255, 255, 255, 0.7);
}

/* 金属夹本体：中间高光 + 两侧压痕，接近实物夹子 */
.clip::after {
  content: '';
  position: absolute;
  left: 50%;
  top: 15px;
  width: 30px;
  height: 32px;
  transform: translateX(-50%);
  border-radius: 6px 6px 5px 5px;
  background:
    linear-gradient(
      90deg,
      rgba(0, 0, 0, 0.3) 0 2px,
      rgba(255, 255, 255, 0.35) 2px 4px,
      transparent 4px 7px,
      rgba(0, 0, 0, 0.16) 7px 8px,
      transparent 8px 22px,
      rgba(0, 0, 0, 0.16) 22px 23px,
      transparent 23px 26px,
      rgba(255, 255, 255, 0.3) 26px 28px,
      rgba(0, 0, 0, 0.3) 28px 30px
    ),
    linear-gradient(180deg, #ffffff 0%, #e3e7ee 24%, #b0b6c1 56%, #858c97 82%, #6e747f 100%);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.95),
    inset 0 -5px 7px rgba(0, 0, 0, 0.26),
    inset 0 0 0 1px rgba(255, 255, 255, 0.2),
    0 5px 12px rgba(0, 0, 0, 0.6);
}

.card-frame {
  position: relative;
  padding: calc(2px * clamp(0.5, var(--card-scale, 1), 1));
  border-radius: calc(22px * clamp(0.45, var(--card-scale, 1), 1));
  background: linear-gradient(
    180deg,
    rgba(255, 255, 255, 0.72) 0%,
    rgba(var(--glow-rgb), 0.98) 16%,
    rgba(var(--glow-rgb), 0.9) 58%,
    rgba(var(--glow-rgb), 0.62) 100%
  );
  box-shadow:
    0 24px 52px -24px rgba(0, 0, 0, 0.92),
    0 0 22px rgba(var(--glow-rgb), 0.58),
    0 0 56px rgba(var(--glow-rgb), 0.3),
    0 0 100px rgba(var(--glow-rgb), 0.14);
  transition: box-shadow 0.35s var(--ease-out-expo);
}

.image-card:hover .card-frame {
  box-shadow:
    0 30px 64px -24px rgba(0, 0, 0, 0.95),
    0 0 26px rgba(var(--glow-rgb), 0.9),
    0 0 68px rgba(var(--glow-rgb), 0.5),
    0 0 120px rgba(var(--glow-rgb), 0.24);
}

.card-body {
  position: relative;
  border-radius: calc(20px * clamp(0.45, var(--card-scale, 1), 1));
  overflow: hidden;
  background: linear-gradient(180deg, rgba(22, 22, 32, 0.88) 0%, rgba(8, 8, 13, 0.94) 100%);
  backdrop-filter: blur(18px) saturate(1.15);
}

.image-wrapper {
  position: relative;
  margin: calc(8px * clamp(0.35, var(--card-scale, 1), 1))
    calc(8px * clamp(0.35, var(--card-scale, 1), 1)) 0;
  height: calc(148px * var(--card-scale, 1));
  border-radius: calc(13px * clamp(0.45, var(--card-scale, 1), 1));
  overflow: hidden;
}

.image-card.is-compact .caption,
.image-card.is-compact .actions {
  display: none;
}

.image-card.is-compact .card-frame,
.image-card.is-compact:hover .card-frame {
  box-shadow:
    0 10px 24px -12px rgba(0, 0, 0, 0.9),
    0 0 10px rgba(var(--glow-rgb), 0.5),
    0 0 24px rgba(var(--glow-rgb), 0.22);
}

.preview-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
  pointer-events: none;
}

.image-sheen {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.14) 0%, rgba(255, 255, 255, 0) 24%),
    linear-gradient(0deg, rgba(0, 0, 0, 0.42) 0%, rgba(0, 0, 0, 0) 42%);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
}

.caption {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 9px 10px 11px;
}

.caption-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.caption-title {
  color: rgba(255, 255, 255, 0.95);
  font-size: 12.5px;
  font-weight: 600;
  letter-spacing: 0.01em;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.caption-time {
  color: rgba(255, 255, 255, 0.44);
  font-size: 10.5px;
  font-weight: 400;
  white-space: nowrap;
}

.open-btn {
  flex-shrink: 0;
  width: 26px;
  height: 26px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  color: rgba(255, 255, 255, 0.7);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.14);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.open-btn:hover {
  color: #fff;
  background: rgba(var(--glow-rgb), 0.85);
  border-color: rgba(255, 255, 255, 0.42);
  box-shadow: 0 0 16px rgba(var(--glow-rgb), 0.6);
}

.actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transform: translateY(-4px);
  transition:
    opacity var(--duration-fast) var(--ease-out-expo),
    transform var(--duration-fast) var(--ease-out-expo);
}

.image-card:hover .actions {
  opacity: 1;
  transform: translateY(0);
}

.action-btn {
  width: 26px;
  height: 26px;
  border-radius: var(--radius-full);
  background: rgba(8, 8, 12, 0.62);
  border: 1px solid rgba(255, 255, 255, 0.16);
  color: rgba(255, 255, 255, 0.9);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
  backdrop-filter: blur(14px);
}

.action-btn:hover {
  background: rgba(var(--glow-rgb), 0.85);
  border-color: rgba(255, 255, 255, 0.4);
  transform: scale(1.08);
}
</style>
