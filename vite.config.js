import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from '@tailwindcss/vite';

export default {
  plugins: [
    sveltekit(),
    tailwindcss(),
  ],
  css: {
    postcss: false,
  },
  server: {
    port: 1420,
    strictPort: true,
  },
  ssr: {
    noExternal: true
  },
  experimental: {
    forceSsrPreload: false
  }
};