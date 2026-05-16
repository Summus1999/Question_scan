import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ResultPanel } from './ResultPanel';

const defaultMessages = {
  title: 'Result',
  idleHint: 'Press shortcut to capture',
  screenshotStatus: 'Capturing...',
  recognizingStatus: 'Recognizing...',
  generatingStatus: 'Generating...',
  completeStatus: 'Complete',
  failedStatus: 'Failed',
  copyCode: 'Copy code',
  copyCodeSuccess: 'Code copied',
  copyFullAnswer: 'Copy full answer',
  copyFullAnswerSuccess: 'Answer copied',
  clearResult: 'Clear',
  regenerate: 'Regenerate',
  switchLanguage: 'Switch language',
  codeCopied: 'Code copied',
  answerCopied: 'Answer copied',
  resultCleared: 'Result cleared',
};

function renderPanel(props: Partial<Parameters<typeof ResultPanel>[0]> = {}) {
  const defaults: Parameters<typeof ResultPanel>[0] = {
    state: 'idle',
    fullText: '',
    displayedText: '',
    error: null,
    currentLanguage: 'cpp20',
    messages: defaultMessages,
    onCopyCode: vi.fn(),
    onCopyFullAnswer: vi.fn(),
    onClearResult: vi.fn(),
    onRegenerate: vi.fn(),
    onSwitchLanguage: vi.fn(),
  };

  return render(<ResultPanel {...defaults} {...props} />);
}

describe('ResultPanel', () => {
  it('shows idle hint when state is idle', () => {
    renderPanel({ state: 'idle' });

    const emptyState = screen.getByTestId('result-empty-state');
    expect(emptyState).toBeInTheDocument();
    expect(emptyState).toHaveTextContent('Press shortcut to capture');
  });

  it('shows generating status during streaming', () => {
    renderPanel({ state: 'streaming', displayedText: 'partial output' });

    expect(screen.getByText('Generating...')).toBeInTheDocument();
    expect(screen.getByText('partial output')).toBeInTheDocument();
  });

  it('shows complete status when done', () => {
    renderPanel({
      state: 'complete',
      fullText: 'final answer',
      displayedText: 'final answer',
    });

    expect(screen.getByText('Complete')).toBeInTheDocument();
  });

  it('shows error message when state is failed', () => {
    renderPanel({
      state: 'failed',
      error: 'API key invalid',
    });

    expect(screen.getByText('Failed')).toBeInTheDocument();
    expect(screen.getByText('API key invalid')).toBeInTheDocument();
  });

  it('shows action buttons when content is available', () => {
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
    });

    expect(
      screen.getByRole('button', { name: 'Copy code' }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('button', { name: 'Copy full answer' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Clear' })).toBeInTheDocument();
  });

  it('hides action buttons when idle', () => {
    renderPanel({ state: 'idle' });

    expect(
      screen.queryByRole('button', { name: 'Copy code' }),
    ).not.toBeInTheDocument();
  });

  it('calls onCopyCode when copy code button is clicked', () => {
    const onCopyCode = vi.fn();
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
      onCopyCode,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Copy code' }));
    expect(onCopyCode).toHaveBeenCalledTimes(1);
  });

  it('calls onCopyFullAnswer when copy full answer button is clicked', () => {
    const onCopyFullAnswer = vi.fn();
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
      onCopyFullAnswer,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Copy full answer' }));
    expect(onCopyFullAnswer).toHaveBeenCalledTimes(1);
  });

  it('calls onClearResult when clear button is clicked', () => {
    const onClearResult = vi.fn();
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
      onClearResult,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Clear' }));
    expect(onClearResult).toHaveBeenCalledTimes(1);
  });

  it('calls onRegenerate when regenerate button is clicked', () => {
    const onRegenerate = vi.fn();
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
      onRegenerate,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Regenerate' }));
    expect(onRegenerate).toHaveBeenCalledTimes(1);
  });

  it('shows language switch menu when switch language is clicked', () => {
    const onSwitchLanguage = vi.fn();
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
      onSwitchLanguage,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Switch language' }));

    expect(screen.getByText('Python')).toBeInTheDocument();
    expect(screen.getByText('Java')).toBeInTheDocument();
  });

  it('calls onSwitchLanguage when a language is selected', () => {
    const onSwitchLanguage = vi.fn();
    renderPanel({
      state: 'complete',
      fullText: 'answer',
      displayedText: 'answer',
      onSwitchLanguage,
    });

    fireEvent.click(screen.getByRole('button', { name: 'Switch language' }));
    fireEvent.click(screen.getByText('Python'));

    expect(onSwitchLanguage).toHaveBeenCalledWith('python');
  });

  it('shows regenerate button for failed state', () => {
    renderPanel({
      state: 'failed',
      error: 'Something went wrong',
    });

    expect(
      screen.getByRole('button', { name: 'Regenerate' }),
    ).toBeInTheDocument();
  });
});
