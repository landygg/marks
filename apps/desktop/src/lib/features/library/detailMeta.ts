import type { Bookmark } from '../../api';
import { formatDate } from '../../format';
import { t } from '../../strings';

export const WORDS_PER_MINUTE = 200;

const ISO_DATE = /^\d{4}-\d{2}-\d{2}/;

/** Parts of the metadata line: site, author, date (formatted when ISO, as-is otherwise), reading time. */
export function detailMeta(b: Bookmark): string[] {
  const published =
    b.published && ISO_DATE.test(b.published) && !Number.isNaN(Date.parse(b.published))
      ? formatDate(b.published)
      : b.published;
  const minutes = b.word_count ? Math.max(1, Math.ceil(b.word_count / WORDS_PER_MINUTE)) : null;
  return [b.site, b.author, published, minutes && t.readingTime(minutes)].filter(
    (part): part is string => !!part,
  );
}

export function rowSummary(b: Bookmark): string | null {
  return b.summary ?? b.description;
}

export function isBlocked(b: Bookmark): boolean {
  return b.status === 'failed' && b.error_kind === 'blocked';
}

/** Same value as `AI_EXTRACTOR_PREFIX` in crates/core/src/pipeline.rs. */
export const AI_EXTRACTOR_PREFIX = 'ai:';

/** Whether an AI model extracted the content ("Extract with AI"). */
export const isAiExtracted = (b: Bookmark): boolean =>
  b.extractor?.startsWith(AI_EXTRACTOR_PREFIX) ?? false;
