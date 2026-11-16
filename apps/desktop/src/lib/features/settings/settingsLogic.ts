// Pure helpers for the settings form.
import type { ExtractionSettings, Provider, Stats, Target } from '../../api';

/** How long "Settings saved." stays after a save. */
export const SAVED_NOTICE_MS = 3000;

/** AI settings are global: their failures show in every profile and lead back to Settings → AI. */
export const AI_GLOBAL: { profile: null; target: Target } = {
  profile: null,
  target: { kind: 'ai' },
};

/** Same value as `OLLAMA_DEFAULT_ENDPOINT` in crates/core/src/settings.rs. */
export const OLLAMA_DEFAULT_ENDPOINT = 'http://localhost:11434';

/** Mirrors `Provider::requires_endpoint` in crates/core/src/settings.rs. */
const ENDPOINT_REQUIRED: readonly Provider[] = ['omlx', 'openai_compatible'];

export const requiresEndpoint = (provider: Provider): boolean =>
  ENDPOINT_REQUIRED.includes(provider);

/** The endpoint the backend uses when the field is left blank; empty when there is none. */
export const endpointPlaceholder = (provider: Provider): string =>
  provider === 'ollama' ? OLLAMA_DEFAULT_ENDPOINT : '';

/** The values of the AI switch; `Segmented` needs strings. */
export const AI_MODES = ['on', 'off'] as const;
export type AiMode = (typeof AI_MODES)[number];

export const aiMode = (enabled: boolean): AiMode => (enabled ? 'on' : 'off');

/** The inverse of `aiMode`: whether the switch value turns the AI on. */
export const enabledFor = (mode: AiMode): boolean => mode === 'on';

/** Argument for `saveSettings`: a typed key is stored, `""` deletes the stored one, `null` keeps it. */
export function keyToSend(input: string, clearKey: boolean): string | null {
  if (input !== '') return input;
  return clearKey ? '' : null;
}

/** Providers whose servers are always outside this machine. */
const CLOUD_PROVIDERS: readonly Provider[] = ['openai', 'anthropic'];
/** Host names of this machine, as `URL.hostname` gives them. */
const LOCAL_HOSTS = ['localhost', '127.0.0.1', '[::1]'];

/** Whether requests leave this machine: a cloud provider, or an endpoint on another host. */
export function isRemoteEndpoint(provider: Provider, endpoint: string | null): boolean {
  if (CLOUD_PROVIDERS.includes(provider)) return true;
  const url = endpoint?.trim() || endpointPlaceholder(provider);
  if (!url) return false;
  try {
    return !LOCAL_HOSTS.includes(new URL(url).hostname);
  } catch {
    return false;
  }
}

/** The values of the extraction switch: the summary settings or its own. */
export const EXTRACTION_MODES = ['same', 'own'] as const;
export type ExtractionMode = (typeof EXTRACTION_MODES)[number];

export const extractionMode = (sameAsSummary: boolean): ExtractionMode =>
  sameAsSummary ? 'same' : 'own';

export const sameAsSummaryFor = (mode: ExtractionMode): boolean => mode === 'same';

/** Arguments for `saveExtractionSettings`: with the summary settings, the key is left alone. */
export function extractionToSave(
  form: ExtractionSettings,
  apiKey: string,
  clearKey: boolean,
): { settings: ExtractionSettings; apiKey: string | null } {
  return {
    settings: form,
    apiKey: form.same_as_summary ? null : keyToSend(apiKey, clearKey),
  };
}

/** Bookmarks in `ready`: `stats.pending` already counts both `pending` and `pending_embed`. */
export const readyCount = (stats: Stats): number =>
  stats.total - stats.pending - stats.failed - stats.imported;

/** The sections of Settings, in the order the sidebar lists them. */
export const SETTINGS_SECTIONS = [
  'appearance',
  'ai',
  'updates',
  'profile',
  'extension',
  'transfer',
  'maintenance',
] as const;

/** The sidebar's two headings: what applies to the whole app, and what belongs to the active profile. */
export const SETTINGS_GROUPS = [
  { scope: 'app', sections: ['appearance', 'ai', 'updates'] },
  { scope: 'profile', sections: ['profile', 'extension', 'transfer', 'maintenance'] },
] as const;

export type SettingsSection = (typeof SETTINGS_SECTIONS)[number];

/** Where new versions are published; Settings → Updates links to it when this Mac does not update itself. */
export const RELEASES_URL = 'https://github.com/landygg/marks/releases';
/** The newest release, where Settings → Browser extension sends you for the Chromium zip. */
export const LATEST_RELEASE_URL = `${RELEASES_URL}/latest`;

const pad2 = (n: number): string => String(n).padStart(2, '0');

/**
 * `marks-<profile>-<yyyy-mm-dd>.html`, the name the save dialog suggests; the date is local. A name
 * with no letter or digit leaves the profile out.
 */
export function exportFileName(profileName: string, date: Date): string {
  // NFC first: a decomposed accent is a separate mark, which the slug would turn into a dash.
  const slug = profileName
    .normalize('NFC')
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, '-')
    .replace(/^-|-$/g, '');
  const day = `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`;
  return slug ? `marks-${slug}-${day}.html` : `marks-${day}.html`;
}

/** The models a role is set with: summary chat, search embeddings, HTML → Markdown extraction. */
export type ModelRole = 'chat' | 'embed' | 'extraction';

export interface ModelRecommendation {
  /** What goes in the model field. */
  model: string;
  /** Rough download size and free memory it needs, in GB; only for local models. */
  sizeGb?: number;
  ramGb?: number;
}

/**
 * Context marks asks Ollama for, in tokens: `OLLAMA_SUMMARY_NUM_CTX` and `OLLAMA_EXTRACT_NUM_CTX` in
 * crates/core/src/genai_model.rs (a test checks it). Settings tells how much memory it adds.
 */
export const OLLAMA_SUMMARY_NUM_CTX = 16_384;
export const OLLAMA_EXTRACT_NUM_CTX = 32_768;

/** ReaderLM-v2 (jinaai) on Ollama: there is no library build, so a community GGUF from Hugging Face. */
const OLLAMA_READERLM = 'hf.co/mradermacher/ReaderLM-v2-GGUF:Q4_K_M';
const OPENAI_SMALL = 'gpt-6-luna';
const ANTHROPIC_SMALL = 'claude-haiku-5-5';

/**
 * Recommended model per provider and role, checked on 2026-11-13 against the Ollama library,
 * Hugging Face, and the OpenAI and Anthropic model pages. Ollama's chat and embeddings are
 * `DEFAULT_CHAT_MODEL` and `DEFAULT_EMBED_MODEL` in crates/core/src/settings.rs (a test checks it).
 * No entry: nothing to recommend (Anthropic has no embeddings; a compatible server can be anything).
 */
export const RECOMMENDED_MODELS: Record<
  Provider,
  Partial<Record<ModelRole, ModelRecommendation>>
> = {
  ollama: {
    chat: { model: 'qwen3:8b', sizeGb: 5.2, ramGb: 8 },
    embed: { model: 'nomic-embed-text', sizeGb: 0.3, ramGb: 1 },
    extraction: { model: OLLAMA_READERLM, sizeGb: 1.1, ramGb: 3 },
  },
  // Hidden until someone checks whether oMLX names a downloaded model by folder or by repo id.
  omlx: {},
  openai: {
    chat: { model: OPENAI_SMALL },
    embed: { model: 'text-embedding-3-small' },
    extraction: { model: OPENAI_SMALL },
  },
  anthropic: {
    chat: { model: ANTHROPIC_SMALL },
    extraction: { model: ANTHROPIC_SMALL },
  },
  openai_compatible: {},
};

export const recommendedModel = (provider: Provider, role: ModelRole): ModelRecommendation | null =>
  RECOMMENDED_MODELS[provider][role] ?? null;

/** Mirrors the Anthropic check in `GenaiModel::embed` (crates/core/src/genai_model.rs). */
export const offersEmbeddings = (provider: Provider): boolean => provider !== 'anthropic';

/** The command that downloads a model; only Ollama has one. */
export const pullCommand = (provider: Provider, model: string): string | null =>
  provider === 'ollama' ? `ollama pull ${model}` : null; // i18n-allow: command
