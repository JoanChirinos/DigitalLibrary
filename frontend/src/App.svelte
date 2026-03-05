<script lang="ts">
  import { onMount } from 'svelte';
  import { loadBooks, loadTags } from './stores';
  import { Library, Sun, Moon } from 'lucide-svelte';
  import BookList from './components/BookList.svelte';
  import BookForm from './components/BookForm.svelte';
  import TagManager from './components/TagManager.svelte';
  import StatsGraph from './components/StatsGraph.svelte';

  const validTabs = ['library', 'tags', 'stats'];
  let activeTab = $state(validTabs.includes(location.hash.slice(1)) ? location.hash.slice(1) : 'library');
  let theme = $state('light');

  function setTab(tab: string) {
    activeTab = tab;
    location.hash = tab;
  }

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    document.cookie = `theme=${theme}; max-age=31536000; path=/`;
  }

  onMount(() => {
    const cookie = document.cookie.split('; ').find(c => c.startsWith('theme='));
    if (cookie) {
      theme = cookie.split('=')[1];
    } else {
      theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    loadBooks();
    loadTags();
  });
</script>

<div class="min-h-screen bg-base-200" data-theme={theme}>
  <div class="max-w-4xl mx-auto p-4">
    <div class="flex justify-between items-center mb-4">
      <h1 class="text-3xl font-bold flex items-center gap-2"><Library size={32} /> DigitalLibrary</h1>
      <button class="btn btn-ghost btn-sm" onclick={toggleTheme}>
        {#if theme === 'light'}
          <Moon size={20} />
        {:else}
          <Sun size={20} />
        {/if}
      </button>
    </div>

    <div class="tabs tabs-boxed mb-6">
      <button class="tab" class:tab-active={activeTab === 'library'} onclick={() => setTab('library')}>Library</button>
      <button class="tab" class:tab-active={activeTab === 'tags'} onclick={() => setTab('tags')}>Tags</button>
      <button class="tab" class:tab-active={activeTab === 'stats'} onclick={() => setTab('stats')}>Stats</button>
    </div>

    {#if activeTab === 'library'}
      <BookForm />
      <BookList />
    {:else if activeTab === 'tags'}
      <TagManager />
    {:else if activeTab === 'stats'}
      <StatsGraph />
    {/if}
  </div>
</div>
