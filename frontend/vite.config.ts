import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			// Explicit SSE proxy with long timeouts
			'/api/sse': {
				target: 'http://localhost:3000',
				changeOrigin: true,
				secure: false,
				timeout: 0, // do not timeout SSE
				proxyTimeout: 0,
				headers: {
					Connection: 'keep-alive'
				}
			},
			// Forward API calls (including SSE) to Rust backend in dev
			'/api': {
				target: 'http://localhost:3000',
				changeOrigin: true,
				secure: false,
				ws: false
			},
			'/health': {
				target: 'http://localhost:3000',
				changeOrigin: true,
				secure: false
			}
		}
	}
});
