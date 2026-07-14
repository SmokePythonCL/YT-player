<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import HistoryList from "../lib/components/HistoryList.svelte";

  // Navigation State
  let activeTab = $state<'home' | 'search' | 'history'>('home');
  let isLoginMode = $state(false);

  // Playback State
  let isPlaying = $state(false);
  let songId = $state("");
  let title = $state("YouTube Music");
  let artist = $state("Esperando reproducción...");
  let album = $state("");
  let durationSec = $state(0);
  let currentTimeSec = $state(0);
  let volume = $state(100);
  let isScrubbing = $state(false);
  let hoverProgress = $state(0);

  // Search Tab State
  let searchQuery = $state("");
  let searchResults = $state<any[]>([]);
  let isLoadingSearch = $state(false);

  // Home Tab State
  let homeFeed = $state<any[]>([]);
  let isLoadingHome = $state(false);

  // Derived Values
  let progressPercent = $derived(durationSec > 0 ? (currentTimeSec / durationSec) * 100 : 0);
  let volumeGradient = $derived(
    `linear-gradient(to right, #ff3344 0%, #ff3344 ${volume}%, rgba(255,255,255,0.1) ${volume}%, rgba(255,255,255,0.1) 100%)`
  );
  let currentThumbnail = $derived(songId ? `https://i.ytimg.com/vi/${songId}/mqdefault.jpg` : "");

  function formatTime(sec: number): string {
    if (!sec || isNaN(sec) || sec < 0) return "0:00";
    const m = Math.floor(sec / 60);
    const s = Math.floor(sec % 60);
    return `${m}:${s < 10 ? '0' : ''}${s}`;
  }

  // Playback Control Invokes
  async function togglePlay() {
    try { await invoke("play_pause"); } catch (e) { console.error(e); }
  }

  async function nextTrack() {
    try { await invoke("next_track"); } catch (e) { console.error(e); }
  }

  async function prevTrack() {
    try { await invoke("previous_track"); } catch (e) { console.error(e); }
  }

  async function handleVolumeChange(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    volume = val;
    try { await invoke("set_volume", { level: val }); } catch (e) { console.error(e); }
  }

  function handleProgressHover(e: MouseEvent) {
    const bar = e.currentTarget as HTMLDivElement;
    const rect = bar.getBoundingClientRect();
    hoverProgress = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  }

  async function handleScrub(e: MouseEvent) {
    if (durationSec <= 0) return;
    const bar = e.currentTarget as HTMLDivElement;
    const rect = bar.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const target = pct * durationSec;
    currentTimeSec = Math.round(target);
    try { await invoke("set_playback_progress", { seconds: target }); } catch (e) { console.error(e); }
  }

  // Navigation & YTM Bridge Invokes
  async function triggerSearch() {
    if (!searchQuery.trim()) return;
    isLoadingSearch = true;
    try {
      await invoke("search_ytmusic", { query: searchQuery });
    } catch (e) {
      console.error("Failed to invoke search:", e);
      isLoadingSearch = false;
    }
  }

  async function loadHomeFeed() {
    isLoadingHome = true;
    try {
      await invoke("get_ytmusic_home");
    } catch (e) {
      console.error("Failed to invoke get home feed:", e);
      isLoadingHome = false;
    }
  }

  async function playTrack(id: string) {
    try {
      await invoke("play_song", { songId: id });
    } catch (e) {
      console.error("Failed to play track:", e);
    }
  }

  async function toggleLoginMode() {
    isLoginMode = !isLoginMode;
    try {
      await invoke("toggle_ytmusic_visibility", { visible: isLoginMode });
      if (!isLoginMode) {
        // Al salir del modo login, recargar el home feed para ver el feed logueado
        loadHomeFeed();
      }
    } catch (e) {
      console.error(e);
    }
  }

  onMount(() => {
    // 1. Escuchar actualizaciones de reproducción en tiempo real
    const unlistenPlayback = listen("playback-state-changed", (event: any) => {
      const s = event.payload;
      isPlaying = s.is_playing;
      if (s.song_id && s.song_id !== songId) songId = s.song_id;
      if (s.title) title = s.title;
      if (s.artist) artist = s.artist;
      album = s.album || "";
      durationSec = s.duration_sec || 0;
      if (!isScrubbing) currentTimeSec = s.current_time_sec || 0;
      if (s.volume != null) volume = s.volume;
    });

    // 2. Escuchar resultados de búsqueda desde InnerTube
    const unlistenSearch = listen("search-results-received", (event: any) => {
      searchResults = event.payload || [];
      isLoadingSearch = false;
    });

    // 3. Escuchar feed de inicio desde InnerTube
    const unlistenHome = listen("home-feed-received", (event: any) => {
      homeFeed = event.payload || [];
      isLoadingHome = false;
    });

    // Cargar feed de inicio inicial
    setTimeout(loadHomeFeed, 1500);

    return () => {
      unlistenPlayback.then(fn => fn());
      unlistenSearch.then(fn => fn());
      unlistenHome.then(fn => fn());
    };
  });
</script>

{#if isLoginMode}
  <!-- Vista reducida de app_control en Modo Login -->
  <div class="login-header">
    <div class="login-info">
      <div class="pulse-indicator"></div>
      <span>Sesión activa de YouTube Music (Interactúa abajo para Iniciar Sesión en tu cuenta Google)</span>
    </div>
    <button onclick={toggleLoginMode} class="btn-primary">Finalizar / Volver a la App</button>
  </div>
{:else}
  <!-- Interfaz de Navegación Nativa a Pantalla Completa -->
  <div class="app-container">
    
    <!-- Sidebar Lateral -->
    <aside class="sidebar">
      <div class="logo">
        <svg viewBox="0 0 24 24" fill="currentColor" class="logo-icon">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z"/>
        </svg>
        <span class="logo-text">YT Player</span>
      </div>

      <nav class="nav-menu">
        <button class="nav-item" class:active={activeTab === 'home'} onclick={() => activeTab = 'home'}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
          <span>Inicio</span>
        </button>
        <button class="nav-item" class:active={activeTab === 'search'} onclick={() => activeTab = 'search'}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
          <span>Buscar</span>
        </button>
        <button class="nav-item" class:active={activeTab === 'history'} onclick={() => activeTab = 'history'}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 8v4l3 3m6-3a9 9 0 1 1-18 0 9 9 0 0 1 18 0z"/></svg>
          <span>Historial</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <button onclick={toggleLoginMode} class="btn-login-toggle" title="Conecta tu cuenta de Google para obtener playlists y recomendaciones personalizadas">
          <svg viewBox="0 0 24 24" fill="currentColor" class="btn-icon-svg"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/></svg>
          <span>Conectar Cuenta</span>
        </button>
      </div>
    </aside>

    <!-- Contenido Principal -->
    <main class="main-content">
      
      <!-- Vista 1: Inicio (Recomendaciones del feed de YTM) -->
      {#if activeTab === 'home'}
        <div class="view-container">
          <div class="header-section">
            <h1>Descubrir Música</h1>
            <button onclick={loadHomeFeed} class="btn-refresh" title="Actualizar inicio" aria-label="Refresh">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M23 4v6h-6M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/></svg>
            </button>
          </div>

          {#if isLoadingHome}
            <div class="loading-wrapper">
              <div class="spinner"></div>
              <p>Cargando recomendaciones personalizadas...</p>
            </div>
          {:else if homeFeed.length === 0}
            <div class="empty-state">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><path d="M9 18c1.5-2 4.5-2 6 0"/><line x1="9" y1="9" x2="9.01" y2="9"/><line x1="15" y1="9" x2="15.01" y2="9"/></svg>
              <h3>No se pudieron cargar recomendaciones</h3>
              <p>Haz clic en 'Conectar Cuenta' en la barra lateral para sincronizar tu sesión de Google o actualiza.</p>
              <button onclick={loadHomeFeed} class="btn-primary" style="margin-top: 12px;">Reintentar</button>
            </div>
          {:else}
            <div class="feed-scrollable">
              {#each homeFeed as section (section.title)}
                <section class="feed-section">
                  <h2>{section.title}</h2>
                  <div class="cards-grid">
                    {#each section.items as item (item.song_id || item.title)}
                      <div class="music-card" onclick={() => item.song_id && playTrack(item.song_id)} role="button" tabindex="0" onkeypress={(e) => e.key === 'Enter' && item.song_id && playTrack(item.song_id)}>
                        <div class="card-art-wrap">
                          {#if item.thumbnail}
                            <img src={item.thumbnail} alt={item.title} class="card-art" loading="lazy" />
                          {:else}
                            <div class="card-art-fallback">
                              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
                            </div>
                          {/if}
                          <div class="card-play-overlay">
                            <button class="btn-play-card" aria-label="Play">
                              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
                            </button>
                          </div>
                        </div>
                        <div class="card-info">
                          <span class="card-title" title={item.title}>{item.title}</span>
                          <span class="card-artist" title={item.artist}>{item.artist}</span>
                        </div>
                      </div>
                    {/each}
                  </div>
                </section>
              {/each}
            </div>
          {/if}
        </div>

      <!-- Vista 2: Buscar -->
      {:else if activeTab === 'search'}
        <div class="view-container">
          <div class="search-bar-wrap">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="search-icon-svg"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
            <input 
              type="text" 
              placeholder="¿Qué quieres escuchar hoy? (Canción, artista, género...)" 
              bind:value={searchQuery}
              onkeydown={(e) => e.key === 'Enter' && triggerSearch()}
              class="search-input-field" 
            />
            {#if searchQuery}
              <button class="btn-clear-search" onclick={() => searchQuery = ""}>✕</button>
            {/if}
            <button onclick={triggerSearch} class="btn-search-trigger">Buscar</button>
          </div>

          {#if isLoadingSearch}
            <div class="loading-wrapper">
              <div class="spinner"></div>
              <p>Buscando en YouTube Music...</p>
            </div>
          {:else if searchResults.length === 0}
            <div class="empty-state">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
              <h3>Encuentra tu música favorita</h3>
              <p>Escribe una canción, álbum o artista arriba y presiona Enter.</p>
            </div>
          {:else}
            <div class="search-results-list">
              <h2>Resultados</h2>
              <div class="results-grid">
                {#each searchResults as song, idx (song.song_id)}
                  <div class="result-row" onclick={() => playTrack(song.song_id)} role="button" tabindex="0" onkeypress={(e) => e.key === 'Enter' && playTrack(song.song_id)}>
                    <span class="result-idx">{idx + 1}</span>
                    <img src={song.thumbnail || `https://i.ytimg.com/vi/${song.song_id}/mqdefault.jpg`} alt={song.title} class="result-art" />
                    <div class="result-info">
                      <span class="result-title" title={song.title}>{song.title}</span>
                      <span class="result-artist" title={song.artist}>{song.artist}{song.album ? ` • ${song.album}` : ''}</span>
                    </div>
                    <span class="result-duration">{formatTime(song.duration_sec)}</span>
                    <button class="btn-play-row" aria-label="Play">
                      <svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
                    </button>
                  </div>
                {/each}
              </div>
            </div>
          {/if}
        </div>

      <!-- Vista 3: Historial -->
      {:else if activeTab === 'history'}
        <div class="view-container">
          <HistoryList />
        </div>
      {/if}
    </main>

    <!-- Barra de Reproducción Inferior -->
    <footer class="player-bar-bottom">
      
      <!-- Información del tema actual -->
      <div class="player-song-info">
        {#if songId}
          <img src={currentThumbnail} alt="Carátula" class="player-art" class:playing={isPlaying} />
        {:else}
          <div class="player-art-fallback">
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
          </div>
        {/if}
        <div class="player-text-info">
          <div class="player-title-wrap">
            <span class="player-title" class:scroll={title.length > 25} title={title}>{title}</span>
          </div>
          <span class="player-artist" title={artist}>{artist}{album ? ` • ${album}` : ''}</span>
        </div>
      </div>

      <!-- Controles centrales -->
      <div class="player-controls-center">
        <div class="control-buttons">
          <button onclick={prevTrack} class="btn-ctrl" title="Anterior (Ctrl+←)" aria-label="Previous">
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
          </button>
          <button onclick={togglePlay} class="btn-play-main" class:playing={isPlaying} title={isPlaying ? "Pausar (Ctrl+Space)" : "Reproducir (Ctrl+Space)"} aria-label="Play/Pause">
            {#if isPlaying}
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
            {:else}
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            {/if}
          </button>
          <button onclick={nextTrack} class="btn-ctrl" title="Siguiente (Ctrl+→)" aria-label="Next">
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 18l8.5-6L6 6zm9-12v12h2V6z"/></svg>
          </button>
        </div>

        <div class="progress-bar-row">
          <span class="time-label">{formatTime(currentTimeSec)}</span>
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div class="progress-slider-track" onclick={handleScrub} onmousemove={handleProgressHover}>
            <div class="progress-slider-bg">
              <div class="progress-slider-hover" style="width: {hoverProgress * 100}%"></div>
              <div class="progress-slider-fill" style="width: {progressPercent}%">
                <div class="progress-slider-dot"></div>
              </div>
            </div>
          </div>
          <span class="time-label">{formatTime(durationSec)}</span>
        </div>
      </div>

      <!-- Controles de volumen y extras (derecha) -->
      <div class="player-controls-right">
        <div class="volume-control-wrap">
          <svg class="vol-icon-svg" viewBox="0 0 24 24" fill="currentColor">
            {#if volume === 0}
              <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/>
            {:else if volume < 50}
              <path d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z"/>
            {:else}
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
            {/if}
          </svg>
          <input 
            type="range" 
            min="0" 
            max="100" 
            value={volume} 
            oninput={handleVolumeChange}
            class="volume-slider-field" 
            style="background: {volumeGradient}" 
            title="Volumen: {Math.round(volume)}% (Ctrl+↑↓)"
            aria-label="Volume"
          />
        </div>
      </div>
    </footer>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0; padding: 0;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #09090b !important;
    color: #e4e4e7;
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
  }

  /* ── Modo Login Header ── */
  .login-header {
    display: flex; justify-content: space-between; align-items: center;
    height: 60px; padding: 0 24px;
    background: #18181b; border-bottom: 1px solid #27272a;
    box-sizing: border-box;
  }
  .login-info {
    display: flex; align-items: center; gap: 10px;
    font-size: 0.85rem; color: #a1a1aa; font-weight: 500;
  }
  .pulse-indicator {
    width: 8px; height: 8px; border-radius: 50%;
    background: #10b981;
    box-shadow: 0 0 8px #10b981;
    animation: pulse 1.8s infinite;
  }
  @keyframes pulse {
    0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(16, 185, 129, 0.7); }
    70% { transform: scale(1); box-shadow: 0 0 0 6px rgba(16, 185, 129, 0); }
    100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(16, 185, 129, 0); }
  }

  /* ── Contenedor General de la App ── */
  .app-container {
    display: flex;
    height: 100vh; width: 100vw;
    background: radial-gradient(circle at top right, rgba(255, 51, 68, 0.05), transparent 60%), #09090b;
    overflow: hidden;
  }

  /* ── Sidebar Izquierdo ── */
  .sidebar {
    width: 240px; height: calc(100vh - 90px);
    background: rgba(15, 15, 18, 0.8);
    border-right: 1px solid rgba(255, 255, 255, 0.04);
    display: flex; flex-direction: column;
    padding: 24px 16px; box-sizing: border-box;
    flex-shrink: 0;
  }
  .logo {
    display: flex; align-items: center; gap: 10px; margin-bottom: 36px; padding-left: 8px;
  }
  .logo-icon {
    width: 28px; height: 28px; color: #ff3344;
  }
  .logo-text {
    font-size: 1.15rem; font-weight: 700; color: #fff; letter-spacing: -0.5px;
  }
  .nav-menu {
    display: flex; flex-direction: column; gap: 6px; flex: 1;
  }
  .nav-item {
    background: none; border: none;
    display: flex; align-items: center; gap: 14px;
    padding: 10px 14px; border-radius: 8px;
    color: #a1a1aa; font-size: 0.9rem; font-weight: 500;
    cursor: pointer; text-align: left;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  }
  .nav-item svg {
    width: 20px; height: 20px; transition: stroke 0.2s;
  }
  .nav-item:hover {
    color: #fff; background: rgba(255, 255, 255, 0.04);
  }
  .nav-item.active {
    color: #fff; background: rgba(255, 51, 68, 0.12);
  }
  .nav-item.active svg {
    stroke: #ff3344;
  }
  .sidebar-footer {
    padding-top: 16px; border-top: 1px solid rgba(255, 255, 255, 0.04);
  }
  .btn-login-toggle {
    width: 100%; display: flex; align-items: center; justify-content: center; gap: 10px;
    background: rgba(255, 255, 255, 0.04); border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 10px; border-radius: 8px;
    color: #e4e4e7; font-size: 0.8rem; font-weight: 600; cursor: pointer;
    transition: all 0.2s;
  }
  .btn-login-toggle:hover {
    background: rgba(255, 255, 255, 0.08); color: #fff; border-color: rgba(255, 51, 68, 0.3);
  }
  .btn-icon-svg {
    width: 16px; height: 16px; color: #ff3344;
  }

  /* ── Contenido Principal ── */
  .main-content {
    flex: 1; height: calc(100vh - 90px);
    overflow: hidden;
    position: relative;
  }
  .view-container {
    height: 100%; padding: 32px 40px; box-sizing: border-box;
    display: flex; flex-direction: column; overflow: hidden;
  }
  .header-section {
    display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;
  }
  .header-section h1 {
    font-size: 1.8rem; font-weight: 700; color: #fff; margin: 0; letter-spacing: -0.5px;
  }
  .btn-refresh {
    background: none; border: none; color: #a1a1aa; cursor: pointer;
    padding: 8px; border-radius: 50%; display: flex; transition: all 0.2s;
  }
  .btn-refresh svg { width: 18px; height: 18px; }
  .btn-refresh:hover { color: #fff; background: rgba(255, 255, 255, 0.06); }

  /* ── Feed Scrollable (Inicio) ── */
  .feed-scrollable {
    flex: 1; overflow-y: auto; padding-right: 8px;
    scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.1) transparent;
  }
  .feed-scrollable::-webkit-scrollbar { width: 6px; }
  .feed-scrollable::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 3px; }
  .feed-section {
    margin-bottom: 32px;
  }
  .feed-section h2 {
    font-size: 1.25rem; font-weight: 600; color: #fff; margin: 0 0 16px 0;
  }
  .cards-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 20px;
  }
  .music-card {
    background: rgba(255, 255, 255, 0.01); border: 1px solid rgba(255, 255, 255, 0.02);
    border-radius: 12px; padding: 12px; cursor: pointer;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }
  .music-card:hover {
    background: rgba(255, 255, 255, 0.04); border-color: rgba(255, 255, 255, 0.06);
    transform: translateY(-4px);
  }
  .card-art-wrap {
    width: 100%; aspect-ratio: 1; border-radius: 8px; overflow: hidden;
    position: relative; margin-bottom: 12px; background: rgba(255,255,255,0.03);
  }
  .card-art {
    width: 100%; height: 100%; object-fit: cover;
    transition: transform 0.4s;
  }
  .music-card:hover .card-art {
    transform: scale(1.05);
  }
  .card-art-fallback {
    width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
    color: #ff3344;
  }
  .card-art-fallback svg { width: 40px; height: 40px; }
  
  .card-play-overlay {
    position: absolute; top: 0; left: 0; width: 100%; height: 100%;
    background: rgba(0,0,0,0.4); opacity: 0; display: flex; align-items: center; justify-content: center;
    transition: opacity 0.2s;
  }
  .music-card:hover .card-play-overlay {
    opacity: 1;
  }
  .btn-play-card {
    background: #ff3344; color: #fff; border: none; width: 44px; height: 44px;
    border-radius: 50%; display: flex; align-items: center; justify-content: center;
    cursor: pointer; box-shadow: 0 4px 12px rgba(255, 51, 68, 0.4);
    transition: transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  }
  .btn-play-card svg { width: 20px; height: 20px; transform: translateX(1px); }
  .btn-play-card:hover {
    transform: scale(1.1); box-shadow: 0 6px 16px rgba(255, 51, 68, 0.6);
  }

  .card-info {
    display: flex; flex-direction: column; gap: 4px;
  }
  .card-title {
    font-size: 0.85rem; font-weight: 600; color: #fff;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .card-artist {
    font-size: 0.75rem; color: #a1a1aa;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  /* ── Barra de Búsqueda ── */
  .search-bar-wrap {
    position: relative; display: flex; align-items: center; gap: 12px; margin-bottom: 24px;
  }
  .search-icon-svg {
    position: absolute; left: 16px; width: 18px; height: 18px; color: #a1a1aa;
  }
  .search-input-field {
    flex: 1; background: rgba(255, 255, 255, 0.04); border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px; padding: 12px 48px 12px 44px;
    color: #fff; font-size: 0.9rem; outline: none;
    transition: all 0.2s;
  }
  .search-input-field:focus {
    background: rgba(255, 255, 255, 0.07); border-color: rgba(255, 51, 68, 0.4);
    box-shadow: 0 0 0 1px rgba(255, 51, 68, 0.2);
  }
  .btn-clear-search {
    position: absolute; right: 100px; background: none; border: none; color: #71717a;
    font-size: 0.8rem; cursor: pointer; padding: 6px;
  }
  .btn-clear-search:hover { color: #fff; }
  .btn-search-trigger {
    background: #ff3344; color: #fff; border: none; padding: 12px 20px;
    border-radius: 10px; font-weight: 600; font-size: 0.88rem; cursor: pointer;
    box-shadow: 0 2px 8px rgba(255, 51, 68, 0.25);
    transition: all 0.2s;
  }
  .btn-search-trigger:hover {
    background: #ff1a2f; transform: translateY(-1px); box-shadow: 0 4px 12px rgba(255, 51, 68, 0.4);
  }
  .btn-search-trigger:active { transform: translateY(0); }

  /* ── Lista de Resultados de Búsqueda ── */
  .search-results-list {
    flex: 1; display: flex; flex-direction: column; overflow: hidden;
  }
  .search-results-list h2 {
    font-size: 1.15rem; font-weight: 600; color: #fff; margin: 0 0 14px 0;
  }
  .results-grid {
    flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 4px;
    padding-right: 8px; scrollbar-width: thin; scrollbar-color: rgba(255,255,255,0.1) transparent;
  }
  .results-grid::-webkit-scrollbar { width: 6px; }
  .results-grid::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 3px; }
  
  .result-row {
    display: flex; align-items: center; gap: 16px;
    padding: 8px 16px; border-radius: 8px; cursor: pointer;
    transition: all 0.15s; position: relative;
  }
  .result-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .result-idx {
    width: 20px; font-size: 0.8rem; color: #71717a; text-align: center;
  }
  .result-art {
    width: 44px; height: 44px; object-fit: cover; border-radius: 6px; background: rgba(255,255,255,0.03);
  }
  .result-info {
    flex: 1; display: flex; flex-direction: column; min-width: 0; gap: 3px;
  }
  .result-title {
    font-size: 0.9rem; font-weight: 600; color: #fff;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .result-artist {
    font-size: 0.78rem; color: #a1a1aa;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }
  .result-duration {
    font-size: 0.8rem; color: #71717a; font-variant-numeric: tabular-nums;
  }
  .btn-play-row {
    background: none; border: none; color: #ff3344; display: none; cursor: pointer;
    padding: 6px;
  }
  .result-row:hover .btn-play-row { display: block; }
  .result-row:hover .result-duration { display: none; }
  .btn-play-row svg { width: 20px; height: 20px; }

  /* ── Elementos Comunes: Carga y Vacíos ── */
  .loading-wrapper {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 16px; color: #a1a1aa; font-size: 0.9rem;
  }
  .spinner {
    width: 36px; height: 36px; border: 3px solid rgba(255, 51, 68, 0.1);
    border-top-color: #ff3344; border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  
  .empty-state {
    flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center;
    color: #71717a; gap: 8px; text-align: center;
  }
  .empty-state svg { width: 48px; height: 48px; color: rgba(255,255,255,0.15); margin-bottom: 8px; }
  .empty-state h3 { font-size: 1.1rem; color: #fff; margin: 0; font-weight: 600; }
  .empty-state p { font-size: 0.85rem; margin: 0; max-width: 320px; line-height: 1.4; }

  .btn-primary {
    background: #ff3344; color: #fff; border: none; padding: 10px 18px;
    border-radius: 8px; font-weight: 600; font-size: 0.85rem; cursor: pointer; transition: all 0.2s;
  }
  .btn-primary:hover { background: #ff1a2f; }

  /* ── Barra de Reproducción Inferior ── */
  .player-bar-bottom {
    position: absolute; bottom: 0; left: 0; right: 0;
    height: 90px; padding: 0 24px;
    background: linear-gradient(180deg, rgba(10, 10, 12, 0.94) 0%, rgba(5, 5, 6, 0.98) 100%);
    backdrop-filter: blur(24px) saturate(180%);
    -webkit-backdrop-filter: blur(24px) saturate(180%);
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    display: flex; justify-content: space-between; align-items: center;
    box-sizing: border-box; z-index: 100;
  }

  /* 1. Player Info */
  .player-song-info {
    display: flex; align-items: center; width: 28%; min-width: 200px;
  }
  .player-art {
    width: 54px; height: 54px; border-radius: 8px; object-fit: cover;
    border: 1px solid rgba(255, 255, 255, 0.06);
    margin-right: 14px; flex-shrink: 0;
    box-shadow: 0 4px 10px rgba(0,0,0,0.4);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
  .player-art.playing {
    border-color: rgba(255, 51, 68, 0.25);
    box-shadow: 0 0 14px rgba(255, 51, 68, 0.15);
  }
  .player-art-fallback {
    width: 54px; height: 54px; border-radius: 8px; display: flex; align-items: center;
    justify-content: center; background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.04);
    margin-right: 14px; flex-shrink: 0; color: #ff3344;
  }
  .player-art-fallback svg { width: 24px; height: 24px; }

  .player-text-info {
    display: flex; flex-direction: column; min-width: 0; gap: 2px;
  }
  .player-title-wrap { overflow: hidden; }
  .player-title {
    font-size: 0.88rem; font-weight: 600; color: #fff;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    display: inline-block;
  }
  .player-title.scroll {
    animation: player-marquee 12s linear infinite;
  }
  @keyframes player-marquee {
    0%, 20% { transform: translateX(0); }
    80%, 100% { transform: translateX(-50%); }
  }
  .player-artist {
    font-size: 0.75rem; color: #a1a1aa;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  /* 2. Player Controls Center */
  .player-controls-center {
    display: flex; flex-direction: column; align-items: center; width: 44%;
  }
  .control-buttons {
    display: flex; align-items: center; gap: 16px; margin-bottom: 6px;
  }
  
  .btn-ctrl {
    background: none; border: none; color: #a1a1aa; cursor: pointer;
    padding: 6px; border-radius: 50%; display: flex; align-items: center;
    transition: all 0.15s ease;
  }
  .btn-ctrl svg { width: 20px; height: 20px; }
  .btn-ctrl:hover { color: #fff; background: rgba(255, 255, 255, 0.06); }
  .btn-ctrl:active { transform: scale(0.9); }

  .btn-play-main {
    background: #fff; color: #000; border: none; cursor: pointer;
    padding: 10px; border-radius: 50%; display: flex; align-items: center; justify-content: center;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    box-shadow: 0 4px 12px rgba(255, 255, 255, 0.1);
  }
  .btn-play-main svg { width: 22px; height: 22px; }
  .btn-play-main:hover { transform: scale(1.08); box-shadow: 0 4px 16px rgba(255, 255, 255, 0.2); }
  .btn-play-main:active { transform: scale(0.95); }
  .btn-play-main.playing { background: #ff3344; color: #fff; box-shadow: 0 4px 14px rgba(255, 51, 68, 0.35); }
  .btn-play-main.playing:hover { box-shadow: 0 4px 20px rgba(255, 51, 68, 0.5); }

  .progress-bar-row {
    width: 100%; display: flex; align-items: center; gap: 12px;
  }
  .time-label {
    font-size: 0.72rem; color: #71717a; min-width: 32px; text-align: center;
    font-variant-numeric: tabular-nums;
  }
  
  .progress-slider-track {
    flex: 1; height: 8px; cursor: pointer; display: flex; align-items: center;
  }
  .progress-slider-bg {
    width: 100%; height: 3px; background: rgba(255, 255, 255, 0.08);
    border-radius: 2px; position: relative; transition: height 0.15s;
  }
  .progress-slider-track:hover .progress-slider-bg {
    height: 5px;
  }
  .progress-slider-hover {
    position: absolute; height: 100%; background: rgba(255, 255, 255, 0.04);
    pointer-events: none; border-radius: 2px;
  }
  .progress-slider-fill {
    position: absolute; height: 100%; background: #ff3344;
    border-radius: 2px; display: flex; align-items: center; justify-content: flex-end;
  }
  .progress-slider-dot {
    width: 0px; height: 0px; border-radius: 50%; background: #ff3344;
    transform: translateX(50%); transition: all 0.15s;
  }
  .progress-slider-track:hover .progress-slider-dot {
    width: 10px; height: 10px; box-shadow: 0 0 6px rgba(255, 51, 68, 0.6);
  }

  /* 3. Player Controls Right */
  .player-controls-right {
    display: flex; align-items: center; justify-content: flex-end; width: 28%; min-width: 200px;
  }
  .volume-control-wrap {
    display: flex; align-items: center; gap: 8px;
  }
  .vol-icon-svg {
    width: 18px; height: 18px; color: #a1a1aa; flex-shrink: 0;
  }
  .volume-slider-field {
    width: 100px; height: 4px;
    -webkit-appearance: none; appearance: none;
    border-radius: 2px; outline: none; cursor: pointer;
    transition: background 0.1s;
  }
  .volume-slider-field::-webkit-slider-thumb {
    -webkit-appearance: none; appearance: none;
    width: 12px; height: 12px; border-radius: 50%;
    background: #fff; border: none;
    box-shadow: 0 1px 4px rgba(0,0,0,0.3);
    transition: transform 0.15s;
  }
  .volume-slider-field:hover::-webkit-slider-thumb {
    transform: scale(1.3);
  }
</style>
