import { afterEach, describe, expect, it } from 'vitest';
import { setLocale } from './i18n';
import { t } from './strings';

describe('confirmDeleteProfile', () => {
  it('names one bookmark in the singular and more in the plural', () => {
    expect(t.confirmDeleteProfile(1)).toBe('¿Eliminar el perfil y su bookmark?');
    expect(t.confirmDeleteProfile(128)).toBe('¿Eliminar el perfil y sus 128 bookmarks?');
  });
});

describe('singular texts for one bookmark', () => {
  it('uses the singular for n = 1 and the plural otherwise', () => {
    expect(t.cmdExtractView(1)).toBe('Extraer el bookmark sin extraer de esta vista');
    expect(t.cmdExtractView(3)).toBe('Extraer los 3 sin extraer de esta vista');
    expect(t.confirmExtractView(1)).toBe('¿Extraer 1 bookmark?');
    expect(t.confirmExtractView(3)).toBe('¿Extraer 3 bookmarks?');
    expect(t.confirmExtractRemote(1)).toBe(
      'Con la IA activada, su página se enviará a tu proveedor para resumirla.',
    );
    expect(t.confirmExtractRemote(3)).toBe(
      'Con la IA activada, sus 3 páginas se enviarán a tu proveedor para resumirlas.',
    );
    expect(t.notices.extractQueued(1)).toBe('1 bookmark en cola para extraer.');
    expect(t.notices.extractQueued(3)).toBe('3 bookmarks en cola para extraer.');
    expect(t.notices.importDone(1)).toBe('1 bookmark importado, sin extraer.');
    expect(t.notices.importDone(3)).toBe('3 bookmarks importados, sin extraer.');
    expect(t.processQueued(1)).toBe('1 bookmark en cola para resumir.');
    expect(t.processQueued(3)).toBe('3 bookmarks en cola para resumir.');
    expect(t.confirmReindex(1)).toBe('Se volverán a calcular los vectores de 1 bookmark.');
    expect(t.confirmReindex(3)).toBe('Se volverán a calcular los vectores de 3 bookmarks.');
    expect(t.reindexQueued(1)).toBe('1 bookmark en cola para reindexar.');
    expect(t.reindexQueued(3)).toBe('3 bookmarks en cola para reindexar.');
  });

  it('writes the import summary counts in the singular for 1', () => {
    expect(t.importSummary({ new: 1, existing: 1, unsupported: 1, invalid: 0 }, 'Casa')).toBe(
      '1 nuevo, 1 ya estaba, 1 no se puede importar. Perfil Casa.',
    );
    expect(t.importSummary({ new: 2, existing: 3, unsupported: 1, invalid: 1 }, 'Casa')).toBe(
      '2 nuevos, 3 ya estaban, 2 no se pueden importar. Perfil Casa.',
    );
  });
});

describe('the same texts in English', () => {
  afterEach(() => setLocale('es'));

  it('uses the singular for n = 1 and the plural otherwise', () => {
    setLocale('en');
    expect(t.confirmDeleteProfile(1)).toBe('Delete the profile and its bookmark?');
    expect(t.confirmDeleteProfile(128)).toBe('Delete the profile and its 128 bookmarks?');
    expect(t.cmdExtractView(1)).toBe('Extract the unextracted bookmark in this view');
    expect(t.cmdExtractView(3)).toBe('Extract the 3 unextracted bookmarks in this view');
    expect(t.confirmExtractView(1)).toBe('Extract 1 bookmark?');
    expect(t.confirmExtractView(3)).toBe('Extract 3 bookmarks?');
    expect(t.confirmExtractRemote(1)).toBe(
      'With AI turned on, its page will be sent to your provider to be summarized.',
    );
    expect(t.confirmExtractRemote(3)).toBe(
      'With AI turned on, their 3 pages will be sent to your provider to be summarized.',
    );
    expect(t.notices.extractQueued(1)).toBe('1 bookmark queued for extraction.');
    expect(t.notices.extractQueued(3)).toBe('3 bookmarks queued for extraction.');
    expect(t.notices.importDone(1)).toBe('1 bookmark imported, not extracted.');
    expect(t.notices.importDone(3)).toBe('3 bookmarks imported, not extracted.');
    expect(t.processQueued(1)).toBe('1 bookmark queued for summarizing.');
    expect(t.processQueued(3)).toBe('3 bookmarks queued for summarizing.');
    expect(t.confirmReindex(1)).toBe('The vectors of 1 bookmark will be computed again.');
    expect(t.confirmReindex(3)).toBe('The vectors of 3 bookmarks will be computed again.');
    expect(t.reindexQueued(1)).toBe('1 bookmark queued for reindexing.');
    expect(t.reindexQueued(3)).toBe('3 bookmarks queued for reindexing.');
  });

  it('writes the import summary counts in the singular for 1', () => {
    setLocale('en');
    expect(t.importSummary({ new: 1, existing: 1, unsupported: 1, invalid: 0 }, 'Casa')).toBe(
      '1 new, 1 already there, 1 can’t be imported. Profile Casa.',
    );
    expect(t.importSummary({ new: 2, existing: 3, unsupported: 1, invalid: 1 }, 'Casa')).toBe(
      '2 new, 3 already there, 2 can’t be imported. Profile Casa.',
    );
  });

  it('writes decimal figures with the active locale', () => {
    expect(t.ollamaContextExtraction(16384)).toContain('0,9 GB');
    setLocale('en');
    expect(t.ollamaContextExtraction(16384)).toContain('0.9 GB');
    expect(t.ollamaContextChat(8192, 16384)).toContain('2.2 and 4.6 GB');
  });

  it('names each theme command without lowercasing a label', () => {
    expect(t.cmdTheme('system')).toBe('Tema: como el sistema');
    setLocale('en');
    expect(t.cmdTheme('dark')).toBe('Theme: dark');
  });
});
