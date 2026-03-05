const BASE = 'http://localhost:8008';

// --- Types ---

export interface Tag {
  id: number;
  name: string;
  kind: string;
}

export interface Author {
  id: number;
  first_name: string;
  last_name: string;
}

export interface Book {
  id: number;
  title: string;
  scan_date: string;
  isbn: string | null;
  cover_url: string | null;
  authors: Author[];
  tags: Tag[];
}

export interface CreateBookRequest {
  title: string;
  scan_date: string;
  isbn?: string;
  cover_url?: string;
  authors: { first_name: string; last_name: string }[];
  tag_ids: number[];
}

// --- Books ---

export async function fetchBooks(): Promise<Book[]> {
  const res = await fetch(`${BASE}/books`);
  return res.json();
}

export async function fetchBook(id: number): Promise<Book> {
  const res = await fetch(`${BASE}/books/${id}`);
  return res.json();
}

export async function createBook(book: CreateBookRequest): Promise<Book> {
  const res = await fetch(`${BASE}/books`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(book),
  });
  return res.json();
}

export async function updateBook(id: number, book: CreateBookRequest): Promise<Book> {
  const res = await fetch(`${BASE}/books/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(book),
  });
  return res.json();
}

export async function deleteBook(id: number): Promise<void> {
  await fetch(`${BASE}/books/${id}`, { method: 'DELETE' });
}

// --- Tags ---

export async function fetchTags(kind?: string): Promise<Tag[]> {
  const url = kind ? `${BASE}/tags?kind=${kind}` : `${BASE}/tags`;
  const res = await fetch(url);
  return res.json();
}

export async function createTag(name: string, kind: string): Promise<Tag> {
  const res = await fetch(`${BASE}/tags`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, kind }),
  });
  return res.json();
}

export async function deleteTag(id: number): Promise<void> {
  await fetch(`${BASE}/tags/${id}`, { method: 'DELETE' });
}

// --- Stats ---

export interface Totals {
  books: number;
  authors: number;
  tags: number;
}

export interface TagCount {
  tag_name: string;
  tag_kind: string;
  count: number;
}

export interface AuthorCount {
  first_name: string;
  last_name: string;
  count: number;
}

export interface GrowthBucket {
  period: string;
  count: number;
}

function statsParams(tags?: number[], start?: string, end?: string): string {
  const params = new URLSearchParams();
  if (tags?.length) params.set('tags', tags.join(','));
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  const str = params.toString();
  return str ? `?${str}` : '';
}

export async function fetchTotals(tags?: number[], start?: string, end?: string): Promise<Totals> {
  const res = await fetch(`${BASE}/stats/totals${statsParams(tags, start, end)}`);
  return res.json();
}

export async function fetchByTag(kind?: string, tags?: number[], start?: string, end?: string): Promise<TagCount[]> {
  const params = new URLSearchParams();
  if (kind) params.set('kind', kind);
  if (tags?.length) params.set('tags', tags.join(','));
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  const str = params.toString();
  const res = await fetch(`${BASE}/stats/by-tag${str ? `?${str}` : ''}`);
  return res.json();
}

export async function fetchByAuthor(tags?: number[], start?: string, end?: string): Promise<AuthorCount[]> {
  const res = await fetch(`${BASE}/stats/by-author${statsParams(tags, start, end)}`);
  return res.json();
}

export async function fetchGrowth(groupBy?: string, tags?: number[], start?: string, end?: string): Promise<GrowthBucket[]> {
  const params = new URLSearchParams();
  if (groupBy) params.set('group_by', groupBy);
  if (tags?.length) params.set('tags', tags.join(','));
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  const str = params.toString();
  const res = await fetch(`${BASE}/stats/growth${str ? `?${str}` : ''}`);
  return res.json();
}
