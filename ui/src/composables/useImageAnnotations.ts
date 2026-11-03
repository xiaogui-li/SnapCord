export type Tool = 'rect' | 'ellipse' | 'arrow' | 'pen' | 'text' | 'mosaic';

export interface Point {
  x: number;
  y: number;
}

export interface Annotation {
  tool: Tool;
  points: Point[];
  text?: string;
}

export const ANNOTATION_TOOLS: { id: Tool; label: string }[] = [
  { id: 'rect', label: '矩形' },
  { id: 'ellipse', label: '圆形' },
  { id: 'arrow', label: '箭头' },
  { id: 'pen', label: '画笔' },
  { id: 'text', label: '文字' },
  { id: 'mosaic', label: '马赛克' },
];

export const ACCENT = '#c084fc';
export const TEXT_FONT = '600 16px system-ui, -apple-system, "Segoe UI", sans-serif';

const MOSAIC_BLOCK = 10;

export interface PaintOptions {
  source?: HTMLCanvasElement | null;
  sourceScaleX?: number;
  sourceScaleY?: number;
}

const paintMosaic = (
  ctx: CanvasRenderingContext2D,
  points: Point[],
  source: HTMLCanvasElement | null,
  scaleX: number,
  scaleY: number,
) => {
  if (!source || points.length < 2) return;
  const p0 = points[0];
  const p1 = points[points.length - 1];
  const left = Math.min(p0.x, p1.x);
  const top = Math.min(p0.y, p1.y);
  const width = Math.abs(p1.x - p0.x);
  const height = Math.abs(p1.y - p0.y);
  if (width < 2 || height < 2) return;

  const tinyWidth = Math.max(1, Math.round(width / MOSAIC_BLOCK));
  const tinyHeight = Math.max(1, Math.round(height / MOSAIC_BLOCK));
  const tiny = document.createElement('canvas');
  tiny.width = tinyWidth;
  tiny.height = tinyHeight;
  const tinyCtx = tiny.getContext('2d');
  if (!tinyCtx) return;
  tinyCtx.drawImage(
    source,
    left * scaleX,
    top * scaleY,
    width * scaleX,
    height * scaleY,
    0,
    0,
    tinyWidth,
    tinyHeight,
  );

  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(tiny, 0, 0, tinyWidth, tinyHeight, left, top, width, height);
  ctx.restore();
};

export const paintAnnotation = (
  ctx: CanvasRenderingContext2D,
  annotation: Annotation,
  options: PaintOptions = {},
) => {
  const points = annotation.points;
  if (points.length === 0) return;

  ctx.strokeStyle = ACCENT;
  ctx.fillStyle = ACCENT;
  ctx.lineWidth = 2;
  ctx.lineCap = 'round';
  ctx.lineJoin = 'round';

  if (annotation.tool === 'text') {
    if (!annotation.text) return;
    ctx.font = TEXT_FONT;
    ctx.textBaseline = 'top';
    ctx.fillText(annotation.text, points[0].x, points[0].y);
    return;
  }

  if (annotation.tool === 'mosaic') {
    paintMosaic(
      ctx,
      points,
      options.source ?? null,
      options.sourceScaleX ?? 1,
      options.sourceScaleY ?? 1,
    );
    return;
  }

  if (annotation.tool === 'pen') {
    if (points.length < 2) return;
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    for (let i = 1; i < points.length; i += 1) {
      ctx.lineTo(points[i].x, points[i].y);
    }
    ctx.stroke();
    return;
  }

  const p0 = points[0];
  const p1 = points[points.length - 1];

  if (annotation.tool === 'rect') {
    ctx.strokeRect(
      Math.min(p0.x, p1.x),
      Math.min(p0.y, p1.y),
      Math.abs(p1.x - p0.x),
      Math.abs(p1.y - p0.y),
    );
    return;
  }

  if (annotation.tool === 'ellipse') {
    const cx = (p0.x + p1.x) / 2;
    const cy = (p0.y + p1.y) / 2;
    ctx.beginPath();
    ctx.ellipse(cx, cy, Math.abs(p1.x - p0.x) / 2, Math.abs(p1.y - p0.y) / 2, 0, 0, Math.PI * 2);
    ctx.stroke();
    return;
  }

  ctx.beginPath();
  ctx.moveTo(p0.x, p0.y);
  ctx.lineTo(p1.x, p1.y);
  ctx.stroke();

  const angle = Math.atan2(p1.y - p0.y, p1.x - p0.x);
  const head = 12;
  ctx.beginPath();
  ctx.moveTo(p1.x, p1.y);
  ctx.lineTo(p1.x - head * Math.cos(angle - Math.PI / 6), p1.y - head * Math.sin(angle - Math.PI / 6));
  ctx.lineTo(p1.x - head * Math.cos(angle + Math.PI / 6), p1.y - head * Math.sin(angle + Math.PI / 6));
  ctx.closePath();
  ctx.fill();
};
