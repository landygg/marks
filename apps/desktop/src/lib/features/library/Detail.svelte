<script lang="ts">
  import { bookmarkFavicon, get } from '../../api';
  import { detailMeta, isAiExtracted, isBlocked } from './detailMeta';
  import { isExtracting } from './extracting';
  import ContentView from './ContentView.svelte';
  import NotesEditor from './NotesEditor.svelte';
  import TagEditor from './TagEditor.svelte';
  import TitleEditor from './TitleEditor.svelte';
  import {
    askDeleteBookmark,
    askReextract,
    deleteBookmark,
    lib,
    openLink,
    report,
    retryBookmark,
    runReextract,
    toggleExpanded,
  } from './libraryState.svelte';
  import { t } from '../../strings';
  import SidebarToggle from '../layout/SidebarToggle.svelte';
  import { layout } from '../layout/layoutState.svelte';
  import Button from '../../ui/Button.svelte';
  import ClampText from '../../ui/ClampText.svelte';
  import CopyButton from '../../ui/CopyButton.svelte';
  import DetailUrl from '../../ui/DetailUrl.svelte';
  import DetailNotice from '../../ui/DetailNotice.svelte';
  import EmptyState from '../../ui/EmptyState.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Favicon from '../../ui/Favicon.svelte';
  import StatusDot from '../../ui/StatusDot.svelte';
  import Topbar from '../../ui/Topbar.svelte';

  const bookmark = $derived(lib.bookmarks.find((b) => b.id === lib.selectedId));

  const busy = $derived(
    bookmark ? isExtracting(lib.extracting, lib.profileId, bookmark.id) : false,
  );

  // The list ships without `content`; the full copy is fetched here, tagged with its profile and id.
  let loaded = $state<{ profile: number; id: number; content: string | null } | null>(null);
  const content = $derived(
    loaded && loaded.profile === lib.profileId && loaded.id === bookmark?.id
      ? loaded.content
      : null,
  );

  // Reloads when the bookmark changes or is rewritten (edits, worker progress bump `updated_at`).
  $effect(() => {
    if (!bookmark) return;
    const { id } = bookmark;
    const profile = lib.profileId;
    void bookmark.updated_at;
    let stale = false;
    get(profile, id)
      .then((full) => {
        if (!stale) loaded = { profile, id, content: full?.content ?? null };
      })
      .catch((err) => report(err, { toast: false }));
    return () => (stale = true);
  });

  // The stored favicon, tagged like `loaded`. Reloaded only when the worker stores it.
  let favicon = $state<{ profile: number; id: number; src: string | null } | null>(null);
  const faviconSrc = $derived(
    favicon && favicon.profile === lib.profileId && favicon.id === bookmark?.id
      ? favicon.src
      : null,
  );
  const faviconOf = $derived(bookmark?.id);
  const faviconAt = $derived(bookmark?.favicon_at);

  $effect(() => {
    if (faviconOf === undefined) return;
    const id = faviconOf;
    const profile = lib.profileId;
    void faviconAt;
    let stale = false;
    bookmarkFavicon(profile, id)
      .then((src) => {
        if (!stale) favicon = { profile, id, src };
      })
      .catch((err) => report(err, { toast: false }));
    return () => (stale = true);
  });

  const meta = $derived(bookmark ? detailMeta(bookmark) : []);
</script>

<section class="detail" data-tauri-drag-region>
  <Topbar lights={layout.collapsed && lib.expanded}>
    {#snippet end()}
      {#if bookmark}
        <IconButton
          icon="external"
          label={t.openLink}
          onclick={() => openLink(bookmark.original_url)}
        />
        {#if bookmark.status === 'failed'}
          <IconButton
            icon="retry"
            label={t.retry}
            onclick={() => retryBookmark(lib.profileId, bookmark.id)}
          />
        {/if}
        <!-- An imported bookmark has nothing to re-extract yet: it is queued, as a failed one is retried. -->
        {#if bookmark.status === 'imported'}
          <IconButton
            icon="reextract"
            label={t.extract}
            onclick={() => retryBookmark(lib.profileId, bookmark.id)}
          />
        {:else if !isBlocked(bookmark)}
          <!-- Not for a blocked page: it is rescued from the browser extension, where the visitor got past the check. -->
          <IconButton
            icon="reextract"
            label={t.reextract}
            disabled={busy}
            onclick={() => askReextract('defuddle')}
          />
          <IconButton
            icon="sparkles"
            label={t.extractWithAi}
            reason={lib.extractionModel ? undefined : t.extractWithAiUnavailable}
            disabled={busy}
            onclick={() => askReextract('ai')}
          />
        {/if}
        <IconButton icon="trash" label={t.delete} danger onclick={askDeleteBookmark} />
      {/if}
      <!-- The view controls, last and apart from the bookmark's actions: Expand, then the sidebar toggle. -->
      <div class="view">
        <IconButton
          icon={lib.expanded ? 'collapse' : 'expand'}
          label={lib.expanded ? t.collapse : t.expand}
          onclick={toggleExpanded}
        />
        <SidebarToggle />
      </div>
    {/snippet}
  </Topbar>

  <Panel>
    {#if bookmark}
      {#if lib.confirmReextract}
        {@const kind = lib.confirmReextract}
        <div class="confirm">
          <InlineConfirm
            question={t.confirmReextract}
            label={t.confirmReextract}
            yes={t.confirmReextractYes}
            cancel={t.cancel}
            onyes={() => runReextract(lib.profileId, bookmark.id, kind)}
            oncancel={() => (lib.confirmReextract = null)}
          />
        </div>
      {/if}
      {#if lib.confirmDelete}
        <div class="confirm">
          <InlineConfirm
            question={t.confirmDelete}
            label={t.confirmDelete}
            yes={t.confirmDeleteYes}
            cancel={t.cancel}
            onyes={() => deleteBookmark(lib.profileId, bookmark.id)}
            oncancel={() => (lib.confirmDelete = false)}
          />
        </div>
      {/if}

      <div class="scroll">
        <article>
          <!-- Fresh editors per bookmark and profile (ids repeat across profiles), so a pending title
               or notes can never be saved into another bookmark. -->
          {#key `${lib.profileId}:${bookmark.id}`}
            <div class="title">
              <TitleEditor profile={lib.profileId} id={bookmark.id} saved={bookmark.title} />
            </div>
            {#if bookmark.description}
              <p class="detail-description">{bookmark.description}</p>
            {/if}
            <div class="line">
              <div class="grow"><DetailUrl url={bookmark.original_url} onopen={openLink} /></div>
              <CopyButton
                text={bookmark.original_url}
                label={t.copyLink}
                copiedLabel={t.linkCopied}
              />
            </div>
            {#if meta.length || faviconSrc}
              <p class="detail-meta">
                <Favicon src={faviconSrc} />
                <span>{meta.join(t.separator)}</span>
              </p>
            {/if}
            <p class="status">
              <StatusDot status={busy ? 'extracting' : bookmark.status} />
              <span
                >{busy
                  ? t.extracting
                  : t.status[
                      bookmark.status
                    ]}{#if bookmark.category}{t.separator}{bookmark.category}{/if}{#if !busy && isAiExtracted(bookmark)}{t.separator}{t.extractedWithAi}{/if}</span
              >
            </p>
            {#if !busy && bookmark.status === 'failed' && bookmark.error}
              <!-- On its own line under the status, so a long message wraps instead of running off. -->
              <div class="line message error-text">
                <div class="grow">
                  <ClampText text={bookmark.error} moreLabel={t.showMore} lessLabel={t.showLess} />
                </div>
                <CopyButton
                  text={bookmark.error}
                  label={t.copyMessage}
                  copiedLabel={t.messageCopied}
                />
              </div>
            {/if}

            {#if isBlocked(bookmark) && !busy}
              <div class="blocked">
                <DetailNotice message={t.blocked.notice}>
                  <Button onclick={() => openLink(bookmark.original_url)}>{t.openLink}</Button>
                </DetailNotice>
              </div>
            {/if}

            {#if bookmark.summary}
              <h2 class="section-label">{t.summaryLabel}</h2>
              <p class="summary">{bookmark.summary}</p>
            {/if}

            <TagEditor profile={lib.profileId} id={bookmark.id} tags={bookmark.tags} />
            <NotesEditor profile={lib.profileId} id={bookmark.id} saved={bookmark.notes} />
            <ContentView {content} />
          {/key}
        </article>
      </div>
    {:else}
      <EmptyState>{t.selectBookmark}</EmptyState>
    {/if}
  </Panel>
</section>

<style>
  .detail {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    padding: 0 var(--gap) var(--gap) 0;
  }

  /* `sm` from the other icons, counting the strip's own gap; the strip's gap between the two. */
  .view {
    display: flex;
    gap: var(--space-xxs);
    margin-left: calc(var(--space-sm) - var(--space-xxs));
  }

  .confirm {
    display: flex;
    flex: none;
    padding: var(--space-sm) var(--space-xl);
    border-bottom: 1px solid var(--border);
  }

  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  /* Content column: `content` plus the padding on each side. */
  article {
    box-sizing: border-box;
    max-width: calc(var(--content) + 2 * var(--space-xl));
    margin: 0 auto;
    padding: var(--space-xl);
  }

  /* Pulled left by the field's own inset, so the title text lines up with the column. */
  .title {
    margin: 0 var(--space-sm) var(--space-xs) calc(-1 * var(--space-sm));
  }

  /* A text and its copy button; the button sits on the first line's centre. */
  .line {
    display: flex;
    align-items: flex-start;
    gap: var(--space-xs);
    min-width: 0;
  }

  .grow {
    display: flex;
    flex: 1;
    min-width: 0;
    padding-top: var(--space-xs);
  }

  /* Under the status words, past the dot, tucked against the status line. */
  .message {
    margin: calc(-1 * var(--space-sm)) 0 var(--space-md) calc(var(--dot) + var(--space-sm));
  }

  .status {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    margin: var(--space-sm) 0 var(--space-md);
    color: var(--muted);
    font: var(--type-label-md);
  }

  /* `detail-meta` and `detail-description` (DESIGN.md, Components). */
  .detail-meta {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    margin: var(--space-xs) 0 0;
    color: var(--muted);
    font: var(--type-label-md);
  }

  .detail-meta span {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail-description {
    margin: 0 0 var(--space-xs);
    color: var(--muted);
    font: var(--type-body-md);
  }

  .blocked {
    margin-bottom: var(--space-md);
  }

  .summary {
    margin: 0;
    font: var(--type-body-lg);
  }
</style>
