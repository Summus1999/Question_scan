/**
 * 手动框选覆盖层组件（阶段 4 兜底 UI）。
 *
 * 职责：当自动识别置信度不足时，为用户提供手动拖拽框选题目区域的能力。
 * 交互：Pointer 事件拖拽绘制矩形 → 确认/取消/重试识别。
 */
import { Check, Crop, RotateCcw, X } from 'lucide-react';
import {
  type PointerEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';
import type { CropSelectionRect } from '../lib/types';

type CropOverlayProps = {
  active: boolean;
  title: string;
  description: string;
  idleHint: string;
  selectionLabel: string;
  cancelLabel: string;
  retryLabel: string;
  confirmLabel: string;
  confirmDisabledHint: string;
  selection: CropSelectionRect | null;
  onSelectionChange: (selection: CropSelectionRect | null) => void;
  onSelectionComplete?: (selection: CropSelectionRect | null) => void;
  onCancel: () => void;
  onRetryRecognition: () => void;
  onConfirmSelection: (selection: CropSelectionRect) => void;
};

type Point = {
  x: number;
  y: number;
};

/** 将坐标值限制在视口范围内，防止拖拽超出窗口边界。 */
function clampToViewport(value: number, max: number) {
  return Math.min(Math.max(value, 0), max);
}

/** 从 PointerEvent 读取鼠标/触摸坐标，并限制在视口内。 */
function readPoint(event: PointerEvent<HTMLDivElement>): Point {
  return {
    x: clampToViewport(Math.round(event.clientX), window.innerWidth),
    y: clampToViewport(Math.round(event.clientY), window.innerHeight),
  };
}

/** 根据起点和终点构建选择矩形。如果宽或高为 0，返回 null。 */
function buildSelection(start: Point, end: Point): CropSelectionRect | null {
  const x = Math.min(start.x, end.x);
  const y = Math.min(start.y, end.y);
  const width = Math.abs(end.x - start.x);
  const height = Math.abs(end.y - start.y);

  if (width === 0 || height === 0) {
    return null;
  }

  return {
    x,
    y,
    width,
    height,
  };
}

/** 手动框选覆盖层主组件。
 *
 * 当 active 为 true 时，覆盖整个视口，用户可通过拖拽绘制选择矩形。
 * 支持：拖拽绘制、取消选择、重试自动识别、确认选择。
 */
// Owns the manual fallback selection layer that sits above the shell while the user drags.
export function CropOverlay({
  active,
  title,
  description,
  idleHint,
  selectionLabel,
  cancelLabel,
  retryLabel,
  confirmLabel,
  confirmDisabledHint,
  selection,
  onSelectionChange,
  onSelectionComplete,
  onCancel,
  onRetryRecognition,
  onConfirmSelection,
}: CropOverlayProps) {
  const dragStartRef = useRef<Point | null>(null);
  const pointerIdRef = useRef<number | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    if (!active) {
      dragStartRef.current = null;
      pointerIdRef.current = null;
      setIsDragging(false);
      onSelectionChange(null);
    }
  }, [active, onSelectionChange]);

  const releasePointer = useCallback((event: PointerEvent<HTMLDivElement>) => {
    if (event.currentTarget.hasPointerCapture?.(event.pointerId)) {
      event.currentTarget.releasePointerCapture?.(event.pointerId);
    }
  }, []);

  const finishSelection = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      const start = dragStartRef.current;
      if (!start) {
        return;
      }

      const nextSelection = buildSelection(start, readPoint(event));
      dragStartRef.current = null;
      pointerIdRef.current = null;
      setIsDragging(false);
      onSelectionChange(nextSelection);
      onSelectionComplete?.(nextSelection);
      releasePointer(event);
    },
    [onSelectionChange, onSelectionComplete, releasePointer],
  );

  const clearSelection = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      if (pointerIdRef.current !== event.pointerId) {
        return;
      }

      dragStartRef.current = null;
      pointerIdRef.current = null;
      setIsDragging(false);
      onSelectionChange(null);
      onSelectionComplete?.(null);
      releasePointer(event);
    },
    [onSelectionChange, onSelectionComplete, releasePointer],
  );

  const handlePointerDown = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      if (event.button !== 0) {
        return;
      }

      event.preventDefault();
      dragStartRef.current = readPoint(event);
      pointerIdRef.current = event.pointerId;
      setIsDragging(true);
      onSelectionChange(null);
      event.currentTarget.setPointerCapture?.(event.pointerId);
    },
    [onSelectionChange],
  );

  const handlePointerMove = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      if (pointerIdRef.current !== event.pointerId || !dragStartRef.current) {
        return;
      }

      event.preventDefault();
      onSelectionChange(buildSelection(dragStartRef.current, readPoint(event)));
    },
    [onSelectionChange],
  );

  const handlePointerUp = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      if (pointerIdRef.current !== event.pointerId) {
        return;
      }

      event.preventDefault();
      finishSelection(event);
    },
    [finishSelection],
  );

  const handleConfirmSelection = useCallback(() => {
    if (selection) {
      onConfirmSelection(selection);
    }
  }, [onConfirmSelection, selection]);

  const stopPanelPointerEvent = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      event.stopPropagation();
    },
    [],
  );

  const hintText = selection
    ? `${selectionLabel} ${selection.width} x ${selection.height}`
    : idleHint;

  if (!active) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 overflow-hidden bg-slate-950/65 text-white">
      <div
        className="absolute inset-0 cursor-crosshair select-none touch-none"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onPointerCancel={clearSelection}
      >
        <div
          className="absolute left-4 top-4 w-[calc(100vw-2rem)] max-w-md rounded-md border border-white/15 bg-slate-950/85 px-4 py-3 shadow-lg shadow-slate-950/30"
          onPointerDown={stopPanelPointerEvent}
          onPointerMove={stopPanelPointerEvent}
          onPointerUp={stopPanelPointerEvent}
        >
          <div className="flex items-start gap-3">
            <div
              className={`mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border ${
                isDragging
                  ? 'border-cyan-300/60 bg-cyan-400/15 text-cyan-100'
                  : 'border-slate-700 bg-slate-900 text-cyan-200'
              }`}
            >
              <Crop className="h-4 w-4" />
            </div>
            <div className="min-w-0">
              <h2 className="text-sm font-semibold text-white">{title}</h2>
              <p className="mt-1 text-xs leading-5 text-slate-300">
                {description}
              </p>
              <p className="mt-2 text-xs font-medium uppercase tracking-[0.18em] text-cyan-200">
                {hintText}
              </p>
            </div>
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <CropOverlayButton
              icon={<X className="h-4 w-4" />}
              label={cancelLabel}
              onClick={onCancel}
            />
            <CropOverlayButton
              icon={<RotateCcw className="h-4 w-4" />}
              label={retryLabel}
              onClick={onRetryRecognition}
            />
            <CropOverlayButton
              icon={<Check className="h-4 w-4" />}
              label={confirmLabel}
              onClick={handleConfirmSelection}
              disabled={!selection}
              disabledTitle={confirmDisabledHint}
              emphasis="primary"
            />
          </div>
        </div>

        {selection ? (
          <div
            className={`absolute border-2 ${
              isDragging
                ? 'border-dashed border-cyan-300 bg-cyan-400/10'
                : 'border-cyan-200 bg-cyan-400/15'
            } shadow-[0_0_0_1px_rgba(34,211,238,0.25)]`}
            style={{
              left: selection.x,
              top: selection.y,
              width: selection.width,
              height: selection.height,
            }}
          >
            <div className="absolute -right-px -bottom-px h-3 w-3 border-b-2 border-r-2 border-cyan-100/80" />
          </div>
        ) : null}
      </div>
    </div>
  );
}

function CropOverlayButton({
  icon,
  label,
  onClick,
  disabled = false,
  disabledTitle,
  emphasis = 'secondary',
}: {
  icon: ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  disabledTitle?: string;
  emphasis?: 'secondary' | 'primary';
}) {
  const tone =
    emphasis === 'primary'
      ? 'border-cyan-200 bg-cyan-200 text-slate-950 hover:bg-cyan-100'
      : 'border-white/15 bg-white/10 text-slate-100 hover:bg-white/15';

  return (
    <button
      type="button"
      className={`inline-flex min-h-9 items-center gap-2 rounded-md border px-3 py-2 text-xs font-medium transition disabled:cursor-not-allowed disabled:opacity-45 ${tone}`}
      onClick={onClick}
      disabled={disabled}
      title={disabled ? disabledTitle : label}
    >
      {icon}
      {label}
    </button>
  );
}
