<!-- `sidebar-toggle` (DESIGN.md, Components): folds and unfolds the sidebar, also with ⌃⌘S. -->
<script lang="ts">
  import IconButton from '../../ui/IconButton.svelte';
  import { t } from '../../strings';
  import { SIDEBAR_ID } from './layout';
  import { layout, toggleSidebar } from './layoutState.svelte';
</script>

<IconButton
  icon="sidebar"
  label={layout.collapsed ? t.layout.showSidebar : t.layout.hideSidebar}
  title={layout.collapsed ? t.layout.showSidebarTitle : t.layout.hideSidebarTitle}
  aria-expanded={!layout.collapsed}
  aria-controls={SIDEBAR_ID}
  data-sidebar-toggle
  onclick={toggleSidebar}
/>
