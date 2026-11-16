import { describe, expect, it } from 'vitest';
import type { Bookmark } from '../../api';
import { detailMeta, isAiExtracted, isBlocked, rowSummary } from './detailMeta';

const base = {
  summary: null,
  description: null,
  author: null,
  site: null,
  published: null,
  word_count: null,
  status: 'ready',
  error_kind: null,
} as unknown as Bookmark;
const mk = (p: Partial<Bookmark>): Bookmark => ({ ...base, ...p });

describe('detailMeta', () => {
  it('joins only the parts that exist', () => {
    expect(detailMeta(base)).toEqual([]);
    expect(detailMeta(mk({ site: 'Blog', published: 'ayer' }))).toEqual(['Blog', 'ayer']);
    expect(
      detailMeta(mk({ site: 'Blog', author: 'Ana', published: '2026-11-09T12:00:00Z' })),
    ).toEqual(['Blog', 'Ana', '9 nov 2026']);
  });
  it('rounds reading time up, with a minimum of one, and none for zero words', () => {
    expect(detailMeta(mk({ word_count: 450 }))).toEqual(['3 min de lectura']);
    expect(detailMeta(mk({ word_count: 5 }))).toEqual(['1 min de lectura']);
    expect(detailMeta(mk({ word_count: 0 }))).toEqual([]);
  });
});

describe('rowSummary', () => {
  it('prefers the summary and falls back to the description', () => {
    expect(rowSummary(mk({ summary: 's', description: 'd' }))).toBe('s');
    expect(rowSummary(mk({ description: 'd' }))).toBe('d');
    expect(rowSummary(base)).toBeNull();
  });
});

describe('isBlocked', () => {
  it('needs both failed and blocked', () => {
    expect(isBlocked(mk({ status: 'failed', error_kind: 'blocked' }))).toBe(true);
    expect(isBlocked(mk({ status: 'failed' }))).toBe(false);
    expect(isBlocked(mk({ status: 'pending', error_kind: 'blocked' }))).toBe(false);
  });
});

describe('isAiExtracted', () => {
  it('is true only for content an AI model extracted', () => {
    expect(isAiExtracted(mk({ extractor: 'ai:qwen3:8b' }))).toBe(true);
    expect(isAiExtracted(mk({ extractor: 'defuddle@0.19.6' }))).toBe(false);
    expect(isAiExtracted(mk({ extractor: null }))).toBe(false);
  });
});
