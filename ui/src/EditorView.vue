<template>
  <div class="editor-root">
    <header class="titlebar" data-tauri-drag-region>
      <span class="title" data-tauri-drag-region>编辑图片</span>
      <button class="icon-btn close" title="关闭" @click="closeWindow">✕</button>
    </header>

    <div ref="stageEl" class="stage">
      <canvas
        ref="canvasEl"
        class="work-canvas"
        @mousedown.stop.prevent="onCanvasMouseDown"
        @mousemove="onCanvasMouseMove"
        @mouseup="onCanvasMouseUp"
        @mouseleave="onCanvasMouseUp"
      ></canvas>

      <input
        v-if="textEditing"
        ref="textInputEl"
        v-model="textDraft"
        class="text-input"
        :style="textInputStyle"
        placeholder="输入文字，回车确认"
        @mousedown.stop
        @click.stop
        @keydown.enter.prevent.stop="commitText"
        @keydown.esc.prevent.stop="cancelText"
        @blur="commitText"
      />

      <div v-if="!imageReady && !error" class="loading">加载中…</div>
      <div v-if="error" class="error">{{ error }}</div>
    </div>

    <footer class="toolbar">
      <button class="tool-btn" :class="{ active: tool === 'crop' }" @click="tool = 'crop'">裁剪</button>
      <span class="divider"></span>
      <button
        v-for="item in annotationTools"
        :key="item.id"
        class="tool-btn"
        :class="{ active: tool === item.id }"
        @click="tool = item.id"
      >
        {{ item.label }}
      </button>
      <span class="divider"></span>
      <button class="tool-btn" :disabled="annotations.length === 0" @click="undo">撤销</button>
      <button class="tool-btn" title="向左旋转 90°" @click="rotate(-90)">左转</button>
      <button class="tool-btn" title="向右旋转 90°" @click="rotate(90)">右转</button>
      <span class="divider"></span>
      <button class="tool-btn primary" :disabled="!imageReady || saving" @click="save">
        {{ saving ? '保存中…' : '保存' }}
      </button>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { api, isTauri } from './api';
import {
  ANNOTATION_TOOLS,
  paintAnnotation,
  type Annotation,
  type Point,
  type Tool,
} from './composables/useImageAnnotations';

interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface DragState {
  start: Point;
  current: Point;
}

const params = new URLSearchParams(window.location.search);
const imageId = params.get('id') ?? '';

const annotationTools = ANNOTATION_TOOLS;
const stageEl = ref<HTMLDivElement | null>(null);
const canvasEl = ref<HTMLCanvasElement | null>(null);
const textInputEl = ref<HTMLInputElement | null>(null);

const imageReady = ref(false);
const error = ref('');
const saving = ref(false);
const tool = ref<'crop' | Tool>('crop');
const annotations = ref<Annotation[]>([]);
const activeAnnotation = ref<Annotation | null>(null);
const crop = ref<Rect | null>(null);
const dragState = ref<DragState | null>(null);
const textEditing = ref<Point | null>(null);
const textDraft = ref('');
const rotation = ref(0);

let imageEl: HTMLImageElement | null = null;
let baseCanvas: HTMLCanvasElement | null = null;
let baseRotation = -1;

const textInputStyle = computed(() => {
  const point = textEditing.value;
  const canvas = canvasEl.value;
  const stage = stageEl.value;
  if (!point || !canvas || !stage) return {};
  const canvasRect = canvas.getBoundingClientRect();
  const stageRect = stage.getBoundingClientRect();
  return {
    left: `${canvasRect.left - stageRect.left + (point.x / canvas.width) * canvasRect.width}px`,
    top: `${canvasRect.top - stageRect.top + (point.y / canvas.height) * canvasRect.height}px`,
  };
});

const rotatedSize = (width: number, height: number, angle: number) =>
  angle === 90 || angle === 270 ? { width: height, height: width } : { width, height };

const applyRotation = (
  ctx: CanvasRenderingContext2D,
  width: number,
  height: number,
  angle: number,
) => {
  if (angle === 90) {
    ctx.translate(width, 0);
    ctx.rotate(Math.PI / 2);
  } else if (angle === 180) {
    ctx.translate(width, height);
    ctx.rotate(Math.PI);
  } else if (angle === 270) {
    ctx.translate(0, height);
    ctx.rotate(-Math.PI / 2);
  }
};

const buildBaseCanvas = () => {
  if (!imageEl) return null;
  const dims = rotatedSize(imageEl.naturalWidth, imageEl.naturalHeight, rotation.value);
  if (baseCanvas && baseRotation === rotation.value) return baseCanvas;

  const base = document.createElement('canvas');
  base.width = dims.width;
  base.height = dims.height;
  const ctx = base.getContext('2d');
  if (!ctx) return null;
  ctx.save();
  applyRotation(ctx, dims.width, dims.height, rotation.value);
  ctx.drawImage(imageEl, 0, 0);
  ctx.restore();
  baseCanvas = base;
  baseRotation = rotation.value;
  return base;
};

const render = () => {
  const canvas = canvasEl.value;
  if (!canvas) return;
  const base = buildBaseCanvas();
  if (!base) return;

  if (canvas.width !== base.width || canvas.height !== base.height) {
    canvas.width = base.width;
    canvas.height = base.height;
  }
  const ctx = canvas.getContext('2d');
  if (!ctx) return;
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.drawImage(base, 0, 0);

  const activeCrop = crop.value ?? cropDraftRect();
  if (activeCrop) {
    ctx.fillStyle = 'rgba(4, 4, 12, 0.55)';
    ctx.fillRect(0, 0, canvas.width, activeCrop.y);
    ctx.fillRect(0, activeCrop.y + activeCrop.height, canvas.width, canvas.height - activeCrop.y - activeCrop.height);
    ctx.fillRect(0, activeCrop.y, activeCrop.x, activeCrop.height);
    ctx.fillRect(
      activeCrop.x + activeCrop.width,
      activeCrop.y,
      canvas.width - activeCrop.x - activeCrop.width,
      activeCrop.height,
    );
    ctx.strokeStyle = 'rgba(168, 85, 247, 0.95)';
    ctx.lineWidth = 1;
    ctx.strokeRect(activeCrop.x + 0.5, activeCrop.y + 0.5, activeCrop.width - 1, activeCrop.height - 1);
  }

  const options = { source: baseCanvas, sourceScaleX: 1, sourceScaleY: 1 };
  for (const annotation of annotations.value) {
    paintAnnotation(ctx, annotation, options);
  }
  if (activeAnnotation.value) {
    paintAnnotation(ctx, activeAnnotation.value, options);
  }
};

const cropDraftRect = (): Rect | null => {
  const drag = dragState.value;
  if (!drag || tool.value !== 'crop') return null;
  const x = Math.min(drag.start.x, drag.current.x);
  const y = Math.min(drag.start.y, drag.current.y);
  const width = Math.abs(drag.current.x - drag.start.x);
  const height = Math.abs(drag.current.y - drag.start.y);
  if (width < 2 || height < 2) return null;
  return { x, y, width, height };
};

const pointOf = (event: MouseEvent): Point => {
  const canvas = canvasEl.value;
  if (!canvas) return { x: 0, y: 0 };
  const rect = canvas.getBoundingClientRect();
  return {
    x: ((event.clientX - rect.left) * canvas.width) / rect.width,
    y: ((event.clientY - rect.top) * canvas.height) / rect.height,
  };
};

const cancelText = () => {
  textEditing.value = null;
  textDraft.value = '';
};

const commitText = () => {
  const anchor = textEditing.value;
  if (!anchor) return;
  const value = textDraft.value.trim();
  textEditing.value = null;
  textDraft.value = '';
  if (!value) return;
  annotations.value.push({ tool: 'text', points: [{ ...anchor }], text: value });
  render();
};

const onCanvasMouseDown = (event: MouseEvent) => {
  if (event.button !== 0 || !imageReady.value) return;
  commitText();
  const point = pointOf(event);

  if (tool.value === 'text') {
    textEditing.value = point;
    textDraft.value = '';
    void nextTick(() => textInputEl.value?.focus());
    return;
  }

  dragState.value = { start: point, current: point };
  if (tool.value !== 'crop') {
    activeAnnotation.value = { tool: tool.value, points: [point] };
  }
  render();
};

const onCanvasMouseMove = (event: MouseEvent) => {
  const drag = dragState.value;
  if (!drag) return;
  const point = pointOf(event);
  drag.current = point;
  if (tool.value === 'crop') {
    render();
    return;
  }
  const active = activeAnnotation.value;
  if (!active) return;
  if (active.tool === 'pen') {
    active.points.push(point);
  } else {
    active.points[1] = point;
  }
  render();
};

const onCanvasMouseUp = () => {
  const drag = dragState.value;
  if (!drag) return;
  dragState.value = null;

  if (tool.value === 'crop') {
    const x = Math.min(drag.start.x, drag.current.x);
    const y = Math.min(drag.start.y, drag.current.y);
    const width = Math.abs(drag.current.x - drag.start.x);
    const height = Math.abs(drag.current.y - drag.start.y);
    crop.value = width > 6 && height > 6 ? { x, y, width, height } : null;
    render();
    return;
  }

  const active = activeAnnotation.value;
  activeAnnotation.value = null;
  if (!active) return;
  const first = active.points[0];
  const last = active.points[active.points.length - 1];
  const moved =
    active.tool === 'pen'
      ? active.points.length > 2
      : Math.hypot(last.x - first.x, last.y - first.y) > 4;
  if (moved) annotations.value.push(active);
  render();
};

const undo = () => {
  annotations.value.pop();
  render();
};

const rotate = (delta: 90 | -90) => {
  commitText();
  const canvas = canvasEl.value;
  if (!canvas) return;
  const { width: currentWidth, height: currentHeight } = canvas;
  const clockwise = delta === 90;

  annotations.value = annotations.value.map((annotation) => ({
    ...annotation,
    points: annotation.points.map((point) =>
      clockwise
        ? { x: currentHeight - point.y, y: point.x }
        : { x: point.y, y: currentWidth - point.x },
    ),
  }));

  if (crop.value) {
    const rect = crop.value;
    crop.value = clockwise
      ? { x: currentHeight - (rect.y + rect.height), y: rect.x, width: rect.height, height: rect.width }
      : { x: rect.y, y: currentWidth - (rect.x + rect.width), width: rect.height, height: rect.width };
  }

  rotation.value = (rotation.value + (clockwise ? 90 : 270)) % 360;
  render();
};

const save = async () => {
  const canvas = canvasEl.value;
  if (!canvas || !imageReady.value || saving.value) return;
  commitText();

  const base = buildBaseCanvas();
  if (!base) return;

  const rect = crop.value;
  const output = document.createElement('canvas');
  output.width = Math.max(1, Math.round(rect ? rect.width : base.width));
  output.height = Math.max(1, Math.round(rect ? rect.height : base.height));
  const ctx = output.getContext('2d');
  if (!ctx) return;

  ctx.save();
  if (rect) ctx.translate(-Math.round(rect.x), -Math.round(rect.y));
  ctx.drawImage(base, 0, 0);
  const options = { source: base, sourceScaleX: 1, sourceScaleY: 1 };
  for (const annotation of annotations.value) {
    paintAnnotation(ctx, annotation, options);
  }
  ctx.restore();

  saving.value = true;
  try {
    await api.saveEditedImage(output.toDataURL('image/png'));
    await closeWindow();
  } catch (err) {
    error.value = String(err);
    saving.value = false;
  }
};

const closeWindow = async () => {
  await getCurrentWindow().close().catch(() => undefined);
};

const onKeyDown = (event: KeyboardEvent) => {
  if (textEditing.value) return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z') {
    event.preventDefault();
    undo();
  } else if (event.key === 'Escape') {
    void closeWindow();
  } else if (event.key === 'Enter') {
    void save();
  }
};

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown);
  if (!isTauri()) {
    error.value = '仅支持在桌面客户端中使用';
    return;
  }
  try {
    const images = await api.getImages();
    const item = images.find((image) => image.id === imageId);
    if (!item) {
      error.value = '图片不存在';
      return;
    }
    // 通过 IPC 读取原始字节再转 blob，避免 asset 协议图片污染画布导致无法导出
    const bytes = await api.readImageBytes(item.path);
    const objectUrl = URL.createObjectURL(new Blob([bytes]));
    const element = new Image();
    element.onload = () => {
      URL.revokeObjectURL(objectUrl);
      imageEl = element;
      imageReady.value = true;
      render();
    };
    element.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      error.value = '图片加载失败';
    };
    element.src = objectUrl;
  } catch (err) {
    error.value = String(err);
  }
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
});
</script>

<style scoped>
.editor-root {
  position: fixed;
  inset: 0;
  display: grid;
  grid-template-rows: 44px minmax(0, 1fr) auto;
  background: rgba(8, 8, 16, 0.92);
  backdrop-filter: blur(24px);
  color: var(--text-primary);
  overflow: hidden;
}

.titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-4);
  border-bottom: 1px solid var(--border-hairline);
}

.title {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  letter-spacing: 0.04em;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.icon-btn:hover {
  background: rgba(239, 68, 68, 0.85);
  color: #fff;
}

.stage {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4);
  min-height: 0;
  overflow: hidden;
}

.work-canvas {
  max-width: 100%;
  max-height: 100%;
  border-radius: var(--radius-md);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.6);
  cursor: crosshair;
}

.text-input {
  position: absolute;
  min-width: 180px;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(192, 132, 252, 0.7);
  background: rgba(10, 10, 22, 0.92);
  color: #fff;
  font-size: 14px;
  outline: none;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  z-index: 2;
}

.loading,
.error {
  position: absolute;
  padding: 8px 18px;
  border-radius: 999px;
  background: rgba(10, 10, 22, 0.85);
  border: 1px solid var(--border-hairline);
  font-size: 12px;
}

.error {
  color: #fca5a5;
  border-color: rgba(239, 68, 68, 0.6);
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border-hairline);
  overflow-x: auto;
}

.tool-btn {
  height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--duration-fast) var(--ease-out-expo);
}

.tool-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.tool-btn.active {
  background: rgba(168, 85, 247, 0.85);
  color: #fff;
}

.tool-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.tool-btn.primary {
  background: var(--gradient-primary);
  color: #fff;
}

.divider {
  width: 1px;
  height: 18px;
  background: rgba(255, 255, 255, 0.16);
  margin: 0 4px;
}
</style>
