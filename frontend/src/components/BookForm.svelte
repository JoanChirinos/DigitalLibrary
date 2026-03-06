<script lang="ts">
  import { books, tags, loadBooks, loadTags } from '../stores';
  import { createBook, createTag, lookupISBN } from '../api';
  import type { Tag } from '../api';
  import { Plus, X, Search, Camera } from 'lucide-svelte';
  import Fuse from 'fuse.js';
  import { onMount, tick } from 'svelte';
  import { Html5Qrcode } from 'html5-qrcode';

  function nowLocal() {
    const now = new Date();
    const offset = now.getTimezoneOffset() * 60000;
    return new Date(now.getTime() - offset).toISOString().slice(0, 19);
  }

  function localToUTC(localDatetime: string): string {
    return new Date(localDatetime).toISOString().slice(0, 19) + 'Z';
  }

  function utcToLocal(utcDatetime: string): string {
    const date = new Date(utcDatetime.endsWith('Z') ? utcDatetime : utcDatetime + 'Z');
    const offset = date.getTimezoneOffset() * 60000;
    return new Date(date.getTime() - offset).toISOString().slice(0, 19);
  }

  let title = $state('');
  let scanDate = $state(nowLocal());
  let isbn = $state('');
  let coverUrl = $state('');
  let firstName = $state('');
  let lastName = $state('');
  let authors = $state<{first_name: string; last_name: string}[]>([]);
  let selectedTagIds = $state<number[]>([]);
  let showSuggestions = $state(false);
  let selectedSuggestionIndex = $state(-1);

  let newTagName = $state('');
  let newTagKind = $state('genre');
  let showNewTag = $state(false);

  // ISBN lookup state
  let isbnInput = $state('');
  let isLookingUp = $state(false);
  let lookupError = $state('');
  let suggestedSubjects = $state<string[]>([]);
  let lastLookupTime = $state<number | null>(null);
  let now = $state(Date.now());

  // Barcode scanner state
  let isScanning = $state(false);
  let scanError = $state('');
  let scanner: Html5Qrcode | null = null;

  onMount(() => {
    const cookie = document.cookie.split('; ').find(c => c.startsWith('lastISBNLookup='));
    if (cookie) {
      lastLookupTime = parseInt(cookie.split('=')[1]);
    }

    const interval = setInterval(() => {
      now = Date.now();
    }, 1000);

    return () => clearInterval(interval);
  });

  let canLookup = $derived(lastLookupTime === null || now - lastLookupTime >= 5000);
  let timeRemaining = $derived(lastLookupTime ? Math.max(0, 5 - Math.floor((now - lastLookupTime) / 1000)) : 0);

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

  async function startScanner() {
    scanError = '';
    isScanning = true;
    await tick(); // Wait for DOM to update
    scanner = new Html5Qrcode('barcode-reader');

    try {
      await scanner.start(
        { facingMode: 'environment' },
        { fps: 10, qrbox: { width: 250, height: 250 } },
        (decodedText) => {
          if (/^\d{13}$/.test(decodedText) && (decodedText.startsWith('978') || decodedText.startsWith('979'))) {
            isbnInput = decodedText;
            stopScanner();
            if (canLookup) handleISBNLookup();
          }
        },
        undefined
      );
    } catch (e: any) {
      scanError = e.message?.includes('NotAllowedError') ? 'Camera permission denied' : 'Failed to start camera';
      isScanning = false;
    }
  }

  async function stopScanner() {
    isScanning = false;
    if (scanner) {
      try {
        const state = await scanner.getState();
        if (state === 2) { // 2 = scanning
          await scanner.stop();
        }
        scanner.clear();
      } catch (e) {
        // Ignore cleanup errors
      }
      scanner = null;
    }
  }

  async function handleISBNLookup() {
    lookupError = '';
    suggestedSubjects = [];
    isLookingUp = true;

    try {
      const result = await lookupISBN(isbnInput);

      // Auto-fill fields
      title = result.title;
      scanDate = nowLocal();
      isbn = isbnInput;
      coverUrl = result.coverUrl || '';

      // Parse and match authors
      const authorFuse = new Fuse(allAuthors, { keys: ['first_name', 'last_name'], threshold: 0.5, includeScore: true });
      for (let i = 0; i < result.authors.length; i++) {
        const name = result.authors[i].trim();
        const parts = name.split(/\s+/);
        const lastNameParsed = parts.pop() || '';
        const firstNameParsed = parts.join(' ') || '';

        // Try fuzzy match
        const matches = authorFuse.search(`${firstNameParsed} ${lastNameParsed}`);

        if (matches.length > 0 && matches[0].score! <= 0.5) {
          // Good match found, add it
          const match = matches[0].item;
          if (!authors.some(a => a.first_name === match.first_name && a.last_name === match.last_name)) {
            authors = [...authors, match];
          }
        } else if (parts.length === 0) {
          // Simple two-word name, auto-add
          if (!authors.some(a => a.first_name === firstNameParsed && a.last_name === lastNameParsed)) {
            authors = [...authors, { first_name: firstNameParsed, last_name: lastNameParsed }];
          }
        } else {
          // Complex name (3+ words), pre-fill fields for first author only
          if (i === 0 && authors.length === 0) {
            firstName = firstNameParsed;
            lastName = lastNameParsed;
          }
        }
      }

      // Auto-select matching genre tags
      const genreTags = ($tags).filter(t => t.kind === 'genre');
      for (const subject of result.subjects) {
        const match = genreTags.find(t => t.name.toLowerCase() === subject.toLowerCase());
        if (match && !selectedTagIds.includes(match.id)) {
          selectedTagIds = [...selectedTagIds, match.id];
        }
      }

      // Store non-matching subjects as suggestions (lowercase)
      const matched = result.subjects.filter(s =>
        genreTags.some(t => t.name.toLowerCase() === s.toLowerCase())
      );
      suggestedSubjects = result.subjects.filter(s => !matched.includes(s)).map(s => s.toLowerCase());

      // Set cookie
      document.cookie = `lastISBNLookup=${Date.now()}; max-age=300`;
      lastLookupTime = Date.now();
    } catch (e: any) {
      lookupError = e.message || 'Lookup failed';
    } finally {
      isLookingUp = false;
    }
  }

  async function addSuggestedGenre(subject: string) {
    const tag = await createTag(subject, 'genre');
    await loadTags();
    selectedTagIds = [...selectedTagIds, tag.id];
    suggestedSubjects = suggestedSubjects.filter(s => s !== subject);
  }

  function addAuthor() {
    const fn = firstName.trim();
    const ln = lastName.trim();
    if (fn && ln && !authors.some(a => a.first_name === fn && a.last_name === ln)) {
      authors = [...authors, { first_name: fn, last_name: ln }];
    }
    firstName = '';
    lastName = '';
    showSuggestions = false;
    selectedSuggestionIndex = -1;
  }

  function selectAuthor(a: {first_name: string; last_name: string}) {
    if (!authors.some(x => x.first_name === a.first_name && x.last_name === a.last_name)) {
      authors = [...authors, a];
    }
    firstName = '';
    lastName = '';
    showSuggestions = false;
    selectedSuggestionIndex = -1;
  }

  function handleAuthorKeydown(e: KeyboardEvent) {
    if (!showSuggestions || authorSuggestions.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedSuggestionIndex = Math.min(selectedSuggestionIndex + 1, authorSuggestions.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedSuggestionIndex = Math.max(selectedSuggestionIndex - 1, -1);
    } else if (e.key === 'Enter' && selectedSuggestionIndex >= 0) {
      e.preventDefault();
      selectAuthor(authorSuggestions[selectedSuggestionIndex]);
    }
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
    const finalName = newTagKind === 'genre' ? name.toLowerCase() : name;
    const tag = await createTag(finalName, newTagKind);
    await loadTags();
    selectedTagIds = [...selectedTagIds, tag.id];
    newTagName = '';
    showNewTag = false;
  }

  async function handleSubmit() {
    if (!title.trim()) return;
    await createBook({
      title: title.trim(),
      scan_date: localToUTC(scanDate),
      isbn: isbn || undefined,
      cover_url: coverUrl || undefined,
      authors,
      tag_ids: selectedTagIds,
    });
    await loadBooks();
    title = '';
    scanDate = nowLocal();
    isbn = '';
    coverUrl = '';
    authors = [];
    firstName = '';
    lastName = '';
    selectedTagIds = [];
    isbnInput = '';
    lookupError = '';
    suggestedSubjects = [];
  }
</script>

<div class="collapse collapse-arrow bg-base-100 shadow mb-6">
  <input type="checkbox" />
  <div class="collapse-title text-lg font-semibold">Add Book</div>
  <div class="collapse-content">

    <!-- ISBN Lookup -->
    <div class="mb-4 p-3 bg-base-200 rounded-lg">
      <div class="flex gap-2 items-end">
        <div class="flex-1">
          <label class="text-sm font-semibold" for="isbn-lookup-input">ISBN Lookup</label>
          <input
            id="isbn-lookup-input"
            class="input input-bordered input-sm w-full mt-1"
            placeholder="Enter ISBN"
            bind:value={isbnInput}
            onkeydown={(e) => { if (e.key === 'Enter' && canLookup && isbnInput.trim()) handleISBNLookup(); }}
          />
        </div>
        <button
          class="btn btn-sm btn-outline"
          onclick={startScanner}
        >
          <Camera size={16} /> Scan
        </button>
        <button
          class="btn btn-sm btn-primary"
          disabled={!canLookup || !isbnInput.trim() || isLookingUp}
          onclick={handleISBNLookup}
        >
          {#if isLookingUp}
            <span class="loading loading-spinner loading-xs"></span>
          {:else if !canLookup}
            Wait {timeRemaining}s
          {:else}
            <Search size={16} /> Lookup
          {/if}
        </button>
      </div>
      {#if lookupError}
        <div class="alert alert-sm mt-2" class:alert-error={lookupError === 'ISBN not found'} class:alert-warning={lookupError === 'Rate limited'} class:alert-info={lookupError !== 'ISBN not found' && lookupError !== 'Rate limited'}>
          {lookupError}
        </div>
      {/if}
      {#if suggestedSubjects.length > 0}
        <div class="mt-2">
          <span class="text-xs opacity-60">Suggested genres from Open Library:</span>
          <div class="flex flex-wrap gap-1 mt-1">
            {#each suggestedSubjects as subject}
              <button class="badge badge-outline badge-sm cursor-pointer" onclick={() => addSuggestedGenre(subject)}>
                + {subject}
              </button>
            {/each}
          </div>
        </div>
      {/if}
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <input class="input input-bordered w-full" placeholder="Title" bind:value={title} />
      <input
        class="input input-bordered w-full"
        type="datetime-local"
        bind:value={scanDate}
        onfocus={() => scanDate = nowLocal()}
      />
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
          onkeydown={handleAuthorKeydown}
        />
        <input
          class="input input-bordered input-sm flex-1"
          placeholder="Last name"
          bind:value={lastName}
          onfocus={() => showSuggestions = true}
          onblur={() => setTimeout(() => showSuggestions = false, 200)}
          onkeydown={handleAuthorKeydown}
        />
        <button class="btn btn-sm btn-outline" onclick={addAuthor}>Add</button>
      </div>
      {#if showSuggestions && authorSuggestions.length > 0}
        <ul class="menu bg-base-100 shadow-lg rounded-box z-10 w-full mt-1 max-h-48 overflow-y-auto">
          {#each authorSuggestions as suggestion, i}
            <li class:bg-base-200={i === selectedSuggestionIndex}><button onmousedown={() => selectAuthor(suggestion)}>{suggestion.first_name} {suggestion.last_name}</button></li>
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

<!-- Barcode Scanner Modal -->
{#if isScanning}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg mb-2">Scan Barcode</h3>
      <p class="text-sm opacity-70 mb-3">Point camera at ISBN barcode</p>
      <div id="barcode-reader" class="w-full"></div>
      {#if scanError}
        <div class="alert alert-error alert-sm mt-2">{scanError}</div>
      {/if}
      <div class="modal-action">
        <button class="btn btn-sm" onclick={stopScanner}>Cancel</button>
      </div>
    </div>
  </div>
{/if}
