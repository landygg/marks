<script lang="ts">
  import AiSettingsForm from './AiSettingsForm.svelte';
  import ExtensionPairing from './ExtensionPairing.svelte';
  import { lib } from '../library/libraryState.svelte';
  import MaintenanceActions from './MaintenanceActions.svelte';
  import ProfileSettings from './ProfileSettings.svelte';
  import { SETTINGS_GROUPS, SETTINGS_SECTIONS, type SettingsSection } from './settingsLogic';
  import { t } from '../../strings';
  import AppearanceSettings from './AppearanceSettings.svelte';
  import TransferSettings from './TransferSettings.svelte';
  import UpdateSettings from './UpdateSettings.svelte';
  import Panel from '../../ui/Panel.svelte';
  import Topbar from '../../ui/Topbar.svelte';
  import SidebarToggle from '../layout/SidebarToggle.svelte';
  import { layout } from '../layout/layoutState.svelte';

  const CONTENT = {
    appearance: AppearanceSettings,
    ai: AiSettingsForm,
    updates: UpdateSettings,
    profile: ProfileSettings,
    extension: ExtensionPairing,
    transfer: TransferSettings,
    maintenance: MaintenanceActions,
  };
  const PROFILE_SECTIONS: readonly SettingsSection[] =
    SETTINGS_GROUPS.find((g) => g.scope === 'profile')?.sections ?? [];
</script>

<Topbar lights={layout.collapsed}>
  {#snippet title()}{t.settings}{t.separator}<b>{t.settingsSections[lib.settingsSection]}</b
    >{/snippet}
  {#snippet end()}<SidebarToggle />{/snippet}
</Topbar>

<Panel>
  <div class="body">
    <!-- Every section stays mounted, so an unsaved form or a result survives a visit to another one.
         A profile's sections are mounted again on a switch: what they hold belongs to one profile. -->
    {#each SETTINGS_SECTIONS as section (section)}
      {@const Content = CONTENT[section]}
      <section hidden={lib.settingsSection !== section}>
        <h1>{t.settingsSections[section]}</h1>
        {#key PROFILE_SECTIONS.includes(section) ? lib.profileId : null}
          <Content />
        {/key}
      </section>
    {/each}
  </div>
</Panel>

<style>
  .body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--space-xl);
  }

  section {
    max-width: var(--content);
    margin: 0 auto;
  }

  h1 {
    margin: 0 0 var(--space-lg);
    font: var(--type-headline-lg);
    letter-spacing: var(--tracking-headline-lg);
  }
</style>
