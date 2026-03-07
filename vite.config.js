import { sveltekit } from '@sveltejs/kit/vite';

export default {
  plugins: [sveltekit()],
  server: {
    port: 1420,
    strictPort: true,
  },
  ssr: {
    noExternal: ['@tauri-apps/api', 'svelte-sonner', 'tauri-plugin-liquid-glass-api'],
  },
};
