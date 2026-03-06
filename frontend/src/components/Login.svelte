<script lang="ts">
  import { onMount } from 'svelte';
  import { Library, Plus } from 'lucide-svelte';

  interface LibraryCard {
    id: number;
    name: string;
  }

  let libraries = $state<LibraryCard[]>([]);
  let selectedLibrary = $state<LibraryCard | null>(null);
  let passkey = $state('');
  let newLibraryName = $state('');
  let newLibraryPasskey = $state('');
  let showCreate = $state(false);
  let error = $state('');
  let isLoading = $state(false);
  let passkeyInput = $state<HTMLInputElement>();

  onMount(async () => {
    const base = import.meta.env.DEV ? 'http://localhost:8008' : '/api';
    const res = await fetch(`${base}/auth/libraries`);
    libraries = await res.json();
  });

  $effect(() => {
    if (selectedLibrary && passkeyInput) {
      passkeyInput.focus();
    }
  });

  async function handleLogin() {
    if (!selectedLibrary || !passkey) return;
    error = '';
    isLoading = true;

    try {
      const base = import.meta.env.DEV ? 'http://localhost:8008' : '/api';
      const res = await fetch(`${base}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: selectedLibrary.name, passkey }),
      });

      if (!res.ok) {
        error = res.status === 401 ? 'Invalid passkey' : 'Login failed';
        isLoading = false;
        return;
      }

      const data = await res.json();
      document.cookie = `auth_token=${data.token}; max-age=2592000; path=/`;
      window.location.reload();
    } catch (e) {
      error = 'Connection failed';
      isLoading = false;
    }
  }

  async function handleCreate() {
    if (!newLibraryName.trim() || !newLibraryPasskey) return;
    error = '';
    isLoading = true;

    try {
      const base = import.meta.env.DEV ? 'http://localhost:8008' : '/api';
      const res = await fetch(`${base}/auth/create`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: newLibraryName.trim(), passkey: newLibraryPasskey }),
      });

      if (!res.ok) {
        error = res.status === 409 ? 'Library name already exists' : 'Creation failed';
        isLoading = false;
        return;
      }

      const data = await res.json();
      document.cookie = `auth_token=${data.token}; max-age=2592000; path=/`;
      window.location.reload();
    } catch (e) {
      error = 'Connection failed';
      isLoading = false;
    }
  }
</script>

<div class="min-h-screen bg-base-200 flex items-center justify-center p-4">
  <div class="card bg-base-100 shadow-xl w-full max-w-md">
    <div class="card-body">
      <h1 class="card-title text-2xl flex items-center gap-2 justify-center mb-4">
        <Library size={32} /> DigitalLibrary
      </h1>

      {#if error}
        <div class="alert alert-error alert-sm">{error}</div>
      {/if}

      {#if showCreate}
        <!-- Create Library -->
        <h2 class="text-lg font-semibold mb-2">Create Library</h2>
        <input class="input input-bordered w-full mb-2" placeholder="Library name" bind:value={newLibraryName} />
        <input class="input input-bordered w-full mb-3" type="password" placeholder="Passkey" bind:value={newLibraryPasskey} />
        <div class="flex gap-2">
          <button class="btn btn-ghost flex-1" onclick={() => showCreate = false}>Cancel</button>
          <button class="btn btn-primary flex-1" onclick={handleCreate} disabled={isLoading || !newLibraryName.trim() || !newLibraryPasskey}>
            {#if isLoading}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              Create
            {/if}
          </button>
        </div>
      {:else if selectedLibrary}
        <!-- Login -->
        <h2 class="text-lg font-semibold mb-2">Login to {selectedLibrary.name}</h2>
        <input
          bind:this={passkeyInput}
          class="input input-bordered w-full mb-3"
          type="password"
          placeholder="Passkey"
          bind:value={passkey}
          onkeydown={(e) => { if (e.key === 'Enter' && passkey) handleLogin(); }}
        />
        <div class="flex gap-2">
          <button class="btn btn-ghost flex-1" onclick={() => { selectedLibrary = null; passkey = ''; }}>Back</button>
          <button class="btn btn-primary flex-1" onclick={handleLogin} disabled={isLoading || !passkey}>
            {#if isLoading}
              <span class="loading loading-spinner loading-sm"></span>
            {:else}
              Login
            {/if}
          </button>
        </div>
      {:else}
        <!-- Select Library -->
        <h2 class="text-lg font-semibold mb-2">Select Library</h2>
        <div class="flex flex-col gap-2 mb-3">
          {#each libraries as lib}
            <button class="btn btn-outline justify-start" onclick={() => selectedLibrary = lib}>
              <Library size={20} /> {lib.name}
            </button>
          {/each}
        </div>
        <button class="btn btn-sm btn-ghost w-full" onclick={() => showCreate = true}>
          <Plus size={16} /> Create New Library
        </button>
      {/if}
    </div>
  </div>
</div>
