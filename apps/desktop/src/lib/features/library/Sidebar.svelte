<script lang="ts">
  import { tick, untrack } from 'svelte';
  import type { Attachment } from 'svelte/attachments';
  import SettingsNav from '../settings/SettingsNav.svelte';
  import {
    CATEGORIES_KEY,
    UNGROUPED_KEY,
    cancelDeleteGroup,
    cancelGroupEdit,
    choose,
    deleteGroup,
    lib,
    moveTag,
    startNewGroup,
    submitGroupEdit,
    toggleCollapsed,
  } from './libraryState.svelte';
  import {
    ALL,
    FAILED,
    sameSelection,
    sidebarGroups,
    type Selection,
    type SidebarGroup as Group,
  } from './sidebar';
  import { SIDEBAR, SIDEBAR_ID } from '../layout/layout';
  import { layout, setWidth } from '../layout/layoutState.svelte';
  import { ENTER_KEY, ESCAPE_KEY } from '../../shortcuts';
  import { t } from '../../strings';
  import { showGroupMenu, showTagMenu } from './tagMenu';
  import IconButton from '../../ui/IconButton.svelte';
  import InlineConfirm from '../../ui/InlineConfirm.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Resizer from '../../ui/Resizer.svelte';
  import SidebarGroup from '../../ui/SidebarGroup.svelte';
  import SidebarHeading from '../../ui/SidebarHeading.svelte';
  import SidebarItem from '../../ui/SidebarItem.svelte';
  import TextField from '../../ui/TextField.svelte';
  import Topbar from '../../ui/Topbar.svelte';
  import Wordmark from '../../ui/Wordmark.svelte';

  interface Props {
    /** Drawn width, which can be under the saved one in a narrow window. */
    width: number;
    /** The widest the detail leaves room for now. */
    max: number;
    /** A drag of the handle starts or ends. */
    onresizing: (active: boolean) => void;
  }

  let { width, max, onresizing }: Props = $props();

  /** Marks our own drags: anything else dragged over the sidebar carries no such type and is refused. */
  const TAG_MIME = 'application/x-marks-tag';
  const MOVE = 'move';

  const groups = $derived(sidebarGroups(lib.tags, lib.groups, t.noGroup));
  const groupKey = (id: number | null): string => (id === null ? UNGROUPED_KEY : String(id));

  // The tag being dragged and the group it came from.
  let dragged = $state<{ tag: string; from: number | null } | null>(null);
  // The zone under the pointer, and how many of its elements the pointer is inside: `dragleave`
  // also fires when moving between children, so the zone is left only when the count returns to 0.
  let dropKey = $state<string | null>(null);
  let dropDepth = 0;

  // What the inline field starts with when it opens: empty for a new group, the current name for a
  // rename; typing overrides it. `untrack`: a refresh replaces `lib.groups` and must not wipe what is
  // being typed.
  let groupName = $derived.by(() => {
    const edit = lib.groupEdit;
    return untrack(() =>
      edit?.id == null ? '' : (lib.groups.find((g) => g.id === edit.id)?.name ?? ''),
    );
  });
  let nameInput = $state<HTMLInputElement>();
  // The edit whose last submit failed: leaving its field then cancels instead of submitting the same
  // name again. Each opening is a new edit, so it starts clean.
  let failedEdit: typeof lib.groupEdit = null;

  function startDrag(event: DragEvent, tag: string, from: number | null): void {
    if (!event.dataTransfer) return;
    event.dataTransfer.setData(TAG_MIME, tag);
    event.dataTransfer.effectAllowed = MOVE;
    dragged = { tag, from };
  }

  // Also bound to the window: if the dragged element is removed mid-drag its own `dragend` is lost.
  function endDrag(): void {
    dragged = null;
    dropKey = null;
    dropDepth = 0;
  }

  /** On a tag's button: a refresh that removes it mid-drag ends the drag, since a detached node never gets `dragend`. */
  const endDragOnRemoval =
    (tag: string): Attachment =>
    () =>
    () => {
      if (dragged?.tag === tag) endDrag();
    };

  /** The group accepts the drag: it is one of our tags and it is not already in that group. */
  function accepts(event: DragEvent, group: Group): boolean {
    const ours = event.dataTransfer?.types.includes(TAG_MIME) ?? false;
    return ours && dragged !== null && dragged.from !== group.id;
  }

  function enter(event: DragEvent, group: Group): void {
    if (!accepts(event, group)) return;
    const key = groupKey(group.id);
    dropDepth = dropKey === key ? dropDepth + 1 : 1;
    dropKey = key;
  }

  function over(event: DragEvent, group: Group): void {
    if (!accepts(event, group)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = MOVE;
  }

  function leave(group: Group): void {
    if (dropKey !== groupKey(group.id)) return;
    dropDepth -= 1;
    if (dropDepth <= 0) dropKey = null;
  }

  function drop(event: DragEvent, group: Group): void {
    if (!accepts(event, group) || !dragged) return;
    event.preventDefault();
    void moveTag(lib.profileId, dragged.tag, group.id);
    endDrag();
  }

  const inSettings = $derived(lib.view === 'settings');
  const profileName = $derived(lib.profiles.find((p) => p.id === lib.profileId)?.name);

  function contextMenu(event: MouseEvent, open: () => Promise<void>): void {
    event.preventDefault();
    void open();
  }

  // `autofocus` only works while nothing else has focus; this field can open from the palette or a
  // menu. After `tick`, so a modal that is closing in the same update no longer holds the focus.
  function focusNow(node: HTMLInputElement): void {
    void tick().then(() => {
      node.focus();
      node.select();
      node.scrollIntoView({ block: 'nearest' });
    });
  }

  async function submitName(): Promise<void> {
    const edit = lib.groupEdit;
    const result = await submitGroupEdit(groupName);
    // "busy": an earlier submit is still running and will report its own result.
    if (result === 'busy') return;
    // Another field opened while this request ran: its outcome is not that field's business.
    if (result === 'failed' && lib.groupEdit !== edit) return;
    failedEdit = result === 'failed' ? edit : null;
    // Refused after the focus already left: nothing would close the field, so it closes now.
    if (failedEdit && document.activeElement !== nameInput) cancelGroupEdit();
  }

  function onNameKey(event: KeyboardEvent): void {
    if (event.key === ENTER_KEY) void submitName();
    if (event.key === ESCAPE_KEY) {
      // Handled here: the window's Esc (closing the notification panel) leaves it alone.
      event.preventDefault();
      cancelGroupEdit();
    }
  }

  // Leaving the field keeps what was typed; an empty field, or a name that was just refused, is a cancel.
  function onNameBlur(): void {
    if (!lib.groupEdit) return;
    if (groupName.trim() === '' || failedEdit === lib.groupEdit) cancelGroupEdit();
    else void submitName();
  }
</script>

<svelte:window ondragend={endDrag} ondrop={endDrag} />

{#snippet item(selection: Selection, label: string, nested: boolean)}
  <SidebarItem
    {nested}
    selected={lib.view === 'library' && sameSelection(lib.selection, selection)}
    onclick={() => choose(selection)}
  >
    {label}
  </SidebarItem>
{/snippet}

{#snippet nameField()}
  <TextField
    bare
    mono
    aria-label={t.groupNameLabel}
    placeholder={t.groupNameLabel}
    bind:ref={nameInput}
    bind:value={groupName}
    oninput={() => (failedEdit = null)}
    onkeydown={onNameKey}
    onblur={onNameBlur}
    {@attach focusNow}
  />
{/snippet}

<aside
  id={SIDEBAR_ID}
  aria-label={inSettings ? t.settingsNavLabel : t.sidebarLabel}
  data-tauri-drag-region
>
  <Topbar lights>
    {#snippet start()}
      <Wordmark>{t.appTitle}</Wordmark>
    {/snippet}
  </Topbar>
  <div class="body">
    <Panel>
      <SidebarHeading title={profileName ?? ''}>
        {#snippet end()}
          {#if !inSettings}
            <IconButton icon="folderPlus" label={t.newGroup} onclick={() => startNewGroup()} />
          {/if}
        {/snippet}
      </SidebarHeading>

      <!-- Settings takes over the navigation (DESIGN.md, Ajustes); the library's stays mounted to keep its state. -->
      {#if inSettings}<SettingsNav />{/if}
      <nav hidden={inSettings}>
        {@render item(ALL, t.allBookmarks, false)}
        <!-- Kept while selected, so the emptied filter after "Retry all" shows its message. -->
        {#if (lib.stats?.failed ?? 0) > 0 || lib.selection.kind === 'failed'}
          {@render item(FAILED, t.failedBookmarks, false)}
        {/if}

        {#if lib.categories.length > 0}
          <SidebarGroup
            label={t.categoriesLabel}
            expanded={!lib.collapsed[CATEGORIES_KEY]}
            onclick={() => toggleCollapsed(CATEGORIES_KEY)}
          />
          {#if !lib.collapsed[CATEGORIES_KEY]}
            {#each lib.categories as value (value)}
              {@render item({ kind: 'category', value }, value, true)}
            {/each}
          {/if}
        {/if}

        {#each groups as group (group.id)}
          {@const id = group.id}
          <!-- A new group goes at the end of the user's groups, before the ungrouped tags. -->
          {#if id === null && lib.groupEdit && lib.groupEdit.id === null}
            <SidebarGroup editing field={nameField} />
          {/if}
          <!-- The ungrouped zone also shows while dragging, as the place to drop a tag out of its group. -->
          {#if id !== null || group.tags.length > 0 || dragged}
            {@const key = groupKey(id)}
            <div
              role="group"
              aria-label={group.name}
              ondragenter={(e) => enter(e, group)}
              ondragover={(e) => over(e, group)}
              ondragleave={() => leave(group)}
              ondrop={(e) => drop(e, group)}
            >
              <!-- The question sits right under its highlighted heading: the name is not repeated. -->
              <SidebarGroup
                label={group.name}
                expanded={!lib.collapsed[key]}
                drop={dropKey === key}
                deleting={id !== null && lib.groupDeleteId === id}
                editing={id !== null && lib.groupEdit?.id === id}
                field={nameField}
                hint={t.confirmDeleteGroupHint}
                onclick={() => toggleCollapsed(key)}
                oncontextmenu={id === null
                  ? undefined
                  : (e) => contextMenu(e, () => showGroupMenu(id, lib.profileId))}
              >
                {#snippet confirm()}
                  {#if id !== null}
                    <InlineConfirm
                      compact
                      question={t.confirmDeleteGroup}
                      label={t.confirmDeleteGroupLabel(group.name, t.confirmDeleteGroupHint)}
                      yes={t.confirmDeleteGroupYes}
                      cancel={t.cancel}
                      onyes={() => deleteGroup(lib.profileId, id)}
                      oncancel={cancelDeleteGroup}
                    />
                  {/if}
                {/snippet}
              </SidebarGroup>
              {#if !lib.collapsed[key]}
                {#each group.tags as tag (tag)}
                  <!-- The grip says the tag can be dragged; the same move is in the context menu and the palette. -->
                  <SidebarItem
                    nested
                    grip
                    dragging={dragged?.tag === tag}
                    selected={lib.view === 'library' &&
                      sameSelection(lib.selection, { kind: 'tag', value: tag })}
                    draggable="true"
                    title={t.dragHint}
                    onclick={() => choose({ kind: 'tag', value: tag })}
                    oncontextmenu={(e) =>
                      contextMenu(e, () => showTagMenu(tag, id, lib.groups, lib.profileId))}
                    ondragstart={(e) => startDrag(e, tag, id)}
                    ondragend={endDrag}
                    {@attach endDragOnRemoval(tag)}
                  >
                    {tag}
                  </SidebarItem>
                {/each}
              {/if}
            </div>
          {/if}
        {/each}
      </nav>
    </Panel>
    {#if !layout.collapsed}
      <Resizer
        label={t.layout.sidebarWidth}
        value={width}
        min={SIDEBAR.min}
        {max}
        reset={SIDEBAR.reset}
        controls={SIDEBAR_ID}
        onchange={(v) => setWidth('sidebar', v)}
        onstart={() => onresizing(true)}
        onend={() => onresizing(false)}
      />
    {/if}
  </div>
</aside>

<style>
  /* The `sidebar` zone: its top strip takes the frame's colour, like the rail and the list's strip.
     Its own width, so the content keeps it while the column narrows; the gap is on its right. */
  aside {
    display: flex;
    flex-direction: column;
    width: var(--sidebar);
    min-height: 0;
    padding: 0 var(--gap) var(--gap) 0;
    background: var(--bg);
    /* Labels are controls, not text: a press-and-move must start a tag drag, never a text selection. */
    -webkit-user-select: none;
    user-select: none;
  }

  /* The handle sits over the gap to the right of the panel, from its top edge to its bottom one. */
  .body {
    position: relative;
    display: flex;
    flex: 1;
    min-height: 0;
  }

  nav {
    flex: 1;
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    padding: 0 var(--space-sm) var(--space-sm);
  }
</style>
