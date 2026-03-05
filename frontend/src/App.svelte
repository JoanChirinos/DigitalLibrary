<script lang="ts">
  import { onMount } from 'svelte';
  import { loadBooks, loadTags } from './stores';
  import { Library, Sun, Moon, LogOut } from 'lucide-svelte';
  import BookList from './components/BookList.svelte';
  import BookForm from './components/BookForm.svelte';
  import TagManager from './components/TagManager.svelte';
  import StatsGraph from './components/StatsGraph.svelte';
  import Login from './components/Login.svelte';

  const validTabs = ['library', 'tags', 'stats'];
  let activeTab = $state(validTabs.includes(location.hash.slice(1)) ? location.hash.slice(1) : 'library');
  let theme = $state('light');
  let isAuthenticated = $state(false);

  function setTab(tab: string) {
    activeTab = tab;
    location.hash = tab;
  }

  function toggleTheme() {
    theme = theme === 'light' ? 'dark' : 'light';
    document.cookie = `theme=${theme}; max-age=31536000; path=/`;
  }

  function logout() {
    document.cookie = 'auth_token=; max-age=0; path=/';
    window.location.reload();
  }

  onMount(() => {
    const themeCookie = document.cookie.split('; ').find(c => c.startsWith('theme='));
    if (themeCookie) {
      theme = themeCookie.split('=')[1];
    } else {
      theme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }

    const authCookie = document.cookie.split('; ').find(c => c.startsWith('auth_token='));
    isAuthenticated = !!authCookie;

    if (isAuthenticated) {
      loadBooks();
      loadTags();
    }
  });
</script>

{#if !isAuthenticated}
  <div data-theme={theme}>
    <Login />
  </div>
{:else}
  <div class="min-h-screen bg-base-200" data-theme={theme}>
    <div class="max-w-4xl mx-auto p-4">
      <div class="flex justify-between items-center mb-4">
        <h1 class="text-3xl font-bold flex items-center gap-2"><Library size={32} /> DigitalLibrary</h1>
        <div class="flex gap-1">
          <button class="btn btn-ghost btn-sm" onclick={toggleTheme} title="Toggle theme">
            {#if theme === 'light'}
              <Moon size={20} />
            {:else}
              <Sun size={20} />
            {/if}
          </button>
          <button class="btn btn-ghost btn-sm" onclick={logout} title="Logout">
            <LogOut size={20} />
          </button>
        </div>
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
{/if}
