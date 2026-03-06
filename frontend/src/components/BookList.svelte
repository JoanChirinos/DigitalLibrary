<script lang="ts">
  import { books, tags, showArchived } from '../stores';
  import { deleteBook } from '../api';
  import { loadBooks } from '../stores';
  import { Trash2, X, Edit, Save, Archive, ArchiveRestore } from 'lucide-svelte';
  import type { Book, Tag } from '../api';
  import Fuse from 'fuse.js';
  import { updateBook, createTag, toggleArchive } from '../api';
  import { loadTags } from '../stores';

  function utcToLocalDate(utc: string): string {
    const date = new Date(utc.endsWith('Z') ? utc : utc + 'Z');
    return date.toLocaleDateString();
  }

  function utcToLocalDateTime(utc: string): string {
    const date = new Date(utc.endsWith('Z') ? utc : utc + 'Z');
    return date.toLocaleString();
  }

  let sortBy = $state<'title' | 'author' | 'recent'>('author');
  let selectedTags = $state<number[]>([]);
  let searchQuery = $state('');

  // Edit state
  let editingId = $state<number | null>(null);
  let detailBook = $state<Book | null>(null);
  let deleteConfirmBook = $state<Book | null>(null);
  let editTitle = $state('');
  let editIsbn = $state('');
  let editCoverUrl = $state('');
  let editAuthors = $state<{first_name: string; last_name: string}[]>([]);
  let editTagIds = $state<number[]>([]);
  let editFirstName = $state('');
  let editLastName = $state('');
  let showEditSuggestions = $state(false);
  let selectedSuggestionIndex = $state(-1);

  // Author autocomplete for edit
  let allAuthors = $derived(
    Object.values(
      $books.flatMap(b => b.authors).reduce((acc: Record<string, {first_name: string; last_name: string}>, a) => {
        acc[`${a.first_name}|${a.last_name}`] = { first_name: a.first_name, last_name: a.last_name };
        return acc;
      }, {})
    ).sort((a, b) => a.last_name.localeCompare(b.last_name))
  );

  let authorFuse = $derived(new Fuse(allAuthors, { keys: ['first_name', 'last_name'], threshold: 0.4 }));

  let editAuthorSuggestions = $derived.by(() => {
    const q = `${editFirstName} ${editLastName}`.trim();
    if (!q) return [];
    return authorFuse.search(q)
      .map(r => r.item)
      .filter(a => !editAuthors.some(x => x.first_name === a.first_name && x.last_name === a.last_name))
      .slice(0, 8);
  });

  // Group tags by kind for the filter UI
  let tagsByKind = $derived(
    ($tags).reduce((acc: Record<string, Tag[]>, tag) => {
      (acc[tag.kind] ??= []).push(tag);
      return acc;
    }, {})
  );

  // Fuse.js for fuzzy search
  let fuse = $derived(new Fuse($books, {
    keys: ['title', 'authors.first_name', 'authors.last_name'],
    threshold: 0.4,
  }));

  let filteredBooks = $derived.by(() => {
    let result = [...$books];

    // Filter archived
    if (!$showArchived) {
      result = result.filter(book => !book.archived);
    }

    // Search
    if (searchQuery.trim()) {
      result = fuse.search(searchQuery).map(r => r.item);
    }

    // Filter by selected tags (must have ALL selected)
    if (selectedTags.length > 0) {
      result = result.filter(book =>
        selectedTags.every(tagId => book.tags.some(t => t.id === tagId))
      );
    }

    // Sort
    if (sortBy === 'title') {
      result.sort((a, b) => a.title.localeCompare(b.title));
    } else if (sortBy === 'author') {
      result.sort((a, b) => {
        const aName = a.authors[0]?.last_name ?? '';
        const bName = b.authors[0]?.last_name ?? '';
        return aName.localeCompare(bName);
      });
    } else {
      result.sort((a, b) => b.scan_date.localeCompare(a.scan_date));
    }

    return result;
  });

  let perPage = $state(20);
  let page = $state(1);

  let totalPages = $derived(Math.ceil(filteredBooks.length / perPage));
  let pagedBooks = $derived(filteredBooks.slice((page - 1) * perPage, page * perPage));

  // Reset to page 1 when filters/sort/perPage change
  $effect(() => {
    filteredBooks;
    perPage;
    page = 1;
  });

  function startEdit(book: Book) {
    editingId = book.id;
    editTitle = book.title;
    editIsbn = book.isbn || '';
    editCoverUrl = book.cover_url || '';
    editAuthors = book.authors.map(a => ({ first_name: a.first_name, last_name: a.last_name }));
    editTagIds = book.tags.map(t => t.id);
    editFirstName = '';
    editLastName = '';
  }

  function cancelEdit() {
    editingId = null;
  }

  function addEditAuthor() {
    const fn = editFirstName.trim();
    const ln = editLastName.trim();
    if (fn && ln && !editAuthors.some(a => a.first_name === fn && a.last_name === ln)) {
      editAuthors = [...editAuthors, { first_name: fn, last_name: ln }];
    }
    editFirstName = '';
    editLastName = '';
    showEditSuggestions = false;
    selectedSuggestionIndex = -1;
  }

  function selectEditAuthor(a: {first_name: string; last_name: string}) {
    if (!editAuthors.some(x => x.first_name === a.first_name && x.last_name === a.last_name)) {
      editAuthors = [...editAuthors, a];
    }
    editFirstName = '';
    editLastName = '';
    showEditSuggestions = false;
    selectedSuggestionIndex = -1;
  }

  function handleEditAuthorKeydown(e: KeyboardEvent) {
    if (!showEditSuggestions || editAuthorSuggestions.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedSuggestionIndex = Math.min(selectedSuggestionIndex + 1, editAuthorSuggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedSuggestionIndex = Math.max(selectedSuggestionIndex - 1, -1);
    } else if (e.key === 'Enter' && selectedSuggestionIndex >= 0) {
      e.preventDefault();
      selectEditAuthor(editAuthorSuggestions[selectedSuggestionIndex]);
    }
  }

  function removeEditAuthor(a: {first_name: string; last_name: string}) {
    editAuthors = editAuthors.filter(x => x.first_name !== a.first_name || x.last_name !== a.last_name);
  }

  function toggleEditTag(id: number) {
    if (editTagIds.includes(id)) {
      editTagIds = editTagIds.filter(t => t !== id);
    } else {
      editTagIds = [...editTagIds, id];
    }
  }

  async function saveEdit() {
    if (!editingId || !editTitle.trim()) return;
    const book = $books.find(b => b.id === editingId);
    if (!book) return;
    await updateBook(editingId, {
      title: editTitle.trim(),
      scan_date: book.scan_date,
      isbn: editIsbn || undefined,
      cover_url: editCoverUrl || undefined,
      authors: editAuthors,
      tag_ids: editTagIds,
    });
    await loadBooks();
    editingId = null;
  }

  function toggleTag(id: number) {
    if (selectedTags.includes(id)) {
      selectedTags = selectedTags.filter(t => t !== id);
    } else {
      selectedTags = [...selectedTags, id];
    }
  }

  async function handleArchive(id: number) {
    await toggleArchive(id);
    await loadBooks();
  }

  async function handleDelete(id: number) {
    await deleteBook(id);
    await loadBooks();
    deleteConfirmBook = null;
  }
</script>

<section>
  <!-- Search -->
  <div class="flex gap-4 items-center mb-4">
    <div class="flex-1 relative">
      <input
        class="input input-bordered w-full"
        placeholder="Search books by title or author..."
        bind:value={searchQuery}
        onkeydown={(e) => { if (e.key === 'Enter') e.currentTarget.blur(); }}
      />
      {#if searchQuery}
        <button
          class="btn btn-ghost btn-sm absolute right-1 top-1"
          onclick={() => searchQuery = ''}
        >
          <X size={16} />
        </button>
      {/if}
    </div>
    <label class="label cursor-pointer gap-2 justify-start">
      <input type="checkbox" class="checkbox checkbox-sm" bind:checked={$showArchived} />
      <span class="label-text">Show archived</span>
    </label>
  </div>

  <!-- Sort + Filter controls -->
  <div class="flex flex-wrap gap-4 mb-4 items-start">
    <div>
      <span class="text-sm font-semibold mr-2">Sort:</span>
      <div class="join">
        <button class="join-item btn btn-sm" class:btn-active={sortBy === 'author'} onclick={() => sortBy = 'author'}>Author</button>
        <button class="join-item btn btn-sm" class:btn-active={sortBy === 'title'} onclick={() => sortBy = 'title'}>Title</button>
        <button class="join-item btn btn-sm" class:btn-active={sortBy === 'recent'} onclick={() => sortBy = 'recent'}>Recent</button>
      </div>
    </div>

    <div>
      <span class="text-sm font-semibold mr-2">Per page:</span>
      <select class="select select-bordered select-sm" bind:value={perPage}>
        <option value={5}>5</option>
        <option value={20}>20</option>
        <option value={50}>50</option>
      </select>
    </div>

    <div class="flex flex-wrap gap-4">
      <!-- Owner tags as pills -->
      {#if tagsByKind['owner']?.length}
        <div>
          <span class="text-xs font-semibold uppercase opacity-60">Owner</span>
          <div class="flex flex-wrap gap-1 mt-1">
            {#each tagsByKind['owner'] as tag}
              <button
                class="badge badge-outline cursor-pointer"
                class:badge-primary={selectedTags.includes(tag.id)}
                onclick={() => toggleTag(tag.id)}
              >{tag.name}</button>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Genre dropdown + pills -->
      {#if tagsByKind['genre']?.length}
        <div>
          <span class="text-xs font-semibold uppercase opacity-60">Genre</span>
          <div class="mt-1">
            <select
              class="select select-xs select-bordered"
              onchange={(e) => { const v = Number(e.currentTarget.value); if (v) toggleTag(v); e.currentTarget.value = ''; }}
            >
              <option value="">+ genre</option>
              {#each tagsByKind['genre'].filter(t => !selectedTags.includes(t.id)).sort((a, b) => a.name.localeCompare(b.name)) as tag}
                <option value={tag.id}>{tag.name}</option>
              {/each}
            </select>
            <div class="flex flex-wrap gap-1 mt-1">
              {#each tagsByKind['genre'].filter(t => selectedTags.includes(t.id)) as tag}
                <button class="badge badge-primary cursor-pointer" onclick={() => toggleTag(tag.id)}>
                  {tag.name} ✕
                </button>
              {/each}
            </div>
          </div>
        </div>
      {/if}

      <!-- Custom dropdown + pills -->
      {#if tagsByKind['custom']?.length}
        <div>
          <span class="text-xs font-semibold uppercase opacity-60">Tags</span>
          <div class="mt-1">
            <select
              class="select select-xs select-bordered"
              onchange={(e) => { const v = Number(e.currentTarget.value); if (v) toggleTag(v); e.currentTarget.value = ''; }}
            >
              <option value="">+ tag</option>
              {#each tagsByKind['custom'].filter(t => !selectedTags.includes(t.id)).sort((a, b) => a.name.localeCompare(b.name)) as tag}
                <option value={tag.id}>{tag.name}</option>
              {/each}
            </select>
            <div class="flex flex-wrap gap-1 mt-1">
              {#each tagsByKind['custom'].filter(t => selectedTags.includes(t.id)) as tag}
                <button class="badge badge-primary cursor-pointer" onclick={() => toggleTag(tag.id)}>
                  {tag.name} ✕
                </button>
              {/each}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <!-- Book list -->
  {#if filteredBooks.length === 0}
    <p class="text-base-content/50">No books found.</p>
  {:else}
    <p class="text-sm opacity-60 mb-2">{filteredBooks.length} books</p>
    <div class="flex flex-col gap-3">
      {#each pagedBooks as book}
        <div class="card card-compact bg-base-100 shadow" class:opacity-50={book.archived}>
          {#if editingId === book.id}
            <!-- Edit mode -->
            <div class="card-body">
              <input class="input input-bordered input-sm w-full mb-2" placeholder="Title" bind:value={editTitle} />
              <input class="input input-bordered input-sm w-full mb-2" placeholder="ISBN" bind:value={editIsbn} />
              <input class="input input-bordered input-sm w-full mb-2" placeholder="Cover URL" bind:value={editCoverUrl} />

              <!-- Authors -->
              <div class="mb-2">
                <span class="text-sm font-semibold">Authors</span>
                <div class="relative flex gap-2 mt-1">
                  <input
                    class="input input-bordered input-sm flex-1"
                    placeholder="First name"
                    bind:value={editFirstName}
                    onfocus={() => showEditSuggestions = true}
                    onblur={() => setTimeout(() => showEditSuggestions = false, 200)}
                    onkeydown={handleEditAuthorKeydown}
                  />
                  <input
                    class="input input-bordered input-sm flex-1"
                    placeholder="Last name"
                    bind:value={editLastName}
                    onfocus={() => showEditSuggestions = true}
                    onblur={() => setTimeout(() => showEditSuggestions = false, 200)}
                    onkeydown={handleEditAuthorKeydown}
                  />
                  <button class="btn btn-sm btn-outline" onclick={addEditAuthor}>Add</button>
                </div>
                {#if showEditSuggestions && editAuthorSuggestions.length > 0}
                  <ul class="menu bg-base-100 shadow-lg rounded-box z-10 w-full mt-1 max-h-48 overflow-y-auto">
                    {#each editAuthorSuggestions as suggestion, i}
                      <li class:bg-base-200={i === selectedSuggestionIndex}><button onmousedown={() => selectEditAuthor(suggestion)}>{suggestion.first_name} {suggestion.last_name}</button></li>
                    {/each}
                  </ul>
                {/if}
                {#if editAuthors.length > 0}
                  <div class="flex flex-wrap gap-1 mt-2">
                    {#each editAuthors as author}
                      <button class="badge badge-primary cursor-pointer" onclick={() => removeEditAuthor(author)}>
                        {author.first_name} {author.last_name} <X size={12} />
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>

              <!-- Tags -->
              <div class="mb-2">
                <span class="text-sm font-semibold">Tags</span>
                <div class="flex flex-wrap gap-2 mt-1">
                  {#each Object.entries(tagsByKind) as [kind, kindTags]}
                    <div>
                      <span class="text-xs uppercase opacity-60">{kind}</span>
                      <div class="flex flex-wrap gap-1 mt-1">
                        {#each kindTags.sort((a, b) => a.name.localeCompare(b.name)) as tag}
                          <button
                            class="badge badge-outline badge-sm cursor-pointer"
                            class:badge-primary={editTagIds.includes(tag.id)}
                            onclick={() => toggleEditTag(tag.id)}
                          >{tag.name}</button>
                        {/each}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>

              <!-- Scan date (read-only) -->
              <p class="text-xs opacity-50">Scanned: {utcToLocalDate(book.scan_date)}</p>

              <div class="flex gap-2 justify-end mt-2">
                <button class="btn btn-sm btn-ghost" onclick={cancelEdit}>Cancel</button>
                <button class="btn btn-sm btn-primary" onclick={saveEdit} disabled={!editTitle.trim()}>
                  <Save size={16} /> Save
                </button>
              </div>
            </div>
          {:else}
            <!-- View mode -->
            <div class="card-body flex-row gap-3 items-center">
              <button class="flex-1 flex gap-3 cursor-pointer bg-transparent border-0 p-0 text-left" onclick={() => detailBook = book}>
                {#if book.cover_url}
                  <img src={book.cover_url} alt={book.title} class="w-16 h-24 object-cover rounded" loading="lazy" />
                {:else}
                  <div class="w-16 h-24 bg-base-300 rounded flex items-center justify-center text-xs opacity-50">No cover</div>
                {/if}
                <div class="flex-1">
                  <h3 class="card-title text-base">{book.title}</h3>
                  <p class="text-sm opacity-70">
                    {book.authors.map(a => `${a.first_name} ${a.last_name}`).join(', ') || 'Unknown author'}
                  </p>
                  <div class="flex flex-wrap gap-1 mt-1">
                    {#each book.tags as tag}
                      <span class="badge badge-sm badge-ghost">{tag.name}</span>
                    {/each}
                  </div>
                  <p class="text-xs opacity-50 mt-1">{utcToLocalDate(book.scan_date)}</p>
                </div>
              </button>
              <div class="flex gap-1">
                <button class="btn btn-ghost btn-sm" onclick={() => startEdit(book)} title="Edit">
                  <Edit size={16} />
                </button>
                <button class="btn btn-ghost btn-sm" onclick={() => handleArchive(book.id)} title={book.archived ? "Unarchive" : "Archive"}>
                  {#if book.archived}
                    <ArchiveRestore size={16} />
                  {:else}
                    <Archive size={16} />
                  {/if}
                </button>
                <button class="btn btn-ghost btn-sm" onclick={() => deleteConfirmBook = book} title="Delete">
                  <Trash2 size={16} />
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if totalPages > 1}
      <div class="join mt-4 flex justify-center">
        <button class="join-item btn btn-sm" disabled={page <= 1} onclick={() => page--}>«</button>
        <span class="join-item btn btn-sm no-animation pointer-events-none">{page} / {totalPages}</span>
        <button class="join-item btn btn-sm" disabled={page >= totalPages} onclick={() => page++}>»</button>
      </div>
    {/if}
  {/if}
</section>

<!-- Book Detail Modal -->
{#if detailBook}
  <div class="modal modal-open">
    <div class="modal-box max-w-2xl">
      <button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2" onclick={() => detailBook = null}>✕</button>

      <div class="flex gap-4">
        {#if detailBook.cover_url}
          <img src={detailBook.cover_url} alt={detailBook.title} class="w-48 h-72 object-cover rounded shadow-lg" />
        {:else}
          <div class="w-48 h-72 bg-base-300 rounded flex items-center justify-center text-sm opacity-50">No cover</div>
        {/if}

        <div class="flex-1">
          <h2 class="text-2xl font-bold mb-2">{detailBook.title}</h2>
          <p class="text-lg mb-3">
            {detailBook.authors.map(a => `${a.first_name} ${a.last_name}`).join(', ') || 'Unknown author'}
          </p>

          <div class="mb-3">
            <span class="text-sm font-semibold opacity-60">Tags</span>
            <div class="flex flex-wrap gap-1 mt-1">
              {#each detailBook.tags as tag}
                <span class="badge badge-lg">{tag.name}</span>
              {/each}
            </div>
          </div>

          {#if detailBook.isbn}
            <p class="text-sm mb-1"><span class="font-semibold">ISBN:</span> {detailBook.isbn}</p>
          {/if}

          <p class="text-sm opacity-60">Scanned: {utcToLocalDateTime(detailBook.scan_date)}</p>
        </div>
      </div>
    </div>
    <button class="modal-backdrop" onclick={() => detailBook = null} aria-label="Close modal"></button>
  </div>
{/if}

<!-- Delete Confirmation Modal -->
{#if deleteConfirmBook}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Delete Book?</h3>
      <p class="py-4">Are you sure you want to delete <strong>{deleteConfirmBook.title}</strong>? This cannot be undone.</p>
      <div class="modal-action">
        <button class="btn btn-ghost" onclick={() => deleteConfirmBook = null}>Cancel</button>
        <button class="btn btn-error" onclick={() => handleDelete(deleteConfirmBook!.id)}>Delete</button>
      </div>
    </div>
    <button class="modal-backdrop" onclick={() => deleteConfirmBook = null} aria-label="Close modal"></button>
  </div>
{/if}
