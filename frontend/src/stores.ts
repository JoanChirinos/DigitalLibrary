import { writable } from 'svelte/store';
import { fetchBooks, fetchTags, type Book, type Tag } from './api';

export const books = writable<Book[]>([]);
export const tags = writable<Tag[]>([]);
export const showArchived = writable<boolean>(false);

export async function loadBooks() {
  books.set(await fetchBooks());
}

export async function loadTags() {
  tags.set(await fetchTags());
}
