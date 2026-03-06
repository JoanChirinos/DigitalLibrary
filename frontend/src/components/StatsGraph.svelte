<script lang="ts">
  import { onMount } from 'svelte';
  import { Chart, BarController, LineController, BarElement, LineElement, PointElement, CategoryScale, LinearScale, Tooltip, Legend } from 'chart.js';
  import { fetchTotals, fetchByTag, fetchByAuthor, fetchGrowth } from '../api';
  import { tags, showArchived, books } from '../stores';
  import type { Totals, TagCount, AuthorCount, GrowthBucket, Tag } from '../api';

  Chart.register(BarController, LineController, BarElement, LineElement, PointElement, CategoryScale, LinearScale, Tooltip, Legend);

  let totals = $state<Totals>({ books: 0, authors: 0, tags: 0 });
  let kindFilter = $state('');
  let groupBy = $state('month');
  let filterTagIds = $state<number[]>([]);

  const kindOrder = ['owner', 'genre', 'custom'];
  let tagsByKind = $derived(
    Object.entries(
      ($tags).reduce((acc: Record<string, Tag[]>, tag) => {
        (acc[tag.kind] ??= []).push(tag);
        return acc;
      }, {})
    ).sort(([a], [b]) =>
      (kindOrder.indexOf(a) === -1 ? 99 : kindOrder.indexOf(a)) -
      (kindOrder.indexOf(b) === -1 ? 99 : kindOrder.indexOf(b))
    )
  );

  function toggleFilterTag(id: number) {
    if (filterTagIds.includes(id)) {
      filterTagIds = filterTagIds.filter(t => t !== id);
    } else {
      filterTagIds = [...filterTagIds, id];
    }
  }

  let tagChart: Chart | null = null;
  let authorChart: Chart | null = null;
  let growthChart: Chart | null = null;

  let tagCanvas: HTMLCanvasElement;
  let authorCanvas: HTMLCanvasElement;
  let growthCanvas: HTMLCanvasElement;

  async function loadTotals() {
    totals = await fetchTotals(filterTagIds.length ? filterTagIds : undefined, undefined, undefined, $showArchived ? undefined : false);
  }

  async function loadTagChart() {
    const raw = await fetchByTag(kindFilter || undefined, filterTagIds.length ? filterTagIds : undefined, undefined, undefined, $showArchived ? undefined : false);
    const filterNames = ($tags).filter(t => filterTagIds.includes(t.id)).map(t => t.tag_name ?? t.name);
    const data = raw.filter(d => !filterTagIds.some(id => {
      const t = ($tags).find(t => t.id === id);
      return t && t.name === d.tag_name;
    }));
    tagChart?.destroy();
    tagChart = new Chart(tagCanvas, {
      type: 'bar',
      data: {
        labels: data.map(d => d.tag_name),
        datasets: [{
          label: 'Books',
          data: data.map(d => d.count),
          backgroundColor: 'oklch(0.65 0.15 240)',
        }],
      },
      options: {
        indexAxis: 'y',
        responsive: true,
        plugins: { legend: { display: false } },
        scales: { x: { beginAtZero: true, ticks: { stepSize: 1 } } },
      },
    });
  }

  async function loadAuthorChart() {
    const data = await fetchByAuthor(filterTagIds.length ? filterTagIds : undefined, undefined, undefined, $showArchived ? undefined : false);
    const top20 = data.slice(0, 20);
    authorChart?.destroy();
    authorChart = new Chart(authorCanvas, {
      type: 'bar',
      data: {
        labels: top20.map(d => `${d.first_name} ${d.last_name}`),
        datasets: [{
          label: 'Books',
          data: top20.map(d => d.count),
          backgroundColor: 'oklch(0.65 0.15 160)',
        }],
      },
      options: {
        indexAxis: 'y',
        responsive: true,
        plugins: { legend: { display: false } },
        scales: { x: { beginAtZero: true, ticks: { stepSize: 1 } } },
      },
    });
  }

  async function loadGrowthChart() {
    const data = await fetchGrowth(groupBy, filterTagIds.length ? filterTagIds : undefined, undefined, undefined, $showArchived ? undefined : false);
    growthChart?.destroy();
    growthChart = new Chart(growthCanvas, {
      type: 'bar',
      data: {
        labels: data.map(d => d.period),
        datasets: [{
          label: 'Books added',
          data: data.map(d => d.count),
          backgroundColor: 'oklch(0.65 0.15 30)',
        }],
      },
      options: {
        responsive: true,
        animation: false,
        plugins: { legend: { display: false } },
        scales: { y: { beginAtZero: true, ticks: { stepSize: 1 } } },
      },
    });
  }

  onMount(() => {
    loadTotals();
    loadTagChart();
    loadAuthorChart();
    loadGrowthChart();
  });

  $effect(() => {
    kindFilter;
    groupBy;
    filterTagIds;
    $showArchived;
    if (tagCanvas) {
      Promise.all([
        loadTotals(),
        loadTagChart(),
        loadAuthorChart(),
        loadGrowthChart(),
      ]);
    }
  });
</script>

<section>
  <!-- Tag filters -->
  <div class="card bg-base-100 shadow mb-4">
    <div class="card-body py-3">
      <div class="flex justify-between items-center mb-2">
        <span class="text-sm font-semibold">Filter by tags</span>
        <label class="label cursor-pointer gap-2 justify-start">
          <input type="checkbox" class="checkbox checkbox-sm" bind:checked={$showArchived} />
          <span class="label-text">Show archived</span>
        </label>
      </div>
      <div class="flex flex-wrap gap-3 mt-1">
        {#each tagsByKind as [kind, kindTags]}
          <div>
            <span class="text-xs uppercase opacity-60">{kind}</span>
            <div class="flex flex-wrap gap-1 mt-1">
              {#each kindTags.sort((a, b) => a.name.localeCompare(b.name)) as tag}
                <button
                  class="badge badge-outline badge-sm cursor-pointer"
                  class:badge-primary={filterTagIds.includes(tag.id)}
                  onclick={() => toggleFilterTag(tag.id)}
                >{tag.name}</button>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>

  <!-- Totals -->
  <div class="stats shadow mb-6 w-full">
    <div class="stat">
      <div class="stat-title">Books</div>
      <div class="stat-value">{totals.books}</div>
    </div>
    <div class="stat">
      <div class="stat-title">Authors</div>
      <div class="stat-value">{totals.authors}</div>
    </div>
    <div class="stat">
      <div class="stat-title">Tags</div>
      <div class="stat-value">{totals.tags}</div>
    </div>
  </div>

  <!-- Books per tag -->
  <div class="card bg-base-100 shadow mb-6">
    <div class="card-body">
      <div class="flex justify-between items-center">
        <h3 class="card-title text-base">Books per Tag</h3>
        <select class="select select-bordered select-sm" bind:value={kindFilter}>
          <option value="">All</option>
          <option value="owner">Owner</option>
          <option value="genre">Genre</option>
          <option value="custom">Custom</option>
        </select>
      </div>
      <canvas bind:this={tagCanvas}></canvas>
    </div>
  </div>

  <!-- Books per author -->
  <div class="card bg-base-100 shadow mb-6">
    <div class="card-body">
      <h3 class="card-title text-base">Books per Author (top 20)</h3>
      <canvas bind:this={authorCanvas}></canvas>
    </div>
  </div>

  <!-- Growth -->
  <div class="card bg-base-100 shadow mb-6">
    <div class="card-body">
      <div class="flex justify-between items-center">
        <h3 class="card-title text-base">Collection Growth</h3>
        <div class="join">
          <button class="join-item btn btn-sm" class:btn-active={groupBy === 'month'} onclick={() => groupBy = 'month'}>Month</button>
          <button class="join-item btn btn-sm" class:btn-active={groupBy === 'year'} onclick={() => groupBy = 'year'}>Year</button>
          <button class="join-item btn btn-sm" class:btn-active={groupBy === 'day'} onclick={() => groupBy = 'day'}>Day</button>
        </div>
      </div>
      <canvas bind:this={growthCanvas}></canvas>
    </div>
  </div>
</section>
