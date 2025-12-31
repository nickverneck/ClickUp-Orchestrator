import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	server: {
		port: 5180,
		host: true // Allows access from LAN (0.0.0.0)
	},
	preview: {
		port: 5180,
		host: true
	}
});
