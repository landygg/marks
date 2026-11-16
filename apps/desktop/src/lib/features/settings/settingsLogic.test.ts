import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import type { ExtractionSettings, Provider } from '../../api';
import {
  AI_MODES,
  EXTRACTION_MODES,
  OLLAMA_DEFAULT_ENDPOINT,
  OLLAMA_EXTRACT_NUM_CTX,
  OLLAMA_SUMMARY_NUM_CTX,
  SETTINGS_GROUPS,
  SETTINGS_SECTIONS,
  aiMode,
  enabledFor,
  endpointPlaceholder,
  exportFileName,
  extractionMode,
  extractionToSave,
  isRemoteEndpoint,
  keyToSend,
  pullCommand,
  recommendedModel,
  type ModelRole,
  sameAsSummaryFor,
  readyCount,
  requiresEndpoint,
} from './settingsLogic';

describe('aiMode', () => {
  it('names each state of the switch', () => {
    expect(aiMode(true)).toBe('on');
    expect(aiMode(false)).toBe('off');
    expect(AI_MODES).toEqual(['on', 'off']);
  });

  it('maps every mode back to the same switch state', () => {
    expect(enabledFor('on')).toBe(true);
    expect(enabledFor('off')).toBe(false);
    for (const mode of AI_MODES) expect(aiMode(enabledFor(mode))).toBe(mode);
  });
});

describe('keyToSend', () => {
  it('keeps the stored key when the input is empty and no clear was asked', () => {
    expect(keyToSend('', false)).toBeNull();
  });

  it('deletes the stored key when a clear was asked', () => {
    expect(keyToSend('', true)).toBe('');
  });

  it('stores a typed key, even after a clear was asked', () => {
    expect(keyToSend('sk-1', false)).toBe('sk-1');
    expect(keyToSend('sk-1', true)).toBe('sk-1');
  });
});

describe('endpointPlaceholder', () => {
  it.each<[Provider, string]>([
    ['ollama', OLLAMA_DEFAULT_ENDPOINT],
    ['omlx', ''],
    ['openai', ''],
    ['anthropic', ''],
    ['openai_compatible', ''],
  ])('%s → %j', (provider, expected) => {
    expect(endpointPlaceholder(provider)).toBe(expected);
  });
});

describe('requiresEndpoint', () => {
  it('is true only for omlx and openai_compatible', () => {
    const providers: Provider[] = ['ollama', 'omlx', 'openai', 'anthropic', 'openai_compatible'];
    expect(providers.filter(requiresEndpoint)).toEqual(['omlx', 'openai_compatible']);
  });
});

describe('readyCount', () => {
  it('is every bookmark that is neither pending (either kind) nor failed', () => {
    expect(
      readyCount({
        total: 10,
        pending: 3,
        failed: 2,
        unsummarized: 1,
        unextracted: 1,
        imported: 0,
      }),
    ).toBe(5);
  });

  it('readyCount leaves out imported', () => {
    expect(
      readyCount({
        total: 10,
        pending: 2,
        failed: 1,
        imported: 3,
        unsummarized: 0,
        unextracted: 0,
      }),
    ).toBe(4);
  });
});

describe('SETTINGS_GROUPS', () => {
  it('lists every section once, in the order of SETTINGS_SECTIONS', () => {
    expect(SETTINGS_GROUPS.flatMap((group) => group.sections)).toEqual([...SETTINGS_SECTIONS]);
  });

  it('lists updates in the app group, after ai', () => {
    expect(SETTINGS_GROUPS.find((group) => group.scope === 'app')?.sections).toEqual([
      'appearance',
      'ai',
      'updates',
    ]);
  });

  it('lists transfer in the profile group, between extension and maintenance', () => {
    expect(SETTINGS_GROUPS.find((group) => group.scope === 'profile')?.sections).toEqual([
      'profile',
      'extension',
      'transfer',
      'maintenance',
    ]);
  });
});

describe('exportFileName', () => {
  it('names the file after the profile and the date', () => {
    expect(exportFileName('Trabajo', new Date(2026, 10, 13))).toBe('marks-trabajo-2026-11-13.html');
  });

  it('lowercases and turns every run of non-alphanumerics into one dash, none at the ends', () => {
    expect(exportFileName('Mi/Perfil: 1', new Date(2026, 10, 13))).toBe(
      'marks-mi-perfil-1-2026-11-13.html',
    );
  });

  it('leaves the profile out when its name has no letter or digit', () => {
    expect(exportFileName('!!!', new Date(2026, 10, 13))).toBe('marks-2026-11-13.html');
    expect(exportFileName('', new Date(2026, 10, 13))).toBe('marks-2026-11-13.html');
  });

  it('keeps a decomposed accent with its letter', () => {
    expect(exportFileName('Re\u0301sume\u0301', new Date(2026, 10, 13))).toBe(
      'marks-résumé-2026-11-13.html',
    );
  });
});

describe('isRemoteEndpoint', () => {
  it('treats the cloud providers as remote, whatever the endpoint', () => {
    expect(isRemoteEndpoint('openai', null)).toBe(true);
    expect(isRemoteEndpoint('anthropic', 'http://localhost:1')).toBe(true);
  });

  it('treats this machine as local', () => {
    expect(isRemoteEndpoint('ollama', null)).toBe(false);
    expect(isRemoteEndpoint('ollama', '')).toBe(false);
    for (const endpoint of [
      'http://localhost:8000',
      'http://127.0.0.1:1234/v1',
      'http://[::1]:9',
    ]) {
      expect(isRemoteEndpoint('openai_compatible', endpoint)).toBe(false);
    }
  });

  it('treats any other host as remote, also on the local network', () => {
    expect(isRemoteEndpoint('ollama', 'http://192.168.1.20:11434')).toBe(true);
    expect(isRemoteEndpoint('omlx', ' http://mac-mini.local:8000 ')).toBe(true);
  });

  it('does not warn while a required endpoint is missing or unreadable', () => {
    expect(isRemoteEndpoint('omlx', null)).toBe(false);
    expect(isRemoteEndpoint('openai_compatible', 'no es una url')).toBe(false);
  });
});

describe('extractionMode', () => {
  it('maps the stored flag to the switch and back', () => {
    expect(extractionMode(true)).toBe('same');
    expect(extractionMode(false)).toBe('own');
    expect(EXTRACTION_MODES).toEqual(['same', 'own']);
    for (const mode of EXTRACTION_MODES) expect(extractionMode(sameAsSummaryFor(mode))).toBe(mode);
  });
});

describe('extractionToSave', () => {
  const form: ExtractionSettings = {
    same_as_summary: false,
    provider: 'openai_compatible',
    endpoint: 'http://192.168.1.20:8000',
    model: 'reader-lm',
  };

  it('with its own settings sends them and the key as typed', () => {
    expect(extractionToSave(form, 'sk-2', false)).toEqual({ settings: form, apiKey: 'sk-2' });
    expect(extractionToSave(form, '', true).apiKey).toBe('');
    expect(extractionToSave(form, '', false).apiKey).toBeNull();
  });

  it('with the summary settings keeps the own fields and never touches the key', () => {
    const same = { ...form, same_as_summary: true };
    expect(extractionToSave(same, 'sk-2', true)).toEqual({ settings: same, apiKey: null });
  });
});

describe('recommendedModel', () => {
  const roles: ModelRole[] = ['chat', 'embed', 'extraction'];

  it.each<[Provider, ModelRole, string | null]>([
    ['ollama', 'chat', 'qwen3:8b'],
    ['ollama', 'embed', 'nomic-embed-text'],
    ['ollama', 'extraction', 'hf.co/mradermacher/ReaderLM-v2-GGUF:Q4_K_M'],
    ['omlx', 'chat', null],
    ['omlx', 'embed', null],
    ['omlx', 'extraction', null],
    ['openai', 'chat', 'gpt-6-luna'],
    ['openai', 'embed', 'text-embedding-3-small'],
    ['openai', 'extraction', 'gpt-6-luna'],
    ['anthropic', 'chat', 'claude-haiku-5-5'],
    ['anthropic', 'embed', null],
    ['anthropic', 'extraction', 'claude-haiku-5-5'],
    ['openai_compatible', 'chat', null],
    ['openai_compatible', 'embed', null],
    ['openai_compatible', 'extraction', null],
  ])('%s %s → %j', (provider, role, expected) => {
    expect(recommendedModel(provider, role)?.model ?? null).toBe(expected);
  });

  it('gives a size and memory for every local model, and none for the cloud', () => {
    for (const provider of ['ollama'] as const)
      for (const role of roles) {
        const rec = recommendedModel(provider, role);
        expect(rec?.sizeGb).toBeGreaterThan(0);
        expect(rec?.ramGb).toBeGreaterThan(0);
      }
    for (const provider of ['openai', 'anthropic'] as const)
      for (const role of roles) expect(recommendedModel(provider, role)?.sizeGb).toBeUndefined();
  });

  it('matches the Ollama defaults in crates/core/src/settings.rs', () => {
    const rust = readFileSync(
      fileURLToPath(new URL('../../../../../../crates/core/src/settings.rs', import.meta.url)),
      'utf8',
    );
    const rustConst = (name: string) => rust.match(new RegExp(`${name}: &str = "([^"]+)"`))?.[1];
    expect(recommendedModel('ollama', 'chat')?.model).toBe(rustConst('DEFAULT_CHAT_MODEL'));
    expect(recommendedModel('ollama', 'embed')?.model).toBe(rustConst('DEFAULT_EMBED_MODEL'));
  });
});

describe('Ollama context', () => {
  it('matches the num_ctx marks sends in crates/core/src/genai_model.rs', () => {
    const rust = readFileSync(
      fileURLToPath(new URL('../../../../../../crates/core/src/genai_model.rs', import.meta.url)),
      'utf8',
    );
    const rustNum = (name: string) =>
      Number(rust.match(new RegExp(`${name}: u32 = ([\\d_]+);`))?.[1].replaceAll('_', ''));
    expect(rustNum('OLLAMA_SUMMARY_NUM_CTX')).toBe(OLLAMA_SUMMARY_NUM_CTX);
    expect(rustNum('OLLAMA_EXTRACT_NUM_CTX')).toBe(OLLAMA_EXTRACT_NUM_CTX);
  });
});

describe('pullCommand', () => {
  it('pulls with Ollama and with nothing else', () => {
    expect(pullCommand('ollama', 'qwen3:8b')).toBe('ollama pull qwen3:8b');
    for (const provider of ['omlx', 'openai', 'anthropic', 'openai_compatible'] as const)
      expect(pullCommand(provider, 'x')).toBeNull();
  });
});
