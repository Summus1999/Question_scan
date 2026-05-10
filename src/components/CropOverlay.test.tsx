import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { CropSelectionRect } from '../lib/types';
import { CropOverlay } from './CropOverlay';

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
