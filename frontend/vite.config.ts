import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		proxy: {
			// Forward API calls to Rust backend in dev
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
