<template>
  <div
    class="capture-root"
    @mousedown.prevent="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @contextmenu.prevent
  >
    <img
      v-if="frame"
      :src="frame"
      class="capture-frame"
      draggable="false"
      @load="onFrameLoad"
      @error="error = '截图画面加载失败'"
    />
    <canvas ref="overlayEl" class="overlay"></canvas>

    <div v-if="selection" class="size-label" :style="labelStyle">{{ sizeText }}</div>

    <input
      v-if="textEditing"
      ref="textInputEl"
      v-model="textDraft"
      class="text-input"
      :style="{ left: `${textEditing.x}px`, top: `${textEditing.y}px` }"
      placeholder="输入文字，回车确认"
      @mousedown.stop
      @click.stop
      @keydown.enter.prevent.stop="commitText"
      @keydown.esc.prevent.stop="textEditing = null"
      @blur="commitText"
    />

    <div v-if="phase === 'select' && !dragging" class="hint">拖拽框选截图区域 · Esc 取消</div>

    <div v-if="phase === 'edit'" class="toolbar" @mousedown.stop @mouseup.stop @click.stop>
      <button
        v-for="tool in tools"
        :key="tool.id"
        class="tool-btn"
        :class="{ active: activeTool === tool.id }"
        @click="activeTool = tool.id"
      >
        {{ tool.label }}
      </button>
      <span class="divider"></span>
      <button class="tool-btn" :disabled="annotations.length === 0" @click="undo">撤销</button>
      <button class="tool-btn" @click="cancel">取消</button>
      <button class="tool-btn primary" @click="confirm">保存</button>
    </div>

    <div v-if="error" class="error">{{ error }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  ANNOTATION_TOOLS as tools,
  paintAnnotation,
  type Annotation,
  type Point,
  type Tool,
} from './composables/useImageAnnotations';

const frame = ref('');
const overlayEl = ref<HTMLCanvasElement | null>(null);
const frameWidth = ref(0);
const frameHeight = ref(0);
const phase = ref<'select' | 'edit'>('select');
const dragging = ref(false);
const error = ref('');
const activeTool = ref<Tool>('rect');
const annotations = ref<Annotation[]>([]);
const activeAnnotation = ref<Annotation | null>(null);
const textEditing = ref<Point | null>(null);
const textDraft = ref('');
const textInputEl = ref<HTMLInputElement | null>(null);
let frameCanvas: HTMLCanvasElement | null = null;

const start = reactive<Point>({ x: 0, y: 0 });
const current = reactive<Point>({ x: 0, y: 0 });

const selection = computed(() => {
  if (!dragging.value && phase.value !== 'edit') return null;
  const width = Math.abs(current.x - start.x);
  const height = Math.abs(current.y - start.y);
  if (width < 2 || height < 2) return null;
  return {
    left: Math.min(start.x, current.x),
    top: Math.min(start.y, current.y),
    width,
    height,
  };
});

const scaleX = () => (frameWidth.value > 0 ? frameWidth.value / window.innerWidth : 1);
const scaleY = () => (frameHeight.value > 0 ? frameHeight.value / window.innerHeight : 1);

const sizeText = computed(() => {
  const sel = selection.value;
  if (!sel) return '';
  const w = Math.round(sel.width * scaleX());
  const h = Math.round(sel.height * scaleY());
  return `${w} × ${h}`;
});

const labelStyle = computed(() => {
  const sel = selection.value;
  if (!sel) return {};
  return {
    left: `${sel.left}px`,
    top: `${sel.top > 32 ? sel.top - 28 : sel.top + 6}px`,
  };
});

const pointOf = (event: MouseEvent): Point => ({ x: event.clientX, y: event.clientY });

const insideSelection = (sel: { left: number; top: number; width: number; height: number }, p: Point) =>
  p.x >= sel.left && p.x <= sel.left + sel.width && p.y >= sel.top && p.y <= sel.top + sel.height;

const onFrameLoad = (event: Event) => {
  const img = event.target as HTMLImageElement;
  frameWidth.value = img.naturalWidth;
  frameHeight.value = img.naturalHeight;
  frameCanvas = document.createElement('canvas');
  frameCanvas.width = img.naturalWidth;
  frameCanvas.height = img.naturalHeight;
  frameCanvas.getContext('2d')?.drawImage(img, 0, 0);
  redraw();
};

const commitText = () => {
  const anchor = textEditing.value;
  if (!anchor) return;
  const value = textDraft.value.trim();
  textEditing.value = null;
  textDraft.value = '';
  if (!value) return;
  annotations.value.push({ tool: 'text', points: [{ ...anchor }], text: value });
  redraw();
};

const onMouseDown = (event: MouseEvent) => {
  if (event.button !== 0) return;
  commitText();
  error.value = '';
  const p = pointOf(event);

  if (phase.value === 'select') {
    dragging.value = true;
    start.x = p.x;
    start.y = p.y;
    current.x = p.x;
    current.y = p.y;
    redraw();
    return;
  }

  const sel = selection.value;
  if (!sel) return;
  if (!insideSelection(sel, p)) {
    annotations.value = [];
    phase.value = 'select';
    dragging.value = true;
    start.x = p.x;
    start.y = p.y;
    current.x = p.x;
    current.y = p.y;
    redraw();
    return;
  }

  if (activeTool.value === 'text') {
    textEditing.value = { x: p.x, y: p.y };
    textDraft.value = '';
    void nextTick(() => textInputEl.value?.focus());
    return;
  }

  activeAnnotation.value = { tool: activeTool.value, points: [p] };
  redraw();
};

const onMouseMove = (event: MouseEvent) => {
  const p = pointOf(event);
  if (dragging.value) {
    current.x = p.x;
    current.y = p.y;
    redraw();
    return;
  }
  const active = activeAnnotation.value;
  if (!active) return;
  if (active.tool === 'pen') {
    active.points.push(p);
  } else {
    active.points[1] = p;
  }
  redraw();
};

const onMouseUp = () => {
  if (dragging.value) {
    const width = Math.abs(current.x - start.x);
    const height = Math.abs(current.y - start.y);
    dragging.value = false;
    if (width > 6 && height > 6) {
      phase.value = 'edit';
    } else {
      start.x = 0;
      start.y = 0;
      current.x = 0;
      current.y = 0;
    }
    redraw();
    return;
  }

  const active = activeAnnotation.value;
  if (!active) return;
  activeAnnotation.value = null;
  const first = active.points[0];
  const last = active.points[active.points.length - 1];
  const moved = active.tool === 'pen'
    ? active.points.length > 2
    : Math.hypot(last.x - first.x, last.y - first.y) > 4;
  if (moved) annotations.value.push(active);
  redraw();
};

const undo = () => {
  annotations.value.pop();
  redraw();
};

const paintOptions = () => ({
  source: frameCanvas,
  sourceScaleX: frameWidth.value / window.innerWidth,
  sourceScaleY: frameHeight.value / window.innerHeight,
});

const redraw = () => {
  const canvas = overlayEl.value;
  if (!canvas) return;

  const width = window.innerWidth;
  const height = window.innerHeight;
  const dpr = window.devicePixelRatio || 1;
  const pixelWidth = Math.max(1, Math.round(width * dpr));
  const pixelHeight = Math.max(1, Math.round(height * dpr));

  if (canvas.width !== pixelWidth || canvas.height !== pixelHeight) {
    canvas.width = pixelWidth;
    canvas.height = pixelHeight;
  }
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, width, height);

  const sel = selection.value;
  if (!sel) return;

  ctx.fillStyle = 'rgba(6, 6, 16, 0.55)';
  ctx.fillRect(0, 0, width, height);
  ctx.clearRect(sel.left, sel.top, sel.width, sel.height);

  ctx.save();
  ctx.beginPath();
  ctx.rect(sel.left, sel.top, sel.width, sel.height);
  ctx.clip();
  for (const annotation of annotations.value) {
    paintAnnotation(ctx, annotation, paintOptions());
  }
  if (activeAnnotation.value) {
    paintAnnotation(ctx, activeAnnotation.value, paintOptions());
  }
  ctx.restore();

  ctx.strokeStyle = 'rgba(168, 85, 247, 0.95)';
  ctx.lineWidth = 1;
  ctx.strokeRect(sel.left + 0.5, sel.top + 0.5, sel.width - 1, sel.height - 1);
};

const confirm = async () => {
  const sel = selection.value;
  if (!sel) return;

  const sx = scaleX();
  const sy = scaleY();
  const outWidth = Math.max(1, Math.round(sel.width * sx));
  const outHeight = Math.max(1, Math.round(sel.height * sy));

  const canvas = document.createElement('canvas');
  canvas.width = outWidth;
  canvas.height = outHeight;
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  ctx.scale(sx, sy);
  ctx.translate(-sel.left, -sel.top);
  for (const annotation of annotations.value) {
    paintAnnotation(ctx, annotation, paintOptions());
  }

  try {
    await invoke('finish_capture', {
      x: Math.round(sel.left * sx),
      y: Math.round(sel.top * sy),
      width: outWidth,
      height: outHeight,
      overlay: canvas.toDataURL('image/png'),
    });
  } catch (err) {
    error.value = String(err);
  }
};

const cancel = () => {
  void invoke('cancel_capture').catch(() => undefined);
};

const onKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    cancel();
  } else if (event.key === 'Enter') {
    if (phase.value === 'edit') void confirm();
  } else if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z') {
    if (annotations.value.length > 0) undo();
  }
};

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown);
  window.addEventListener('resize', redraw);
  try {
    frame.value = await invoke<string>('get_capture_frame');
  } catch (err) {
    error.value = String(err);
  }
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
  window.removeEventListener('resize', redraw);
});
</script>

<style scoped>
.capture-root {
  position: fixed;
  inset: 0;
  cursor: crosshair;
  user-select: none;
  overflow: hidden;
  background: rgba(6, 6, 16, 0.45);
}

.capture-frame {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
  pointer-events: none;
  -webkit-user-drag: none;
}

.overlay {
  position: absolute;
  inset: 0;
}

.text-input {
  position: absolute;
  min-width: 180px;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid rgba(192, 132, 252, 0.7);
  background: rgba(10, 10, 22, 0.9);
  color: #fff;
  font-size: 14px;
  outline: none;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}

.size-label {
  position: absolute;
  padding: 3px 8px;
  border-radius: 6px;
  background: rgba(10, 10, 22, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.14);
  color: #fff;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  pointer-events: none;
}

.hint {
  position: absolute;
  top: 28px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 18px;
  border-radius: 999px;
  background: rgba(10, 10, 22, 0.78);
  border: 1px solid rgba(255, 255, 255, 0.14);
  backdrop-filter: blur(12px);
  color: rgba(255, 255, 255, 0.9);
  font-size: 12px;
  pointer-events: none;
}

.toolbar {
  position: absolute;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px;
  border-radius: 999px;
  background: rgba(10, 10, 22, 0.86);
  border: 1px solid rgba(255, 255, 255, 0.14);
  backdrop-filter: blur(16px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.45);
}

.tool-btn {
  height: 30px;
  padding: 0 12px;
  border-radius: 999px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.82);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tool-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
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
  background: var(--gradient-primary, linear-gradient(135deg, #a855f7, #6366f1));
  color: #fff;
}

.divider {
  width: 1px;
  height: 18px;
  background: rgba(255, 255, 255, 0.16);
  margin: 0 4px;
}

.error {
  position: absolute;
  bottom: 80px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 16px;
  border-radius: 8px;
  background: rgba(220, 38, 38, 0.85);
  color: #fff;
  font-size: 12px;
}
</style>
