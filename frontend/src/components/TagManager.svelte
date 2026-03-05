<script lang="ts">
  import { tags, loadTags } from '../stores';
  import { createTag, deleteTag } from '../api';
  import { Trash2 } from 'lucide-svelte';
  import type { Tag } from '../api';

  let newName = $state('');
  let newKind = $state('genre');
  let deleteConfirmTag = $state<Tag | null>(null);

  const kindOrder = ['owner', 'genre', 'custom'];

  let tagsByKind = $derived(
    ($tags).reduce((acc: Record<string, Tag[]>, tag) => {
      (acc[tag.kind] ??= []).push(tag);
      return acc;
    }, {})
  );

  let sortedKinds = $derived(
    Object.entries(tagsByKind).sort(([a], [b]) =>
      (kindOrder.indexOf(a) === -1 ? 99 : kindOrder.indexOf(a)) -
      (kindOrder.indexOf(b) === -1 ? 99 : kindOrder.indexOf(b))
    )
  );

  async function handleCreate() {
    const name = newName.trim();
    if (!name) return;
    await createTag(name, newKind);
    await loadTags();
    newName = '';
  }

  async function handleDelete(id: number) {
    await deleteTag(id);
    await loadTags();
    deleteConfirmTag = null;
  }
</script>

<section>
  <div class="flex gap-2 mb-4 items-end">
    <input class="input input-bordered input-sm" placeholder="Tag name" bind:value={newName}
      onkeydown={(e) => { if (e.key === 'Enter') handleCreate(); }} />
    <select class="select select-bordered select-sm" bind:value={newKind}>
      <option value="owner">owner</option>
      <option value="genre">genre</option>
      <option value="custom">custom</option>
    </select>
    <button class="btn btn-sm btn-primary" onclick={handleCreate} disabled={!newName.trim()}>Create</button>
  </div>

  {#each sortedKinds as [kind, kindTags]}
    <div class="mb-4">
      <h3 class="text-sm font-semibold uppercase opacity-60 mb-1">{kind}</h3>
      <div class="flex flex-wrap gap-2">
        {#each kindTags.sort((a, b) => a.name.localeCompare(b.name)) as tag}
          <div class="badge badge-lg gap-1">
            {tag.name}
            <button class="cursor-pointer opacity-50 hover:opacity-100" onclick={() => deleteConfirmTag = tag}>
              <Trash2 size={12} />
            </button>
          </div>
        {/each}
      </div>
    </div>
  {/each}
</section>

<!-- Delete Confirmation Modal -->
{#if deleteConfirmTag}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">Delete Tag?</h3>
      <p class="py-4">Are you sure you want to delete the <strong>{deleteConfirmTag.name}</strong> tag? It will be removed from all books.</p>
      <div class="modal-action">
        <button class="btn btn-ghost" onclick={() => deleteConfirmTag = null}>Cancel</button>
        <button class="btn btn-error" onclick={() => handleDelete(deleteConfirmTag!.id)}>Delete</button>
      </div>
    </div>
    <div class="modal-backdrop" onclick={() => deleteConfirmTag = null}></div>
  </div>
{/if}
