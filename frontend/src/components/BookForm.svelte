<script lang="ts">
  import { books, tags, loadBooks, loadTags } from '../stores';
  import { createBook, createTag } from '../api';
  import type { Tag } from '../api';
  import { Plus, X } from 'lucide-svelte';
  import Fuse from 'fuse.js';

  let title = $state('');
  let scanDate = $state(new Date().toISOString().slice(0, 10));
  let isbn = $state('');
  let coverUrl = $state('');
  let firstName = $state('');
  let lastName = $state('');
  let authors = $state<{first_name: string; last_name: string}[]>([]);
  let selectedTagIds = $state<number[]>([]);
  let showSuggestions = $state(false);

  let newTagName = $state('');
  let newTagKind = $state('genre');
  let showNewTag = $state(false);

  let allAuthors = $derived(
    Object.values(
      $books.flatMap(b => b.authors).reduce((acc: Record<string, {first_name: string; last_name: string}>, a) => {
        acc[`${a.first_name}|${a.last_name}`] = { first_name: a.first_name, last_name: a.last_name };
        return acc;
      }, {})
    ).sort((a, b) => a.last_name.localeCompare(b.last_name))
  );

  let fuse = $derived(new Fuse(allAuthors, {
    keys: ['first_name', 'last_name'],
    threshold: 0.4,
  }));

  let authorSuggestions = $derived.by(() => {
    const q = `${firstName} ${lastName}`.trim();
    if (!q) return [];
    return fuse.search(q)
      .map(r => r.item)
      .filter(a => !authors.some(x => x.first_name === a.first_name && x.last_name === a.last_name))
      .slice(0, 8);
  });

  let tagsByKind = $derived(
    ($tags).reduce((acc: Record<string, Tag[]>, tag) => {
      (acc[tag.kind] ??= []).push(tag);
      return acc;
    }, {})
  );

  function addAuthor() {
    const fn = firstName.trim();
    const ln = lastName.trim();
    if (fn && ln && !authors.some(a => a.first_name === fn && a.last_name === ln)) {
      authors = [...authors, { first_name: fn, last_name: ln }];
    }
    firstName = '';
    lastName = '';
    showSuggestions = false;
  }

  function selectAuthor(a: {first_name: string; last_name: string}) {
    if (!authors.some(x => x.first_name === a.first_name && x.last_name === a.last_name)) {
      authors = [...authors, a];
    }
    firstName = '';
    lastName = '';
    showSuggestions = false;
  }

  function removeAuthor(a: {first_name: string; last_name: string}) {
    authors = authors.filter(x => x.first_name !== a.first_name || x.last_name !== a.last_name);
  }

  function toggleTag(id: number) {
    if (selectedTagIds.includes(id)) {
      selectedTagIds = selectedTagIds.filter(t => t !== id);
    } else {
      selectedTagIds = [...selectedTagIds, id];
    }
  }

  async function handleNewTag() {
    const name = newTagName.trim();
    if (!name) return;
    const tag = await createTag(name, newTagKind);
    await loadTags();
    selectedTagIds = [...selectedTagIds, tag.id];
    newTagName = '';
    showNewTag = false;
  }

  async function handleSubmit() {
    if (!title.trim()) return;
    await createBook({
      title: title.trim(),
      scan_date: scanDate,
      isbn: isbn || undefined,
      cover_url: coverUrl || undefined,
      authors,
      tag_ids: selectedTagIds,
    });
    await loadBooks();
    title = '';
    scanDate = new Date().toISOString().slice(0, 10);
    isbn = '';
    coverUrl = '';
    authors = [];
    firstName = '';
    lastName = '';
    selectedTagIds = [];
  }
</script>

<div class="collapse collapse-arrow bg-base-100 shadow mb-6">
  <input type="checkbox" />
  <div class="collapse-title text-lg font-semibold">Add Book</div>
  <div class="collapse-content">

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <input class="input input-bordered w-full" placeholder="Title" bind:value={title} />
      <input class="input input-bordered w-full" type="date" bind:value={scanDate} />
      <input class="input input-bordered w-full" placeholder="ISBN (optional)" bind:value={isbn} />
      <input class="input input-bordered w-full" placeholder="Cover URL (optional)" bind:value={coverUrl} />
    </div>

    <!-- Authors -->
    <div class="mt-2">
      <span class="text-sm font-semibold">Authors</span>
      <div class="relative flex gap-2 mt-1">
        <input
          class="input input-bordered input-sm flex-1"
          placeholder="First name"
          bind:value={firstName}
          onfocus={() => showSuggestions = true}
          onblur={() => setTimeout(() => showSuggestions = false, 200)}
          onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); addAuthor(); } }}
        />
        <input
          class="input input-bordered input-sm flex-1"
          placeholder="Last name"
          bind:value={lastName}
          onfocus={() => showSuggestions = true}
          onblur={() => setTimeout(() => showSuggestions = false, 200)}
          onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); addAuthor(); } }}
        />
        <button class="btn btn-sm btn-outline" onclick={addAuthor}>Add</button>
      </div>
      {#if showSuggestions && authorSuggestions.length > 0}
        <ul class="menu bg-base-100 shadow-lg rounded-box z-10 w-full mt-1 max-h-48 overflow-y-auto">
          {#each authorSuggestions as suggestion}
            <li><button onmousedown={() => selectAuthor(suggestion)}>{suggestion.first_name} {suggestion.last_name}</button></li>
          {/each}
        </ul>
      {/if}
      {#if authors.length > 0}
        <div class="flex flex-wrap gap-1 mt-2">
          {#each authors as author}
            <button class="badge badge-primary cursor-pointer" onclick={() => removeAuthor(author)}>
              {author.first_name} {author.last_name} <X size={12} />
            </button>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Tags -->
    <div class="mt-2">
      <span class="text-sm font-semibold">Tags</span>
      <div class="flex flex-wrap gap-4 mt-1">
        {#each Object.entries(tagsByKind) as [kind, kindTags]}
          <div>
            <span class="text-xs uppercase opacity-60">{kind}</span>
            <div class="flex flex-wrap gap-1 mt-1">
              {#each kindTags.sort((a, b) => a.name.localeCompare(b.name)) as tag}
                <button
                  class="badge badge-outline cursor-pointer"
                  class:badge-primary={selectedTagIds.includes(tag.id)}
                  onclick={() => toggleTag(tag.id)}
                >{tag.name}</button>
              {/each}
            </div>
          </div>
        {/each}
      </div>

      {#if showNewTag}
        <div class="flex gap-2 mt-2 items-center">
          <input class="input input-bordered input-sm" placeholder="Tag name" bind:value={newTagName} />
          <select class="select select-bordered select-sm" bind:value={newTagKind}>
            <option value="genre">genre</option>
            <option value="owner">owner</option>
            <option value="custom">custom</option>
          </select>
          <button class="btn btn-sm btn-primary" onclick={handleNewTag}>Create</button>
          <button class="btn btn-sm btn-ghost" onclick={() => showNewTag = false}>Cancel</button>
        </div>
      {:else}
        <button class="btn btn-xs btn-ghost mt-2" onclick={() => showNewTag = true}>
          <Plus size={14} /> New tag
        </button>
      {/if}
    </div>

    <div class="card-actions justify-end mt-3">
      <button class="btn btn-primary" onclick={handleSubmit} disabled={!title.trim()}>Add Book</button>
    </div>
  </div>
</div>
