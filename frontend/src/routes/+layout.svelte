<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import { isAuthenticated } from '$lib/stores/auth';

	let { children } = $props();

	// No SSE in this build
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />
<Toaster richColors closeButton />

<!-- SSE removed for now -->

{@render children?.()}
