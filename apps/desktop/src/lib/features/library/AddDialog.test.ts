// @vitest-environment jsdom
import { flushSync, mount, tick, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import profileDialog from '../profiles/ProfileDialog.svelte?raw';
import AddDialog from './AddDialog.svelte';

vi.mock('../../api', () => ({ add: vi.fn(async () => Promise.reject('Invalid URL')) }));
vi.mock('./libraryState.svelte', () => ({
  lib: { addDialogOpen: true },
  closeAddDialog: vi.fn(),
  refresh: vi.fn(),
  report: vi.fn(),
}));

let view: ReturnType<typeof mount> | null = null;

beforeEach(() => {
  // jsdom has no modal dialogs: the attachment only needs the two methods.
  HTMLDialogElement.prototype.showModal = function () {
    this.open = true;
  };
  HTMLDialogElement.prototype.close = function () {
    this.open = false;
  };
  view = mount(AddDialog, { target: document.body });
  flushSync();
});

afterEach(() => {
  if (view) unmount(view);
  view = null;
  document.body.replaceChildren();
});

const urlField = () => document.body.querySelector('input[type="url"]') as HTMLInputElement;

async function submitFailing(): Promise<void> {
  document.body
    .querySelector('form')
    ?.dispatchEvent(new SubmitEvent('submit', { cancelable: true }));
  await vi.waitFor(() => expect(document.body.querySelector('[role="alert"]')).not.toBeNull());
  await tick();
}

describe('AddDialog', () => {
  it('marks the URL field with the error, points it at the message and focuses it', async () => {
    await submitFailing();
    const field = urlField();
    expect(field.getAttribute('aria-invalid')).toBe('true');
    const message = document.getElementById(field.getAttribute('aria-describedby') ?? '');
    expect(message?.getAttribute('role')).toBe('alert');
    expect(message?.textContent).toBe('Invalid URL');
    expect(document.activeElement).toBe(field);
  });

  it('clears the mark when the user types', async () => {
    await submitFailing();
    const field = urlField();
    field.value = 'https://example.com';
    field.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(field.hasAttribute('aria-invalid')).toBe(false);
  });

  it('titles the dialog with an h1', () => {
    const dialog = document.body.querySelector('dialog');
    const title = document.getElementById(dialog?.getAttribute('aria-labelledby') ?? '');
    expect(title?.tagName).toBe('H1');
  });
});

describe('ProfileDialog', () => {
  it('titles the dialog with an h1', () => {
    expect(profileDialog).toMatch(/<h1\b[^>]*id="\{uid\}-title"/);
    expect(profileDialog).not.toMatch(/<h2\b/);
  });
});
