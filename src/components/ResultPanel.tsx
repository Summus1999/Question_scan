/**
 * 结果面板组件（阶段 7）。
 *
 * 职责：展示 AI 生成结果的状态、内容和操作工具栏。
 * 包含：状态指示器（空闲/加载/流式/完成/失败）、Markdown 渲染（含代码高亮）、
 *       操作工具栏（复制代码、复制全文、清空、重新生成、切换语言）。
 */
import {
  CheckCircle2,
  Copy,
  Eraser,
  FileCode2,
  Languages,
  Loader2,
  RefreshCw,
  XCircle,
} from 'lucide-react';
import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react';
import ReactMarkdown from 'react-markdown';
import type { AiResultState, LanguageId } from '../lib/types';
import { LANGUAGE_OPTIONS } from '../lib/types';

/**
 * 结果面板的消息文本配置接口。
 * 所有显示文本通过 props 传入，便于国际化。
 */
export interface ResultPanelMessages {
  title: string;
  idleHint: string;
  screenshotStatus: string;
  recognizingStatus: string;
  generatingStatus: string;
  completeStatus: string;
  failedStatus: string;
  copyCode: string;
  copyCodeSuccess: string;
  copyFullAnswer: string;
  copyFullAnswerSuccess: string;
  clearResult: string;
  regenerate: string;
  switchLanguage: string;
  codeCopied: string;
  answerCopied: string;
  resultCleared: string;
}

export interface ResultPanelProps {
  state: AiResultState;
  fullText: string;
  displayedText: string;
  error: string | null;
  currentLanguage: LanguageId;
  messages: ResultPanelMessages;
  onCopyCode: () => void;
  onCopyFullAnswer: () => void;
  onClearResult: () => void;
  onRegenerate: () => void;
  onSwitchLanguage: (language: LanguageId) => void;
}

/** 代码块渲染组件：使用 Prism.js 进行语法高亮，支持懒加载语言组件。 */
function CodeBlock({
  language,
  children,
}: {
  language: string;
  children: string;
}) {
  const codeRef = useRef<HTMLElement>(null);
  const [highlighted, setHighlighted] = useState(false);

  useEffect(() => {
    let cancelled = false;
    async function highlight() {
      try {
        const Prism = (await import('prismjs')).default;
        // Load common languages
        await import('prismjs/components/prism-clike');
        await import('prismjs/components/prism-c');
        await import('prismjs/components/prism-cpp');
        await import('prismjs/components/prism-python');
        await import('prismjs/components/prism-java');
        await import('prismjs/components/prism-javascript');
        await import('prismjs/components/prism-typescript');
        await import('prismjs/components/prism-go');
        await import('prismjs/components/prism-rust');
        await import('prismjs/components/prism-markdown');
        if (cancelled) return;
        if (codeRef.current) {
          Prism.highlightElement(codeRef.current);
          setHighlighted(true);
        }
      } catch {
        // Highlighting is best-effort; fall back to plain text
      }
    }
    void highlight();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <pre className="overflow-auto rounded-md border border-slate-200 bg-slate-900 px-4 py-3 text-sm leading-6">
      <code
        ref={codeRef}
        className={`language-${language}`}
        style={{ color: highlighted ? undefined : '#e2e8f0' }}
      >
        {children}
      </code>
    </pre>
  );
}

/** Markdown 组件覆盖：自定义代码块和内联代码的渲染行为。 */
const markdownComponents = {
  code({
    inline,
    className,
    children,
    ...props
  }: {
    inline?: boolean;
    className?: string;
    children: ReactNode;
  }) {
    const match = /language-(\w+)/.exec(className || '');
    const language = match?.[1] ?? 'text';
    const codeText = String(children).replace(/\n$/, '');

    if (inline) {
      return (
        <code
          className="rounded bg-slate-100 px-1.5 py-0.5 text-sm font-mono text-slate-800"
          {...props}
        >
          {children}
        </code>
      );
    }

    return <CodeBlock language={language}>{codeText}</CodeBlock>;
  },
  pre({ children }: { children: ReactNode }) {
    // Let the code component handle its own pre wrapper
    return <>{children}</>;
  },
};

/** 状态指示器：根据 AI 结果状态显示对应的图标和文本。 */
function StatusIndicator({
  state,
  messages,
}: {
  state: AiResultState;
  messages: ResultPanelMessages;
}) {
  const config: Record<
    AiResultState,
    { icon: ReactNode; text: string; color: string }
  > = {
    idle: {
      icon: <FileCode2 className="h-4 w-4" />,
      text: messages.idleHint,
      color: 'text-slate-500',
    },
    loading: {
      icon: <Loader2 className="h-4 w-4 animate-spin" />,
      text: messages.generatingStatus,
      color: 'text-amber-600',
    },
    streaming: {
      icon: <Loader2 className="h-4 w-4 animate-spin" />,
      text: messages.generatingStatus,
      color: 'text-amber-600',
    },
    complete: {
      icon: <CheckCircle2 className="h-4 w-4" />,
      text: messages.completeStatus,
      color: 'text-emerald-600',
    },
    failed: {
      icon: <XCircle className="h-4 w-4" />,
      text: messages.failedStatus,
      color: 'text-rose-600',
    },
  };

  const { icon, text, color } = config[state];

  return (
    <div className={`flex items-center gap-2 text-sm font-medium ${color}`}>
      {icon}
      <span>{text}</span>
    </div>
  );
}

/** 操作工具栏：提供复制、清空、重新生成、切换语言等操作按钮。 */
function ActionToolbar({
  state,
  messages,
  currentLanguage,
  onCopyCode,
  onCopyFullAnswer,
  onClearResult,
  onRegenerate,
  onSwitchLanguage,
}: {
  state: AiResultState;
  messages: ResultPanelMessages;
  currentLanguage: LanguageId;
  onCopyCode: () => void;
  onCopyFullAnswer: () => void;
  onClearResult: () => void;
  onRegenerate: () => void;
  onSwitchLanguage: (language: LanguageId) => void;
}) {
  const hasContent = state === 'complete' || state === 'streaming';
  const [showLangMenu, setShowLangMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setShowLangMenu(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleLanguageSelect = useCallback(
    (lang: LanguageId) => {
      setShowLangMenu(false);
      onSwitchLanguage(lang);
    },
    [onSwitchLanguage],
  );

  return (
    <div className="flex flex-wrap items-center gap-2">
      {hasContent && (
        <>
          <ToolbarButton
            icon={<Copy className="h-3.5 w-3.5" />}
            label={messages.copyCode}
            onClick={onCopyCode}
          />
          <ToolbarButton
            icon={<FileCode2 className="h-3.5 w-3.5" />}
            label={messages.copyFullAnswer}
            onClick={onCopyFullAnswer}
          />
          <ToolbarButton
            icon={<Eraser className="h-3.5 w-3.5" />}
            label={messages.clearResult}
            onClick={onClearResult}
          />
        </>
      )}

      {(state === 'complete' || state === 'failed') && (
        <ToolbarButton
          icon={<RefreshCw className="h-3.5 w-3.5" />}
          label={messages.regenerate}
          onClick={onRegenerate}
        />
      )}

      {hasContent && (
        <div className="relative" ref={menuRef}>
          <ToolbarButton
            icon={<Languages className="h-3.5 w-3.5" />}
            label={messages.switchLanguage}
            onClick={() => setShowLangMenu((v) => !v)}
          />
          {showLangMenu && (
            <div className="absolute right-0 z-10 mt-1 w-40 rounded-md border border-slate-200 bg-white py-1 shadow-lg">
              {LANGUAGE_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  type="button"
                  className={`block w-full px-3 py-1.5 text-left text-sm transition hover:bg-slate-50 ${
                    option.value === currentLanguage
                      ? 'font-medium text-emerald-600'
                      : 'text-slate-700'
                  }`}
                  onClick={() => handleLanguageSelect(option.value)}
                >
                  {option.label}
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/** 工具栏按钮：统一的图标按钮样式。 */
function ToolbarButton({
  icon,
  label,
  onClick,
}: {
  icon: ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center gap-1.5 rounded-md border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-600 transition hover:bg-slate-50 hover:text-slate-900"
    >
      {icon}
      {label}
    </button>
  );
}

/** 结果面板主组件：整合状态、内容和操作工具栏。 */
export function ResultPanel({
  state,
  fullText,
  displayedText,
  error,
  currentLanguage,
  messages,
  onCopyCode,
  onCopyFullAnswer,
  onClearResult,
  onRegenerate,
  onSwitchLanguage,
}: ResultPanelProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const isStreaming = state === 'streaming' || state === 'loading';

  // Auto-scroll to bottom during streaming
  useEffect(() => {
    if (isStreaming && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [isStreaming]);

  const showEmpty = state === 'idle';
  const showError = state === 'failed' && error;
  const showContent = state === 'streaming' || state === 'complete';

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-200 pb-3">
        <StatusIndicator state={state} messages={messages} />
        <ActionToolbar
          state={state}
          messages={messages}
          currentLanguage={currentLanguage}
          onCopyCode={onCopyCode}
          onCopyFullAnswer={onCopyFullAnswer}
          onClearResult={onClearResult}
          onRegenerate={onRegenerate}
          onSwitchLanguage={onSwitchLanguage}
        />
      </div>

      {/* Content area */}
      <div
        ref={scrollRef}
        className="mt-3 flex-1 overflow-auto rounded-md border border-slate-200 bg-white"
      >
        {showEmpty && (
          <div
            data-testid="result-empty-state"
            className="flex h-48 items-center justify-center text-sm text-slate-400"
          >
            {messages.idleHint}
          </div>
        )}

        {showError && (
          <div className="p-4">
            <div className="rounded-md border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-800">
              {error}
            </div>
          </div>
        )}

        {showContent && (
          <div className="p-4">
            <div className="prose prose-sm max-w-none prose-headings:text-slate-900 prose-p:text-slate-700 prose-strong:text-slate-900 prose-li:text-slate-700">
              <ReactMarkdown components={markdownComponents}>
                {displayedText}
              </ReactMarkdown>
            </div>
            {isStreaming && (
              <span className="ml-1 inline-block h-4 w-0.5 animate-pulse bg-amber-500" />
            )}
          </div>
        )}
      </div>

      {/* Hidden full text for clipboard access */}
      <span className="sr-only" aria-live="polite">
        {fullText}
      </span>
    </div>
  );
}
