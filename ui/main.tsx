import {createRoot} from 'react-dom/client';
import App from './App.tsx';
import './index.css';

const container = document.getElementById('root')!;
const root = createRoot(container);
root.render(<App />);

/**
 * Tell the Tauri shell the UI has mounted so it can reveal the window.
 *
 * The window is created hidden to avoid a white flash before React paints. The
 * shell also has a timeout fallback, so a failure here degrades to "the window
 * shows anyway" rather than "the app never appears".
 */
const signalReady = async (): Promise<void> => {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('frontend_ready');
  } catch {
    // The shell's fallback timer will show the window; nothing to report.
  }
};

// Two frames later, so the first paint has happened before the window appears.
requestAnimationFrame(() => requestAnimationFrame(() => void signalReady()));

// Stop the browser from opening dropped files when the user misses a drop zone.
// Tauri's own drag handler needs the default prevented to report the paths.
window.addEventListener('dragover', (event) => event.preventDefault());
window.addEventListener('drop', (event) => event.preventDefault());
