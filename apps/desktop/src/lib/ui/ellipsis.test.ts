import { describe, expect, it } from 'vitest';
import { TAIL_CHARS, splitTail } from './ellipsis';

const LONG =
  'https://signin.example-cloud.com/signin?redirect_uri=https%3A%2F%2Fconsole.example-cloud.com&client_id=urn%3Aexample&code_challenge_method=SHA-256';

describe('splitTail', () => {
  it('keeps the start in the head and the last characters in the tail', () => {
    const { head, tail } = splitTail(LONG);
    expect(head.startsWith('https://signin.example-cloud.com/signin')).toBe(true);
    expect(tail).toBe(LONG.slice(-TAIL_CHARS));
    expect(tail.endsWith('code_challenge_method=SHA-256')).toBe(true);
  });

  it('loses nothing: head and tail make the original', () => {
    const { head, tail } = splitTail(LONG);
    expect(head + tail).toBe(LONG);
  });

  it('leaves a short URL whole', () => {
    expect(splitTail('https://bun.sh/docs')).toEqual({ head: 'https://bun.sh/docs', tail: '' });
    const edge = 'x'.repeat(TAIL_CHARS * 2);
    expect(splitTail(edge)).toEqual({ head: edge, tail: '' });
  });

  it('never cuts an emoji or an accent in two', () => {
    const family = '👨‍👩‍👧';
    const text = `https://example.com/${'a'.repeat(10)}${family}é${'b'.repeat(2)}`;
    const { head, tail } = splitTail(text, 4);
    expect(tail).toBe(`${family}ébb`);
    expect(head + tail).toBe(text);
  });

  it('splits a long unicode path by characters, not code units', () => {
    const text = `https://例え.jp/${'日本語'.repeat(30)}`;
    const { tail } = splitTail(text);
    expect(Array.from(tail)).toHaveLength(TAIL_CHARS);
    expect(text.endsWith(tail)).toBe(true);
  });
});
