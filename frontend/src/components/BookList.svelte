<script lang="ts">
  import { books, tags } from '../stores';
  import { deleteBook } from '../api';
  import { loadBooks } from '../stores';
  import { Trash2 } from 'lucide-svelte';
  import type { Book, Tag } from '../api';

  let sortBy = $state<'title' | 'author' | 'recent'>('author');
  let selectedTags = $state<number[]>([]);

  // Group tags by kind for the filter UI
  let tagsByKind = $derived(
    ($tags).reduce((acc: Record<string, Tag[]>, tag) => {
      (acc[tag.kind] ??= []).push(tag);
      return acc;
    }, {})
  );

  let filteredBooks = $derived.by(() => {
    let result = [...$books];

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

  function toggleTag(id: number) {
    if (selectedTags.includes(id)) {
      selectedTags = selectedTags.filter(t => t !== id);
    } else {
      selectedTags = [...selectedTags, id];
    }
  }

  async function handleDelete(id: number) {
    await deleteBook(id);
    await loadBooks();
  }
</script>

<section>
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
        <div class="card card-compact bg-base-100 shadow">
          <div class="card-body flex-row justify-between items-center">
            <div>
              <h3 class="card-title text-base">{book.title}</h3>
              <p class="text-sm opacity-70">
                {book.authors.map(a => `${a.first_name} ${a.last_name}`).join(', ') || 'Unknown author'}
              </p>
              <div class="flex flex-wrap gap-1 mt-1">
                {#each book.tags as tag}
                  <span class="badge badge-sm badge-ghost">{tag.name}</span>
                {/each}
              </div>
              <p class="text-xs opacity-50 mt-1">{book.scan_date}</p>
            </div>
            <button class="btn btn-ghost btn-sm" onclick={() => handleDelete(book.id)}>
              <Trash2 size={16} />
            </button>
          </div>
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
