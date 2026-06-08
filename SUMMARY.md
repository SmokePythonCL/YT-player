# Resumen del Proyecto - YouTube Music Desktop Client

Este documento resume las librerías, utilidades y la arquitectura utilizada para crear el cliente de escritorio para YouTube Music utilizando **Tauri 2.0** y **Svelte 5**.

---

## 🛠️ Tecnologías y Librerías Utilizadas

### Backend (Rust)
El núcleo de la aplicación está escrito en Rust para maximizar la eficiencia y el control nativo del sistema.

*   **Tauri v2.0 (`tauri`):** Framework base para la creación de ventanas de escritorio nativas e intercomunicación segura con la web.
    *   *Features habilitadas:* `unstable` (para soportar layouts multi-webview integrados).
*   **Tauri Plugin Global Shortcut (`tauri-plugin-global-shortcut`):** Captura atajos de teclado globales a nivel del sistema operativo.
*   **SQLx (`sqlx` con drivers `sqlite` y `runtime-tokio-native-tls`):** Ejecución asíncrona de consultas y transacciones preparadas para SQLite.
*   **Tokio (`tokio`):** Runtime asíncrono en Rust utilizado para manejar tareas en segundo plano.
*   **Serde (`serde` y `serde_json`):** Serialización y deserialización de estructuras de datos (intercambio JSON Rust ↔ JS).
*   **Chrono (`chrono`):** Gestión de fechas y marcas de tiempo en el historial de reproducción.
*   **Tracing (`tracing` y `tracing-subscriber`):** Sistema de registro y diagnóstico (*logging*) estructurado para depuración.

### Base de Datos (SQLite Local)
Una base de datos local y autónoma ubicada en el directorio de datos del usuario (`~/.local/share/com.yt-music.desktop/yt-music.db`).
*   **Tabla `playback_history`:** Historial detallado con marcas de tiempo e información del tema.
*   **Tabla `recently_played`:** Lista única y rápida de las canciones reproducidas recientemente.
*   **Tabla `user_preferences`:** Almacenamiento clave-valor persistente para volumen y estado visual.
*   **Tabla `playlists_cache`:** Caché local para metadatos de listas de reproducción.

### Frontend (Svelte 5 + TypeScript)
Interfaz de control premium, diseñada con alta fidelidad y efectos visuales modernos.

*   **Svelte v5:** Uso de *Runes* (`$state`, `$derived`, `$effect`) para una reactividad ultra rápida y ligera.
*   **SvelteKit (Static Adapter):** Configurado en modo Single Page App (SPA) sin servidor de Node.js en producción.
*   **Vite:** Servidor de desarrollo y bundler optimizado para el empaquetado final de estáticos en la carpeta `build`.
*   **Vanilla CSS:** Estilos puros para diseño translúcido (*glassmorphism*), animaciones de pulso en reproducción y transiciones en controles.

---

## 📐 Arquitectura de la Aplicación

El diseño de la aplicación evita el uso de *iframes* (los cuales bloquea Google por políticas de seguridad) implementando **Multi-Webview** nativo:

```
┌────────────────────────────────────────────────────────┐
│  [🎵]  Controlador Local (Svelte 5) - app_control       │  ← Altura: 80px
├────────────────────────────────────────────────────────┤
│                                                        │
│                                                        │
│    Web Oficial de YouTube Music - ytmusic              │  ← Altura: Restante
│    (https://music.youtube.com)                         │
│                                                        │
└────────────────────────────────────────────────────────┘
```

1.  **Webview `app_control` (Local):** Renderiza el panel Svelte 5 a nivel local.
2.  **Webview `ytmusic` (Remoto):** Carga el reproductor web oficial de YouTube.
3.  **Inyección (`inject.js`):** Un script JS ligero se inyecta en la web oficial y monitoriza la etiqueta `<video>` y el API `mediaSession` para notificar al backend en Rust mediante IPC cada cambio de pista, volumen o progreso.
4.  **Backend Rust:** Centraliza los eventos, registra las canciones en la base de datos SQLite y ejecuta acciones de control en la ventana web (Play, Pause, Saltos de canción y Volumen) mediante evaluaciones directas de JavaScript (`.eval()`).

---

## ⚙️ Configuración del Entorno de Desarrollo

Debido a la ausencia de Node.js global en el sistema, la inicialización del proyecto se estructuró así:
1.  Descarga de Node.js portátil v20.12.2 en la carpeta oculta `.node-env/`.
2.  Instalación local de dependencias mediante `npm install` prependeando la ruta del binario local al `PATH`.
3.  Compilación saltándose el bundler externo (`linuxdeploy`) mediante la modificación de `"targets": []` en `tauri.conf.json`.

---

## 🚀 Comandos Útiles

### Modo Desarrollo
Inicia la app en caliente (hot-reload en frontend y backend):
```bash
PATH="/home/kran/Documentos/Proyectos/YT-player/.node-env/node/bin:$PATH" npm run tauri dev
```

### Compilación de Producción
Genera el ejecutable optimizado en `src-tauri/target/release/tauri-app`:
```bash
PATH="/home/kran/Documentos/Proyectos/YT-player/.node-env/node/bin:$PATH" npm run tauri build
```
