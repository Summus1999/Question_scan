import { useCallback, useEffect, useRef, useState } from 'react';
import type { OutputSpeed } from '../lib/types';

// ------------------------------------------------------------------------------
// Speed configuration: characters per reveal tick
// ------------------------------------------------------------------------------

const SPEED_CONFIG: Record<Exclude<OutputSpeed, 'custom'>, number> = {
  fast: 48, // ~48 chars per tick
  normal: 24, // ~24 chars per tick
  slow: 8, // ~8 chars per tick
};

const TICK_MS = 50; // Base tick interval

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

// ------------------------------------------------------------------------------
// useTypewriter hook
// ------------------------------------------------------------------------------

export interface UseTypewriterOptions {
  speed: OutputSpeed;
  customCharactersPerSecond: number;
}

export interface UseTypewriterReturn {
  /** The full accumulated text (always complete). */
  fullText: string;
  /** The text currently displayed (may be partial during typing). */
  displayedText: string;
  /** Whether the typewriter is actively revealing text. */
  isTyping: boolean;
  /** Append raw text to the buffer. */
  append: (text: string) => void;
  /** Reset all text and stop typing. */
  reset: () => void;
  /** Instantly reveal all buffered text. */
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
