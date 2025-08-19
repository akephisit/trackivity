import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, loadEnv } from 'vite';

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '');

  // Use PUBLIC_API_URL as proxy target if provided, otherwise fallback to localhost
  const rawTarget = (env.PUBLIC_API_URL || '').trim().replace(/\/$/, '');
  const target = rawTarget || 'http://localhost:3000';

  return {
    plugins: [tailwindcss(), sveltekit()],
    server: {
      proxy: {
        // Forward API calls to Rust backend in dev; target can be external
        '/api': {
          target,
          changeOrigin: true,
          secure: false,
          ws: false
        },
        '/health': {
          target,
          changeOrigin: true,
          secure: false
        }
      }
    }
  };
});
