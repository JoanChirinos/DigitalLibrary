<script lang="ts">
  import { books, tags, loadBooks, loadTags } from '../stores';
  import { createBook, createTag, lookupISBN, searchByTitle } from '../api';
  import type { Tag, TitleSearchResult } from '../api';
  import { Plus, X, Search, Camera, Flashlight } from 'lucide-svelte';
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

  type AuthorName = { first_name: string; middle_name?: string | null; last_name: string };
  const sameAuthor = (a: AuthorName, b: AuthorName) =>
    a.first_name === b.first_name && (a.middle_name || '') === (b.middle_name || '') && a.last_name === b.last_name;
  const authorLabel = (a: AuthorName) => [a.first_name, a.middle_name, a.last_name].filter(Boolean).join(' ');

  let title = $state('');
  let scanDate = $state(nowLocal());
  let isbn = $state('');
  let coverUrl = $state('');
  let firstName = $state('');
  let middleName = $state('');
  let lastName = $state('');
  let authors = $state<{first_name: string; middle_name?: string; last_name: string}[]>([]);
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

  // Title lookup state
  let titleQuery = $state('');
  let titleResults = $state<TitleSearchResult[]>([]);
  let isSearchingTitle = $state(false);
  let titleSearchError = $state('');

  // Barcode scanner state
  let isScanning = $state(false);
  let scanError = $state('');
  let scanner: Html5Qrcode | null = null;
  let canZoom = $state(false);
  let canTorch = $state(false);
  let zoom2x = $state(false);
  let torchOn = $state(false);

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
      $books.flatMap(b => b.authors).reduce((acc: Record<string, AuthorName>, a) => {
        acc[`${a.first_name}|${a.middle_name ?? ''}|${a.last_name}`] = { first_name: a.first_name, middle_name: a.middle_name ?? undefined, last_name: a.last_name };
        return acc;
      }, {})
    ).sort((a, b) => a.last_name.localeCompare(b.last_name))
  );

  let fuse = $derived(new Fuse(allAuthors, {
    keys: ['first_name', 'middle_name', 'last_name'],
    threshold: 0.4,
  }));

  let authorSuggestions = $derived.by(() => {
    const q = `${firstName} ${middleName} ${lastName}`.trim();
    if (!q) return [];
    return fuse.search(q)
      .map(r => r.item)
      .filter(a => !authors.some(x => sameAuthor(x, a)))
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
    canZoom = false;
    canTorch = false;
    zoom2x = false;
    torchOn = false;
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
      try {
        const caps = scanner.getRunningTrackCameraCapabilities();
        canZoom = caps.zoomFeature().isSupported() && caps.zoomFeature().max() >= 2;
        canTorch = caps.torchFeature().isSupported();
      } catch {
        // Capabilities unavailable on this device; leave toggles hidden.
      }
    } catch (e: any) {
      scanError = e.message?.includes('NotAllowedError') ? 'Camera permission denied' : 'Failed to start camera';
      isScanning = false;
    }
  }

  async function toggleZoom() {
    if (!scanner) return;
    const zf = scanner.getRunningTrackCameraCapabilities().zoomFeature();
    if (!zf.isSupported()) return;
    zoom2x = !zoom2x;
    const target = zoom2x ? Math.min(2, zf.max()) : Math.max(1, zf.min());
    try {
      await zf.apply(target);
    } catch {
      // Ignore; some devices reject mid-stream zoom changes.
    }
  }

  async function toggleTorch() {
    if (!scanner) return;
    const tf = scanner.getRunningTrackCameraCapabilities().torchFeature();
    if (!tf.isSupported()) return;
    torchOn = !torchOn;
    try {
      await tf.apply(torchOn);
    } catch {
      // Ignore; some devices reject mid-stream torch changes.
    }
  }

  async function stopScanner() {
    isScanning = false;
    canZoom = false;
    canTorch = false;
    zoom2x = false;
    torchOn = false;
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

  function mergeOpenLibraryAuthors(names: string[]) {
    const authorFuse = new Fuse(allAuthors, { keys: ['first_name', 'middle_name', 'last_name'], threshold: 0.5, includeScore: true });
    for (let i = 0; i < names.length; i++) {
      const name = names[i].trim();
      const tokens = name.split(/\s+/).filter(Boolean);
      const lastNameParsed = tokens.length > 1 ? tokens[tokens.length - 1] : (tokens[0] || '');
      const firstNameParsed = tokens.length > 1 ? tokens[0] : '';
      const middleNameParsed = tokens.length > 2 ? tokens.slice(1, -1).join(' ') : '';

      // Try fuzzy match
      const matches = authorFuse.search(`${firstNameParsed} ${middleNameParsed} ${lastNameParsed}`.replace(/\s+/g, ' ').trim());

      if (matches.length > 0 && matches[0].score! <= 0.5) {
        // Good match found, add it
        const match = matches[0].item;
        if (!authors.some(a => sameAuthor(a, match))) {
          authors = [...authors, { first_name: match.first_name, middle_name: match.middle_name ?? undefined, last_name: match.last_name }];
        }
      } else if (tokens.length <= 1) {
        // Single-token name, auto-add
        const candidate = { first_name: firstNameParsed, last_name: lastNameParsed };
        if (!authors.some(a => sameAuthor(a, candidate))) {
          authors = [...authors, candidate];
        }
      } else {
        // Multi-word name, pre-fill fields for first author only
        if (i === 0 && authors.length === 0) {
          firstName = firstNameParsed;
          middleName = middleNameParsed;
          lastName = lastNameParsed;
        }
      }
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

      mergeOpenLibraryAuthors(result.authors);

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

  async function handleTitleSearch() {
    const q = titleQuery.trim();
    if (!q || !canLookup) return;
    titleSearchError = '';
    isSearchingTitle = true;
    try {
      titleResults = await searchByTitle(q);
      if (titleResults.length === 0) titleSearchError = 'No results found';
      document.cookie = `lastISBNLookup=${Date.now()}; max-age=300`;
      lastLookupTime = Date.now();
    } catch (e: any) {
      titleSearchError = e.message || 'Search failed';
    } finally {
      isSearchingTitle = false;
    }
  }

  async function selectTitleResult(r: TitleSearchResult) {
    titleResults = [];
    titleSearchError = '';
    if (r.isbn) {
      isbnInput = r.isbn;
      await handleISBNLookup();
      if (title) return;
      // ISBN lookup came up empty; fall back to the search data.
    }
    title = r.title;
    scanDate = nowLocal();
    coverUrl = r.coverUrl || '';
    mergeOpenLibraryAuthors(r.authors);
  }

  async function addSuggestedGenre(subject: string) {
    const tag = await createTag(subject, 'genre');
    await loadTags();
    selectedTagIds = [...selectedTagIds, tag.id];
    suggestedSubjects = suggestedSubjects.filter(s => s !== subject);
  }

  function addAuthor() {
    const fn = firstName.trim();
    const mn = middleName.trim();
    const ln = lastName.trim();
    const candidate = { first_name: fn, middle_name: mn || undefined, last_name: ln };
    if (fn && ln && !authors.some(a => sameAuthor(a, candidate))) {
      authors = [...authors, candidate];
    }
    firstName = '';
    middleName = '';
    lastName = '';
    showSuggestions = false;
    selectedSuggestionIndex = -1;
  }

  function selectAuthor(a: AuthorName) {
    if (!authors.some(x => sameAuthor(x, a))) {
      authors = [...authors, { first_name: a.first_name, middle_name: a.middle_name ?? undefined, last_name: a.last_name }];
    }
    firstName = '';
    middleName = '';
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

  function removeAuthor(a: AuthorName) {
    authors = authors.filter(x => !sameAuthor(x, a));
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
    middleName = '';
    lastName = '';
    selectedTagIds = [];
    isbnInput = '';
    lookupError = '';
    suggestedSubjects = [];
    titleQuery = '';
    titleResults = [];
    titleSearchError = '';
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

    <!-- Title Lookup -->
    <div class="mb-4 p-3 bg-base-200 rounded-lg">
      <div class="flex gap-2 items-end">
        <div class="flex-1">
          <label class="text-sm font-semibold" for="title-lookup-input">Title Lookup</label>
          <input
            id="title-lookup-input"
            class="input input-bordered input-sm w-full mt-1"
            placeholder="Search by title"
            bind:value={titleQuery}
            onkeydown={(e) => { if (e.key === 'Enter' && canLookup && titleQuery.trim()) handleTitleSearch(); }}
          />
        </div>
        <button
          class="btn btn-sm btn-primary"
          disabled={!canLookup || !titleQuery.trim() || isSearchingTitle}
          onclick={handleTitleSearch}
        >
          {#if isSearchingTitle}
            <span class="loading loading-spinner loading-xs"></span>
          {:else if !canLookup}
            Wait {timeRemaining}s
          {:else}
            <Search size={16} /> Search
          {/if}
        </button>
      </div>
      {#if titleSearchError}
        <div class="alert alert-sm mt-2" class:alert-warning={titleSearchError === 'Rate limited'} class:alert-info={titleSearchError !== 'Rate limited'}>
          {titleSearchError}
        </div>
      {/if}
      {#if titleResults.length > 0}
        <ul class="menu bg-base-100 rounded-box mt-2 max-h-72 overflow-y-auto">
          {#each titleResults as result}
            <li>
              <button class="flex gap-3 items-center text-left w-full" onclick={() => selectTitleResult(result)}>
                {#if result.coverUrl}
                  <img src={result.coverUrl} alt={result.title} class="w-10 h-14 object-cover rounded" loading="lazy" />
                {:else}
                  <div class="w-10 h-14 bg-base-300 rounded flex items-center justify-center text-[10px] opacity-50">No cover</div>
                {/if}
                <span class="flex-1">
                  <span class="font-semibold">{result.title}</span>
                  <span class="block text-xs opacity-70">
                    {result.authors.join(', ') || 'Unknown author'}{result.year ? ` · ${result.year}` : ''}
                  </span>
                </span>
              </button>
            </li>
          {/each}
        </ul>
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
          placeholder="Middle (optional)"
          bind:value={middleName}
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
            <li class:bg-base-200={i === selectedSuggestionIndex}><button onmousedown={() => selectAuthor(suggestion)}>{authorLabel(suggestion)}</button></li>
          {/each}
        </ul>
      {/if}
      {#if authors.length > 0}
        <div class="flex flex-wrap gap-1 mt-2">
          {#each authors as author}
            <button class="badge badge-primary cursor-pointer" onclick={() => removeAuthor(author)}>
              {authorLabel(author)} <X size={12} />
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
      {#if canZoom || canTorch}
        <div class="flex gap-2 mt-3 justify-center">
          {#if canZoom}
            <button class="btn btn-sm" class:btn-active={zoom2x} onclick={toggleZoom}>
              {zoom2x ? '2x' : '1x'}
            </button>
          {/if}
          {#if canTorch}
            <button class="btn btn-sm" class:btn-active={torchOn} onclick={toggleTorch}>
              <Flashlight size={16} /> {torchOn ? 'On' : 'Off'}
            </button>
          {/if}
        </div>
      {/if}
      {#if scanError}
        <div class="alert alert-error alert-sm mt-2">{scanError}</div>
      {/if}
      <div class="modal-action">
        <button class="btn btn-sm" onclick={stopScanner}>Cancel</button>
      </div>
    </div>
  </div>
{/if}
