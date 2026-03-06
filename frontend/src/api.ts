const BASE = import.meta.env.DEV ? 'http://localhost:8008/api' : '/api';

function getAuthToken(): string | null {
  const cookie = document.cookie.split('; ').find(c => c.startsWith('auth_token='));
  return cookie ? cookie.split('=')[1] : null;
}

function authHeaders(): HeadersInit {
  const token = getAuthToken();
  return token ? { 'Authorization': `Bearer ${token}` } : {};
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (res.status === 401) {
    document.cookie = 'auth_token=; max-age=0; path=/';
    window.location.reload();
    throw new Error('Unauthorized');
  }
  return res.json();
}

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
  archived: boolean;
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
  const res = await fetch(`${BASE}/books`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function fetchBook(id: number): Promise<Book> {
  const res = await fetch(`${BASE}/books/${id}`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function createBook(book: CreateBookRequest): Promise<Book> {
  const res = await fetch(`${BASE}/books`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(book),
  });
  return handleResponse(res);
}

export async function updateBook(id: number, book: CreateBookRequest): Promise<Book> {
  const res = await fetch(`${BASE}/books/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify(book),
  });
  return handleResponse(res);
}

export async function deleteBook(id: number): Promise<void> {
  await fetch(`${BASE}/books/${id}`, { method: 'DELETE', headers: authHeaders() });
}

export async function toggleArchive(id: number): Promise<Book> {
  const res = await fetch(`${BASE}/books/${id}/archive`, {
    method: 'POST',
    headers: authHeaders(),
  });
  return handleResponse(res);
}

// --- Tags ---

export async function fetchTags(kind?: string): Promise<Tag[]> {
  const url = kind ? `${BASE}/tags?kind=${kind}` : `${BASE}/tags`;
  const res = await fetch(url, { headers: authHeaders() });
  return handleResponse(res);
}

export async function createTag(name: string, kind: string): Promise<Tag> {
  const res = await fetch(`${BASE}/tags`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', ...authHeaders() },
    body: JSON.stringify({ name, kind }),
  });
  return handleResponse(res);
}

export async function deleteTag(id: number): Promise<void> {
  await fetch(`${BASE}/tags/${id}`, { method: 'DELETE', headers: authHeaders() });
}

// --- Open Library ---

export interface ISBNLookupResult {
  title: string;
  authors: string[];
  subjects: string[];
  coverUrl: string | null;
}

export async function lookupISBN(isbn: string): Promise<ISBNLookupResult> {
  const cleanISBN = isbn.replace(/[^0-9X]/gi, '');
  const url = `https://openlibrary.org/api/books?bibkeys=ISBN:${cleanISBN}&jscmd=data&format=json`;

  let res: Response;
  try {
    res = await fetch(url);
  } catch (e) {
    throw new Error('Connection failed');
  }

  if (res.status === 429) {
    throw new Error('Rate limited');
  }
  if (res.status >= 500) {
    throw new Error('Service unavailable');
  }
  if (!res.ok) {
    throw new Error('Request failed');
  }

  const data = await res.json();
  const key = `ISBN:${cleanISBN}`;
  const book = data[key];

  if (!book) {
    throw new Error('ISBN not found');
  }

  return {
    title: book.title || '',
    authors: (book.authors || []).map((a: any) => a.name || '').filter(Boolean),
    subjects: (book.subjects || []).map((s: any) => s.name || '').filter(Boolean),
    coverUrl: book.cover?.medium || null,
  };
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

function statsParams(tags?: number[], start?: string, end?: string, archived?: boolean): string {
  const params = new URLSearchParams();
  if (tags?.length) params.set('tags', tags.join(','));
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  if (archived !== undefined) params.set('archived', String(archived));
  const str = params.toString();
  return str ? `?${str}` : '';
}

export async function fetchTotals(tags?: number[], start?: string, end?: string, archived?: boolean): Promise<Totals> {
  const res = await fetch(`${BASE}/stats/totals${statsParams(tags, start, end, archived)}`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function fetchByTag(kind?: string, tags?: number[], start?: string, end?: string, archived?: boolean): Promise<TagCount[]> {
  const params = new URLSearchParams();
  if (kind) params.set('kind', kind);
  if (tags?.length) params.set('tags', tags.join(','));
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  if (archived !== undefined) params.set('archived', String(archived));
  const str = params.toString();
  const res = await fetch(`${BASE}/stats/by-tag${str ? `?${str}` : ''}`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function fetchByAuthor(tags?: number[], start?: string, end?: string, archived?: boolean): Promise<AuthorCount[]> {
  const res = await fetch(`${BASE}/stats/by-author${statsParams(tags, start, end, archived)}`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function fetchGrowth(groupBy?: string, tags?: number[], start?: string, end?: string, archived?: boolean): Promise<GrowthBucket[]> {
  const params = new URLSearchParams();
  if (groupBy) params.set('group_by', groupBy);
  if (tags?.length) params.set('tags', tags.join(','));
  if (start) params.set('start', start);
  if (end) params.set('end', end);
  if (archived !== undefined) params.set('archived', String(archived));
  const str = params.toString();
  const res = await fetch(`${BASE}/stats/growth${str ? `?${str}` : ''}`, { headers: authHeaders() });
  return handleResponse(res);
}
