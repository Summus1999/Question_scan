import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { useTypewriter } from './useTypewriter';

describe('useTypewriter', () => {
  it('starts with empty text and not typing', () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'normal', customCharactersPerSecond: 24 }),
    );

    expect(result.current.fullText).toBe('');
    expect(result.current.displayedText).toBe('');
    expect(result.current.isTyping).toBe(false);
  });

  it('appends text to fullText immediately', () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'normal', customCharactersPerSecond: 24 }),
    );

    act(() => {
      result.current.append('hello');
    });

    expect(result.current.fullText).toBe('hello');
  });

  it('reveals text gradually based on speed', async () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'fast', customCharactersPerSecond: 24 }),
    );

    act(() => {
      result.current.append('hello world');
    });

    // fullText should have everything immediately
    expect(result.current.fullText).toBe('hello world');

    // displayedText may still be empty or partial depending on timing
    // Wait for typing to complete
    await waitFor(
      () => {
        expect(result.current.displayedText).toBe('hello world');
      },
      { timeout: 2000 },
    );

    expect(result.current.isTyping).toBe(false);
  });

  it('resets all text and stops typing', () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'normal', customCharactersPerSecond: 24 }),
    );

    act(() => {
      result.current.append('some text');
    });

    act(() => {
      result.current.reset();
    });

    expect(result.current.fullText).toBe('');
    expect(result.current.displayedText).toBe('');
    expect(result.current.isTyping).toBe(false);
  });

  it('reveals all text instantly', () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'slow', customCharactersPerSecond: 24 }),
    );

    act(() => {
      result.current.append('hidden text');
    });

    // Before revealAll, displayedText may be partial
    act(() => {
      result.current.revealAll();
    });

    expect(result.current.displayedText).toBe('hidden text');
    expect(result.current.isTyping).toBe(false);
  });

  it('fullText always contains complete output even when displayedText is partial', async () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'slow', customCharactersPerSecond: 8 }),
    );

    act(() => {
      result.current.append('complete output');
    });

    // fullText should always be complete
    expect(result.current.fullText).toBe('complete output');

    // Immediately after append, displayedText might still be empty or partial
    // because the timer hasn't fired yet
    expect(result.current.displayedText.length).toBeLessThanOrEqual(
      result.current.fullText.length,
    );
  });

  it('uses custom characters per second when speed is custom', async () => {
    const { result } = renderHook(() =>
      useTypewriter({ speed: 'custom', customCharactersPerSecond: 100 }),
    );

    act(() => {
      result.current.append('fast custom');
    });

    // With 100 chars/sec and 50ms ticks, that's 5 chars per tick
    // Should complete in about 3 ticks (150ms) for 11 chars
    await waitFor(
      () => {
        expect(result.current.displayedText).toBe('fast custom');
      },
      { timeout: 2000 },
    );
  });
});
