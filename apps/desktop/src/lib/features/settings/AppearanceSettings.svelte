<script lang="ts">
  import CornerSetting from './CornerSetting.svelte';
  import LanguageSetting from './LanguageSetting.svelte';
  import SettingRows from '../../ui/SettingRows.svelte';
  import ThemeSetting from './ThemeSetting.svelte';
</script>

<SettingRows>
  <ThemeSetting />
  <LanguageSetting />
  <CornerSetting />
</SettingRows>
