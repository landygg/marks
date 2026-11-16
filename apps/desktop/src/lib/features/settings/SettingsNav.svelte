<script lang="ts">
  import { closeSettings, lib, openSettings } from '../library/libraryState.svelte';
  import { SETTINGS_GROUPS } from './settingsLogic';
  import { t } from '../../strings';
  import SidebarItem from '../../ui/SidebarItem.svelte';

  const profileName = $derived(lib.profiles.find((p) => p.id === lib.profileId)?.name ?? '');
</script>

<nav>
  <SidebarItem icon="chevronLeft" onclick={closeSettings}>{t.backToLibrary}</SidebarItem>
  {#each SETTINGS_GROUPS as group (group.scope)}
    <div class="heading">
      <span class="section-label"
        >{group.scope === 'app' ? t.settingsApp : t.settingsProfile(profileName)}</span
      >
    </div>
    {#each group.sections as section (section)}
      <SidebarItem
        page
        selected={lib.settingsSection === section}
        onclick={() => openSettings(section)}
      >
        {t.settingsSections[section]}
      </SidebarItem>
    {/each}
  {/each}
</nav>

<style>
  nav {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 var(--space-sm) var(--space-sm);
  }

  /* Lines the label up with the items' text; the label itself keeps its one look. */
  .heading {
    padding: 0 var(--space-xs);
  }
</style>
