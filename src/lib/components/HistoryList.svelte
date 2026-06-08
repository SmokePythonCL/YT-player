<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface PlaybackHistoryEntry {
    id: number;
    song_id: string;
    title: string;
    artist: string;
    album: string | null;
    duration_sec: number | null;
    played_at: string;
  }

  let historyEntries = $state<PlaybackHistoryEntry[]>([]);
  let searchQuery = $state("");
  let isLoading = $state(true);

  let filteredEntries = $derived(
    searchQuery.trim()
      ? historyEntries.filter(e =>
          e.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
          e.artist.toLowerCase().includes(searchQuery.toLowerCase()) ||
          (e.album?.toLowerCase().includes(searchQuery.toLowerCase()))
        )
      : historyEntries
  );

  async function loadHistory() {
    try {
      historyEntries = await invoke("get_playback_history", { limit: 100 });
    } catch (e) {
      console.error("Failed to load playback history:", e);
    } finally {
      isLoading = false;
    }
  }

  function fmtDuration(sec: number | null): string {
    if (!sec) return "—";
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  }

  function fmtTime(ts: string): string {
    try {
      const [, time] = ts.split(' ');
      if (!time) return ts;
      const [h, m] = time.split(':');
      return `${h}:${m}`;
    } catch { return ts; }
  }

  onMount(() => {
    loadHistory();
    const iv = setInterval(loadHistory, 8000);
    return () => clearInterval(iv);
  });
</script>

<div class="hc">
  <div class="hdr">
    <h3>Historial de Reproducción</h3>
    <span class="count">{historyEntries.length} canciones</span>
    <button class="btn-r" onclick={loadHistory} aria-label="Refresh" title="Actualizar">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M23 4v6h-6"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
      </svg>
    </button>
  </div>

  <div class="search">
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
    </svg>
    <input type="text" placeholder="Buscar canción, artista..." bind:value={searchQuery} />
    {#if searchQuery}
      <button class="clear" onclick={() => searchQuery = ''}>✕</button>
    {/if}
  </div>

  <div class="list-wrap">
    {#if isLoading}
      {#each Array(5) as _}
        <div class="skeleton"><div class="sk-bar"></div><div class="sk-bar short"></div></div>
      {/each}
    {:else if filteredEntries.length === 0}
      <div class="empty">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><path d="M8 15s1.5 2 4 2 4-2 4-2"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg>
        <p>{searchQuery ? 'Sin resultados para "' + searchQuery + '"' : 'Aún no hay canciones en el historial'}</p>
      </div>
    {:else}
      <ul>
        {#each filteredEntries as entry, i (entry.id)}
          <li class:alt={i % 2 === 1}>
            <span class="idx">{i + 1}</span>
            <div class="info">
              <span class="t">{entry.title}</span>
              <span class="a">{entry.artist}{entry.album ? ` • ${entry.album}` : ''}</span>
            </div>
            <div class="meta">
              <span class="dur">{fmtDuration(entry.duration_sec)}</span>
              <span class="when">{fmtTime(entry.played_at)}</span>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .hc { display: flex; flex-direction: column; height: 100%; color: #e5e5e5; padding: 0 6px; }

  .hdr {
    display: flex; align-items: center; gap: 10px; margin-bottom: 10px;
  }
  .hdr h3 { margin: 0; font-size: 1rem; font-weight: 600; color: #fff; }
  .count { font-size: 0.72rem; color: rgba(255,255,255,0.3); margin-left: auto; }

  .btn-r {
    background: none; border: none; color: rgba(255,255,255,0.4); cursor: pointer;
    padding: 4px; border-radius: 4px; display: flex; transition: all 0.2s;
  }
  .btn-r svg { width: 14px; height: 14px; }
  .btn-r:hover { color: #fff; background: rgba(255,255,255,0.08); }
  .btn-r:active svg { animation: spin 0.5s ease; }
  @keyframes spin { to { transform: rotate(360deg); } }

  .search {
    position: relative; margin-bottom: 10px;
  }
  .search svg {
    position: absolute; left: 10px; top: 50%; transform: translateY(-50%);
    width: 14px; height: 14px; color: rgba(255,255,255,0.3);
  }
  .search input {
    width: 100%; box-sizing: border-box;
    background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08);
    border-radius: 8px; padding: 7px 30px 7px 32px;
    color: #fff; font-size: 0.82rem; outline: none;
    transition: all 0.2s;
  }
  .search input::placeholder { color: rgba(255,255,255,0.25); }
  .search input:focus { background: rgba(255,255,255,0.1); border-color: rgba(255,68,68,0.3); }
  .clear {
    position: absolute; right: 8px; top: 50%; transform: translateY(-50%);
    background: none; border: none; color: rgba(255,255,255,0.3);
    cursor: pointer; font-size: 0.75rem; padding: 2px 4px;
  }
  .clear:hover { color: #fff; }

  .list-wrap {
    flex: 1; overflow-y: auto; max-height: 240px;
    scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.12) transparent;
  }
  .list-wrap::-webkit-scrollbar { width: 5px; }
  .list-wrap::-webkit-scrollbar-track { background: transparent; }
  .list-wrap::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.12); border-radius: 3px; }

  ul { list-style: none; padding: 0; margin: 0; }

  li {
    display: flex; align-items: center; gap: 10px;
    padding: 7px 8px; border-radius: 6px;
    transition: background 0.12s;
  }
  li.alt { background: rgba(255,255,255,0.02); }
  li:hover { background: rgba(255,255,255,0.06); }

  .idx {
    width: 22px; text-align: center; flex-shrink: 0;
    font-size: 0.72rem; color: rgba(255,255,255,0.2); font-variant-numeric: tabular-nums;
  }

  .info { display: flex; flex-direction: column; min-width: 0; flex: 1; gap: 2px; }
  .t {
    font-size: 0.85rem; font-weight: 500; color: #fff;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }
  .a {
    font-size: 0.72rem; color: rgba(255,255,255,0.4);
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  }

  .meta { display: flex; flex-direction: column; align-items: flex-end; flex-shrink: 0; gap: 2px; }
  .dur { font-size: 0.72rem; color: rgba(255,255,255,0.4); font-variant-numeric: tabular-nums; }
  .when { font-size: 0.65rem; color: rgba(255,255,255,0.2); }

  .empty {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    padding: 30px 0; color: rgba(255,255,255,0.25); gap: 10px;
  }
  .empty svg { width: 36px; height: 36px; }
  .empty p { margin: 0; font-size: 0.85rem; }

  .skeleton {
    padding: 10px 8px; display: flex; flex-direction: column; gap: 6px;
  }
  .sk-bar {
    height: 10px; background: rgba(255,255,255,0.06); border-radius: 4px;
    animation: pulse 1.5s ease-in-out infinite;
  }
  .sk-bar.short { width: 60%; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
</style>
