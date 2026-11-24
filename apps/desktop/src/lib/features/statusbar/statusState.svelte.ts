// The AI and local API status: global, not per profile, so no refresh guard.
import { appStatus, type AppStatus } from '../../api';
import { report } from '../library/libraryState.svelte';
import { coalesce } from '../library/refreshGuard';

export const app = $state<{ status: AppStatus | null }>({ status: null });

/** Reloads the status; calls made while it runs share one more run. */
export const refreshAppStatus = coalesce(async () => {
  try {
    app.status = await appStatus();
  } catch (err) {
    report(err, { toast: false, profile: null });
  }
});
