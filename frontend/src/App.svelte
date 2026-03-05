<script lang="ts">
  import { onMount } from 'svelte';
  import { loadBooks, loadTags } from './stores';
  import { Library } from 'lucide-svelte';
  import BookList from './components/BookList.svelte';
  import BookForm from './components/BookForm.svelte';
  import TagManager from './components/TagManager.svelte';
  import StatsGraph from './components/StatsGraph.svelte';

  const validTabs = ['library', 'tags', 'stats'];
  let activeTab = $state(validTabs.includes(location.hash.slice(1)) ? location.hash.slice(1) : 'library');

  function setTab(tab: string) {
    activeTab = tab;
    location.hash = tab;
  }

  onMount(() => {
    loadBooks();
    loadTags();
  });
</script>

<div class="min-h-screen bg-base-200" data-theme="light">
  <div class="max-w-4xl mx-auto p-4">
    <h1 class="text-3xl font-bold mb-4 flex items-center gap-2"><Library size={32} /> DigitalLibrary</h1>

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
