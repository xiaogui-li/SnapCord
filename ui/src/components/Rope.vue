<template>
  <div class="rope-shelf" :style="shelfStyle">
    <!-- Rope: a catenary between the two top corners of the screen -->
    <svg
      class="rope-wire"
      :width="width"
      height="120"
      :viewBox="`0 0 ${width} 120`"
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id="rope-tint" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stop-color="#f4f4f5" />
          <stop offset="12%" stop-color="#c084fc" />
          <stop offset="34%" stop-color="#818cf8" />
          <stop offset="55%" stop-color="#22d3ee" />
          <stop offset="72%" stop-color="#34d399" />
          <stop offset="90%" stop-color="#a7f3d0" />
          <stop offset="100%" stop-color="#ecfdf5" />
        </linearGradient>
        <filter id="rope-glow" x="-6%" y="-260%" width="112%" height="620%">
          <feGaussianBlur stdDeviation="6" />
        </filter>
      </defs>
      <path class="rope-halo" :d="ropePath" />
      <path class="rope-glow" :d="ropePath" />
      <path class="rope-line" :d="ropePath" />
      <path class="rope-core" :d="ropePath" />
      <circle class="rope-knot" cx="4" :cy="ROPE_Y" r="4.5" />
      <circle class="rope-knot" :cx="width - 4" :cy="ROPE_Y" r="4.5" />
    </svg>

    <div class="rope-scroll" ref="scrollEl" @scroll="scheduleLayout" @wheel="handleWheel">
      <div class="rope-track">
        <div
          v-for="(image, index) in images"
          :key="image.id"
          class="hang-slot"
          data-hit
          :style="slotStyle(index)"
        >
          <ImageCard
            :image="image"
            :index="index"
            :is-expanded="isExpanded"
            :card-scale="cardScale"
            :card-swing="cardSwing"
            @click="handleImageClick"
            @drag-start="handleDragStart"
            @pin="handlePin"
            @copy="handleCopy"
            @basket="handleBasket"
            @edit="handleEdit"
            @mouseenter="isExpanded = true"
            @mouseleave="isExpanded = false"
          />
        </div>
      </div>
    </div>

    <div v-if="canScrollLeft" class="scroll-fade"></div>

    <div v-if="images.length === 0" class="empty-state">
      <p class="empty-title">绳索空着</p>
      <p class="empty-text">截图与剪贴板图片会自动挂在上面</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import ImageCard from './ImageCard.vue';
import type { ImageItem } from '../types';

const props = defineProps<{
  images: ImageItem[];
  cardScale?: number;
  ropeSag?: number;
  cardSwing?: number;
  isVisible?: boolean;
  animationStyle?: number;
  animationDuration?: number;
}>();

const MIN_CARD_SCALE = 0.2;
const MAX_CARD_SCALE = 1.3;
/// 弧度 100% 时的最大下垂量（像素）。
const MAX_SAG = 80;

const cardScale = computed(() =>
  Math.min(MAX_CARD_SCALE, Math.max(MIN_CARD_SCALE, props.cardScale ?? 1)),
);

const sag = computed(() => {
  const percent = Math.min(100, Math.max(0, props.ropeSag ?? 35));
  return (percent / 100) * MAX_SAG;
});

const emit = defineEmits<{
  (e: 'image-click', image: ImageItem): void;
  (e: 'drag-start', image: ImageItem): void;
  (e: 'pin', image: ImageItem): void;
  (e: 'copy', image: ImageItem): void;
  (e: 'basket', image: ImageItem): void;
  (e: 'edit', image: ImageItem): void;
}>();

const ROPE_Y = 24;

const width = ref(window.innerWidth);
const scrollEl = ref<HTMLDivElement>();
const slotPositions = ref<number[]>([]);
const isExpanded = ref(false);
const canScrollLeft = ref(false);
const animationProgress = ref(0);
const isAnimating = ref(false);

const shelfStyle = computed(() => ({
  '--card-scale': cardScale.value,
}));

let frame = 0;
let animationFrame = 0;

// 弧度展开：绳头沿最终弧线走（de Casteljau 子曲线），
// 图片保持固定间距随绳滑入，Y 跟随当前位置的绳索高度。
const ropePath = computed(() => {
  if (animationProgress.value < 1) {
    // 动画进行中，根据进度计算路径
    const progress = animationProgress.value;
    if (props.animationStyle === 1) {
      // 拉绳抖动风格：先直线后下坠
      if (progress < 0.5) {
        // 第一阶段：直线从左到右
        const lineProgress = progress * 2;
        const endX = width.value * lineProgress;
        return `M 0 ${ROPE_Y} L ${endX} ${ROPE_Y}`;
      } else {
        // 第二阶段：下坠到既定曲线
        const dropProgress = (progress - 0.5) * 2;
        const currentSag = sag.value * dropProgress;
        return `M 0 ${ROPE_Y} Q ${width.value / 2} ${ROPE_Y + currentSag * 2} ${width.value} ${ROPE_Y}`;
      }
    } else {
      // 弧度展开风格：线头沿弧度路径移动
      // 使用 de Casteljau 算法计算子曲线
      const endpointX = progress * width.value;
      const endpointY = ROPE_Y + 4 * sag.value * progress * (1 - progress);
      const controlX = progress * width.value / 2;
      const controlY = ROPE_Y + progress * sag.value * 2;
      return `M 0 ${ROPE_Y} Q ${controlX} ${controlY} ${endpointX} ${endpointY}`;
    }
  }
  return `M 0 ${ROPE_Y} Q ${width.value / 2} ${ROPE_Y + sag.value * 2} ${width.value} ${ROPE_Y}`;
});

const tiltAt = (index: number) => (((index * 37) % 7) - 3) * 0.6;

// 吊环对齐修正：吊环底部在 .hang-slot 顶部上方 12px（-30+18），
// 但 .image-card 的 margin-top: 14px 把卡片往下推了 14px，
// 所以 net offset = 14 - 12 = 2px，translateY 需要 -2 才能让吊环贴住绳索。
const HANGER_OFFSET = -2;

const slotStyle = (index: number) => {
  const finalPosition = slotPositions.value[index] ?? 0;
  const progress = animationProgress.value;
  const tilt = tiltAt(index);

  // 图片固定挂在绳索的固定位置：绳索（含图片）整体从左侧屏幕外拉入，
  // 间距不变，最右侧的图最先出现；Y 跟随当前位置的绳索弧线高度。
  const currentPosition = (finalPosition - 1 + progress) * width.value;
  const translateX = currentPosition - finalPosition * width.value;
  const currentFraction = Math.min(1, Math.max(0, currentPosition / width.value));
  const currentSag = 4 * sag.value * currentFraction * (1 - currentFraction);
  const translateY = currentSag + HANGER_OFFSET;

  return {
    transform: `translateX(${translateX}px) translateY(${translateY}px) rotate(${tilt}deg)`,
  };
};

const layout = () => {
  width.value = window.innerWidth;
  const root = scrollEl.value;
  if (!root) {
    slotPositions.value = [];
    return;
  }
  canScrollLeft.value = root.scrollLeft > 4;
  const slots = Array.from(root.querySelectorAll<HTMLElement>('.hang-slot'));
  // 挂点必须用视口坐标（rect 会随滚动变化，这正是需要的语义），
  // 再减掉当前动画的整体位移，这样动画中/首帧测量都不会被 transform 污染。
  const shiftX = (animationProgress.value - 1) * width.value;
  slotPositions.value = slots.map((slot) => {
    const rect = slot.getBoundingClientRect();
    const center = rect.left + rect.width / 2 - shiftX;
    return Math.min(1, Math.max(0, center / width.value));
  });
};

const scheduleLayout = () => {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
    layout();
  });
};

watch(
  () => props.images,
  () => nextTick(scheduleLayout),
  { deep: true },
);

watch(
  () => props.ropeSag,
  () => nextTick(scheduleLayout),
);

watch(
  () => props.cardScale,
  () => nextTick(scheduleLayout),
);

watch(
  () => props.isVisible,
  (visible) => {
    if (visible) {
      showRope();
    } else {
      hideRope();
    }
  },
  { immediate: true },
);

const animationDuration = computed(() => props.animationDuration ?? 800);

const showRope = () => {
  if (isAnimating.value) return;
  isAnimating.value = true;
  animationProgress.value = 0;
  let startTime: number | null = null;

  const animate = (currentTime: number) => {
    if (startTime === null) {
      startTime = currentTime;
    }
    const elapsed = currentTime - startTime;
    const progress = Math.min(1, elapsed / animationDuration.value);
    animationProgress.value = 1 - Math.pow(1 - progress, 3);

    if (progress < 1) {
      animationFrame = requestAnimationFrame(animate);
    } else {
      isAnimating.value = false;
      animationProgress.value = 1;
      nextTick(scheduleLayout);
    }
  };

  animationFrame = requestAnimationFrame(animate);
};

const hideRope = () => {
  if (isAnimating.value) return;
  isAnimating.value = true;
  let startTime: number | null = null;

  const animate = (currentTime: number) => {
    if (startTime === null) {
      startTime = currentTime;
    }
    const elapsed = currentTime - startTime;
    const progress = Math.min(1, elapsed / animationDuration.value);
    animationProgress.value = 1 - Math.pow(progress, 3);

    if (progress < 1) {
      animationFrame = requestAnimationFrame(animate);
    } else {
      isAnimating.value = false;
      animationProgress.value = 0;
    }
  };

  animationFrame = requestAnimationFrame(animate);
};

// 新图片挂在最右端，滚动过去让用户直接看到。
watch(
  () => props.images.length,
  (count, previous) => {
    if (count <= previous) return;
    nextTick(() => {
      const el = scrollEl.value;
      if (el) el.scrollLeft = el.scrollWidth;
      scheduleLayout();
    });
  },
);

const handleWheel = (event: WheelEvent) => {
  const el = scrollEl.value;
  if (!el || Math.abs(event.deltaY) <= Math.abs(event.deltaX)) return;
  el.scrollLeft += event.deltaY;
  event.preventDefault();
};

onMounted(() => {
  window.addEventListener('resize', scheduleLayout);
  nextTick(scheduleLayout);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', scheduleLayout);
  if (frame) cancelAnimationFrame(frame);
  if (animationFrame) cancelAnimationFrame(animationFrame);
});

const handleImageClick = (image: ImageItem) => {
  emit('image-click', image);
};

const handleDragStart = (image: ImageItem) => {
  emit('drag-start', image);
};

const handlePin = (image: ImageItem) => {
  emit('pin', image);
};

const handleCopy = (image: ImageItem) => {
  emit('copy', image);
};

const handleBasket = (image: ImageItem) => {
  emit('basket', image);
};

const handleEdit = (image: ImageItem) => {
  emit('edit', image);
};
</script>

<style scoped>
.rope-shelf {
  position: relative;
  width: 100%;
  height: 100vh;
}

.rope-wire {
  position: absolute;
  top: 0;
  left: 0;
  overflow: visible;
  pointer-events: none;
  z-index: 1;
}

.rope-halo {
  fill: none;
  stroke: rgba(125, 211, 252, 0.13);
  stroke-width: 30;
  filter: blur(11px);
}

.rope-glow {
  fill: none;
  stroke: rgba(168, 85, 247, 0.62);
  stroke-width: 12;
  filter: url(#rope-glow);
}

.rope-line {
  fill: none;
  stroke: url(#rope-tint);
  stroke-width: 4;
  stroke-linecap: round;
  filter: drop-shadow(0 0 7px rgba(192, 132, 252, 0.75))
    drop-shadow(0 0 22px rgba(34, 211, 238, 0.4));
}

.rope-core {
  fill: none;
  stroke: rgba(255, 255, 255, 0.72);
  stroke-width: 1.1;
  stroke-linecap: round;
}

.rope-knot {
  fill: #f8fafc;
  stroke: rgba(0, 0, 0, 0.4);
  stroke-width: 1;
  filter: drop-shadow(0 0 8px rgba(255, 255, 255, 0.7));
}

.rope-scroll {
  position: absolute;
  inset: 0;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: none;
  z-index: 2;
}

.rope-scroll::-webkit-scrollbar {
  display: none;
}

.scroll-fade {
  position: absolute;
  top: 0;
  left: 0;
  width: 56px;
  height: 100%;
  z-index: 3;
  pointer-events: none;
  background: linear-gradient(90deg, rgba(3, 3, 8, 0.7) 0%, rgba(3, 3, 8, 0) 100%);
}

.rope-track {
  display: flex;
  align-items: flex-start;
  gap: calc(52px * var(--card-scale, 1));
  width: max-content;
  min-width: 100%;
  box-sizing: border-box;
  padding: 24px 96px 0 44px;
}

.hang-slot {
  flex: 0 0 auto;
  transform-origin: 50% 0;
}

.hang-slot:first-child {
  margin-left: auto;
}

.hang-slot:last-child {
  margin-right: auto;
}

.empty-state {
  position: absolute;
  top: 96px;
  left: 50%;
  transform: translateX(-50%);
  text-align: center;
  pointer-events: none;
  z-index: 2;
}

.empty-title {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  letter-spacing: 0.08em;
}

.empty-text {
  margin-top: var(--space-1);
  color: var(--text-muted);
  font-size: var(--text-xs, 12px);
  font-weight: var(--font-light);
}
</style>
