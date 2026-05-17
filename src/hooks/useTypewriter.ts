/**
 * 打字机效果 Hook（阶段 7）。
 *
 * 职责：模拟打字机逐字显示效果，用于 AI 流式输出的视觉呈现。
 * 支持：三种预设速度（快/中/慢）、自定义速度、追加文本、重置、立即显示全部。
 * 原理：使用 setInterval 定期从缓冲区向显示区追加字符。
 */
import { useCallback, useEffect, useRef, useState } from 'react';
import type { OutputSpeed } from '../lib/types';

/** 速度配置：每次 tick 显示的字符数。 */
const SPEED_CONFIG: Record<Exclude<OutputSpeed, 'custom'>, number> = {
  fast: 48, // ~48 chars per tick
  normal: 24, // ~24 chars per tick
  slow: 8, // ~8 chars per tick
};

const TICK_MS = 50; // Base tick interval

/** 根据速度设置计算每次 tick 显示的字符数。 */
function getCharsPerTick(
  speed: OutputSpeed,
  customCharsPerSecond: number,
): number {
  if (speed === 'custom') {
    // Convert chars/second to chars/tick
    return Math.max(1, Math.round((customCharsPerSecond * TICK_MS) / 1000));
  }
  return SPEED_CONFIG[speed];
}

/** useTypewriter Hook 选项和返回值接口。 */
export interface UseTypewriterOptions {
  speed: OutputSpeed;
  customCharactersPerSecond: number;
}

export interface UseTypewriterReturn {
  /** 完整累积文本（始终完整）。 */
  fullText: string;
  /** 当前显示的文本（打字过程中可能不完整）。 */
  displayedText: string;
  /** 打字机是否正在活跃显示文本。 */
  isTyping: boolean;
  /** 追加原始文本到缓冲区。 */
  append: (text: string) => void;
  /** 重置所有文本并停止打字。 */
  reset: () => void;
  /** 立即显示缓冲区中的所有文本。 */
  revealAll: () => void;
}

export function useTypewriter({
  speed,
  customCharactersPerSecond,
}: UseTypewriterOptions): UseTypewriterReturn {
  const [fullText, setFullText] = useState('');
  const [displayedText, setDisplayedText] = useState('');
  const [isTyping, setIsTyping] = useState(false);

  const bufferRef = useRef('');
  const displayedRef = useRef('');
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const charsPerTickRef = useRef(
    getCharsPerTick(speed, customCharactersPerSecond),
  );

  // Update charsPerTick when speed config changes
  useEffect(() => {
    charsPerTickRef.current = getCharsPerTick(speed, customCharactersPerSecond);
  }, [speed, customCharactersPerSecond]);

  const stopTimer = useCallback(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const tick = useCallback(() => {
    const buffer = bufferRef.current;
    const displayed = displayedRef.current;

    if (displayed.length >= buffer.length) {
      stopTimer();
      setIsTyping(false);
      return;
    }

    const charsPerTick = charsPerTickRef.current;
    const nextEnd = Math.min(buffer.length, displayed.length + charsPerTick);
    const nextText = buffer.slice(0, nextEnd);

    displayedRef.current = nextText;
    setDisplayedText(nextText);

    if (nextEnd >= buffer.length) {
      stopTimer();
      setIsTyping(false);
    }
  }, [stopTimer]);

  const startTimer = useCallback(() => {
    stopTimer();
    setIsTyping(true);
    timerRef.current = setInterval(tick, TICK_MS);
  }, [stopTimer, tick]);

  const append = useCallback(
    (text: string) => {
      if (!text) return;

      bufferRef.current += text;
      setFullText((prev) => prev + text);

      // Start or keep the timer running if there's more to reveal
      if (displayedRef.current.length < bufferRef.current.length) {
        if (!timerRef.current) {
          startTimer();
        }
      }
    },
    [startTimer],
  );

  const reset = useCallback(() => {
    stopTimer();
    bufferRef.current = '';
    displayedRef.current = '';
    setFullText('');
    setDisplayedText('');
    setIsTyping(false);
  }, [stopTimer]);

  const revealAll = useCallback(() => {
    stopTimer();
    const buffer = bufferRef.current;
    displayedRef.current = buffer;
    setDisplayedText(buffer);
    setIsTyping(false);
  }, [stopTimer]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      stopTimer();
    };
  }, [stopTimer]);

  return {
    fullText,
    displayedText,
    isTyping,
    append,
    reset,
    revealAll,
  };
}
