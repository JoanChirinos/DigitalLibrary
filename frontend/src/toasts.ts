import { writable } from 'svelte/store';

export interface Toast {
  id: number;
  message: string;
  kind: 'error' | 'info' | 'success';
}

export const toasts = writable<Toast[]>([]);

let nextId = 0;

export function pushToast(message: string, kind: Toast['kind'] = 'error', ttl = 5000) {
  const id = ++nextId;
  toasts.update((list) => [...list, { id, message, kind }]);
  setTimeout(() => dismissToast(id), ttl);
}

export function dismissToast(id: number) {
  toasts.update((list) => list.filter((t) => t.id !== id));
}

export function pushError(message: string) {
  pushToast(message, 'error');
}
