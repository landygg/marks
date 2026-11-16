// Entry of the hidden `parser` window: Rust passes it downloaded HTML and reads the result.
// It imports nothing from Tauri, and the window has no capabilities.
import { cleanHtml, extractHtml, type Extracted } from 'marks-extract';

// Rust turns `error` into a readable failure and destroys the window without waiting.
function guarded<T>(run: () => T): T | { error: string } {
  try {
    return run();
  } catch (e) {
    return { error: String(e) };
  }
}

function marksExtractHtml(html: string, url: string): Extracted | { error: string } {
  return guarded(() => extractHtml(html, url));
}

function marksCleanHtml(html: string, url: string): string | { error: string } {
  return guarded(() => cleanHtml(html, url));
}

Object.assign(globalThis, { marksExtractHtml, marksCleanHtml, marksReady: true });
