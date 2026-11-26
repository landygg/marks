import { mount } from 'svelte';
import { localeInfo } from './lib/api';
import { setLocale } from './lib/i18n';
import { applyToDocument, storedTheme } from './lib/features/settings/theme';
import './app.css';

// Before anything awaits, so a pinned theme does not flash the system one.
applyToDocument(storedTheme());

// App is imported only after `setLocale`, so no module reads `t` before the locale is set.
async function start() {
  const target = document.getElementById('app');
  if (!target) throw new Error('#app element missing from index.html'); // i18n-allow: invariant
  try {
    setLocale((await localeInfo()).active);
  } catch {
    // Same fallback as the backend.
    setLocale('en');
  }
  const { default: App } = await import('./App.svelte');
  mount(App, { target });
}

void start();
