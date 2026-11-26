<script lang="ts">
  import { parseTags } from '../../format';
  import { saveBookmark } from './libraryState.svelte';
  import { ENTER_KEY, isComposing } from '../../shortcuts';
  import { t } from '../../strings';
  import Tag from '../../ui/Tag.svelte';
  import TagField from '../../ui/TagField.svelte';

  interface Props {
    /** The profile the bookmark was loaded in; fixed for the editor's life (`Detail` keys on it). */
    profile: number;
    id: number;
    tags: string[];
  }

  let { profile, id, tags }: Props = $props();

  let input = $state('');
  const inputId = $props.id();

  async function addTags(event: KeyboardEvent): Promise<void> {
    if (isComposing(event) || event.key !== ENTER_KEY) return;
    const parsed = parseTags(input);
    if (parsed.length === 0) return;
    // Duplicates are dropped by the backend's `normalize_tags`.
    if (await saveBookmark(profile, id, { tags: [...tags, ...parsed] })) input = '';
  }

  const removeTag = (tag: string) =>
    void saveBookmark(profile, id, { tags: tags.filter((x) => x !== tag) });
</script>

<section aria-labelledby="{inputId}-heading">
  <h2 class="section-label" id="{inputId}-heading">{t.tagsLabel}</h2>
  <ul class="chips">
    {#each tags as tag (tag)}
      <li><Tag removeLabel={t.removeTag(tag)} onremove={() => removeTag(tag)}>{tag}</Tag></li>
    {/each}
    <li>
      <TagField
        aria-label={t.addTagsLabel}
        placeholder={t.addTagsPlaceholder}
        bind:value={input}
        onkeydown={addTags}
      />
    </li>
  </ul>
</section>

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
