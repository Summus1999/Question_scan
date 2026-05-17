/**
 * ResultPanel 组件测试。
 *
 * 覆盖：空闲状态、流式/加载状态、完成状态、失败状态、
 *       操作按钮显隐、复制/清空/重新生成/切换语言交互。
 */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ResultPanel } from './ResultPanel';

/** 默认消息文本配置，用于测试渲染。 */
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

/** 测试辅助函数：渲染 ResultPanel 并填充默认 props。 */
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
  /** 空闲状态下显示空状态提示文案。 */
  it('shows idle hint when state is idle', () => {
    renderPanel({ state: 'idle' });

    const emptyState = screen.getByTestId('result-empty-state');
    expect(emptyState).toBeInTheDocument();
    expect(emptyState).toHaveTextContent('Press shortcut to capture');
  });

  /** 流式状态下显示生成中文案和已输出内容。 */
  it('shows generating status during streaming', () => {
    renderPanel({ state: 'streaming', displayedText: 'partial output' });

    expect(screen.getByText('Generating...')).toBeInTheDocument();
    expect(screen.getByText('partial output')).toBeInTheDocument();
  });

  /** 完成状态下显示完成状态标识。 */
  it('shows complete status when done', () => {
    renderPanel({
      state: 'complete',
      fullText: 'final answer',
      displayedText: 'final answer',
    });

    expect(screen.getByText('Complete')).toBeInTheDocument();
  });

  /** 失败状态下显示错误信息和失败标识。 */
  it('shows error message when state is failed', () => {
    renderPanel({
      state: 'failed',
      error: 'API key invalid',
    });

    expect(screen.getByText('Failed')).toBeInTheDocument();
    expect(screen.getByText('API key invalid')).toBeInTheDocument();
  });

  /** 有内容时显示复制代码、复制全文和清空按钮。 */
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

  /** 空闲状态下隐藏操作按钮。 */
  it('hides action buttons when idle', () => {
    renderPanel({ state: 'idle' });

    expect(
      screen.queryByRole('button', { name: 'Copy code' }),
    ).not.toBeInTheDocument();
  });

  /** 点击复制代码按钮触发 onCopyCode 回调。 */
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

  /** 点击复制全文按钮触发 onCopyFullAnswer 回调。 */
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

  /** 点击清空按钮触发 onClearResult 回调。 */
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

  /** 点击重新生成按钮触发 onRegenerate 回调。 */
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

  /** 点击切换语言按钮弹出语言选择菜单。 */
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

  /** 选择语言后触发 onSwitchLanguage 回调并传入对应语言 ID。 */
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

  /** 失败状态下显示重新生成按钮。 */
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
