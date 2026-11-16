import type { Extracting } from '../../api';

/** Whether bookmark `id` of `profile` is being extracted; ids repeat across profiles. */
export const isExtracting = (list: Extracting[], profile: number, id: number): boolean =>
  list.some((e) => e.profile === profile && e.id === id);
