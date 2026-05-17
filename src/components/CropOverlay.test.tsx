/**
 * CropOverlay 组件测试。
 *
 * 覆盖：inactive 渲染、拖拽归一化、视口边界限制、pointer cancel 清空、
 *       取消/重试动作、确认裁剪条件。
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { CropSelectionRect } from '../lib/types';
import { CropOverlay } from './CropOverlay';

/** 测试辅助组件：管理选区状态，模拟真实使用场景。 */
function CropOverlayHarness({
  onSelectionComplete,
  onCancel = vi.fn(),
  onRetryRecognition = vi.fn(),
  onConfirmSelection = vi.fn(),
}: {
  onSelectionComplete: (selection: CropSelectionRect | null) => void;
  onCancel?: () => void;
  onRetryRecognition?: () => void;
  onConfirmSelection?: (selection: CropSelectionRect) => void;
}) {
  const [selection, setSelection] = useState<CropSelectionRect | null>(null);

  return (
    <CropOverlay
      active
      title="Manual selection"
      description="Drag across the question area."
      idleHint="Drag to start a selection"
      selectionLabel="Selected region"
      cancelLabel="Cancel"
      retryLabel="Retry auto recognition"
      confirmLabel="Confirm crop"
      confirmDisabledHint="Select a question region before confirming."
      selection={selection}
      onSelectionChange={setSelection}
      onSelectionComplete={onSelectionComplete}
      onCancel={onCancel}
      onRetryRecognition={onRetryRecognition}
      onConfirmSelection={onConfirmSelection}
    />
  );
}

describe('CropOverlay', () => {
  /** 非激活状态下不渲染任何 DOM 节点。 */
  it('does not render while inactive', () => {
    const { container } = render(
      <CropOverlay
        active={false}
        title="Manual selection"
        description="Drag across the question area."
        idleHint="Drag to start a selection"
        selectionLabel="Selected region"
        cancelLabel="Cancel"
        retryLabel="Retry auto recognition"
        confirmLabel="Confirm crop"
        confirmDisabledHint="Select a question region before confirming."
        selection={null}
        onSelectionChange={vi.fn()}
        onCancel={vi.fn()}
        onRetryRecognition={vi.fn()}
        onConfirmSelection={vi.fn()}
      />,
    );

    expect(container).toBeEmptyDOMElement();
  });

  /** 拖拽操作应归一化为正向矩形，并触发 onSelectionComplete 回调。 */
  it('normalizes a drag into a visible crop selection', () => {
    const onSelectionComplete = vi.fn();
    render(<CropOverlayHarness onSelectionComplete={onSelectionComplete} />);

    const overlay = screen.getByLabelText('Manual selection');
    fireEvent.pointerDown(overlay, {
      button: 0,
      clientX: 420,
      clientY: 260,
      pointerId: 1,
    });
    fireEvent.pointerMove(overlay, {
      clientX: 120,
      clientY: 80,
      pointerId: 1,
    });

    expect(screen.getByText('Selected region 300 x 180')).toBeInTheDocument();

    fireEvent.pointerUp(overlay, {
      clientX: 120,
      clientY: 80,
      pointerId: 1,
    });

    expect(onSelectionComplete).toHaveBeenCalledWith({
      x: 120,
      y: 80,
      width: 300,
      height: 180,
    });
  });

  /** 超出视口的坐标应被限制在视口边界内。 */
  it('clamps drag coordinates to the viewport bounds', () => {
    const onSelectionComplete = vi.fn();
    render(<CropOverlayHarness onSelectionComplete={onSelectionComplete} />);

    const overlay = screen.getByLabelText('Manual selection');
    fireEvent.pointerDown(overlay, {
      button: 0,
      clientX: -40,
      clientY: -20,
      pointerId: 3,
    });
    fireEvent.pointerMove(overlay, {
      clientX: window.innerWidth + 40,
      clientY: window.innerHeight + 20,
      pointerId: 3,
    });

    expect(
      screen.getByText(
        `Selected region ${window.innerWidth} x ${window.innerHeight}`,
      ),
    ).toBeInTheDocument();

    fireEvent.pointerUp(overlay, {
      clientX: window.innerWidth + 40,
      clientY: window.innerHeight + 20,
      pointerId: 3,
    });

    expect(onSelectionComplete).toHaveBeenCalledWith({
      x: 0,
      y: 0,
      width: window.innerWidth,
      height: window.innerHeight,
    });
  });

  /** pointer cancel 事件应清空当前选区并回调 null。 */
  it('clears an in-progress selection when pointer capture is canceled', () => {
    const onSelectionComplete = vi.fn();
    render(<CropOverlayHarness onSelectionComplete={onSelectionComplete} />);

    const overlay = screen.getByLabelText('Manual selection');
    fireEvent.pointerDown(overlay, {
      button: 0,
      clientX: 100,
      clientY: 120,
      pointerId: 4,
    });
    fireEvent.pointerMove(overlay, {
      clientX: 220,
      clientY: 260,
      pointerId: 4,
    });

    expect(screen.getByText('Selected region 120 x 140')).toBeInTheDocument();

    fireEvent.pointerCancel(overlay, {
      clientX: 220,
      clientY: 260,
      pointerId: 4,
    });

    expect(onSelectionComplete).toHaveBeenCalledWith(null);
    expect(screen.getByText('Drag to start a selection')).toBeInTheDocument();
  });

  /** 取消和重试按钮应正确触发对应回调。 */
  it('exposes cancel and retry actions while selection is active', () => {
    const onCancel = vi.fn();
    const onRetryRecognition = vi.fn();

    render(
      <CropOverlayHarness
        onSelectionComplete={vi.fn()}
        onCancel={onCancel}
        onRetryRecognition={onRetryRecognition}
      />,
    );

    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));
    fireEvent.click(
      screen.getByRole('button', { name: 'Retry auto recognition' }),
    );

    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(onRetryRecognition).toHaveBeenCalledTimes(1);
  });

  /** 未选区时确认按钮禁用，选区后点击触发 onConfirmSelection。 */
  it('requires a selected region before confirming the crop', () => {
    const onConfirmSelection = vi.fn();
    render(
      <CropOverlayHarness
        onSelectionComplete={vi.fn()}
        onConfirmSelection={onConfirmSelection}
      />,
    );

    expect(screen.getByRole('button', { name: 'Confirm crop' })).toBeDisabled();

    const overlay = screen.getByLabelText('Manual selection');
    fireEvent.pointerDown(overlay, {
      button: 0,
      clientX: 40,
      clientY: 50,
      pointerId: 2,
    });
    fireEvent.pointerUp(overlay, {
      clientX: 180,
      clientY: 150,
      pointerId: 2,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Confirm crop' }));

    expect(onConfirmSelection).toHaveBeenCalledWith({
      x: 40,
      y: 50,
      width: 140,
      height: 100,
    });
  });
});
