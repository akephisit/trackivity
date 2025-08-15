<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import { isAuthenticated } from '$lib/stores/auth';
	import { activityTracker } from '$lib/activity-tracker';
	import { onMount } from 'svelte';

	let { children } = $props();

	// Initialize activity tracker when authenticated
	onMount(() => {
		const unsubscribe = isAuthenticated.subscribe((authenticated) => {
			if (authenticated) {
				activityTracker.startTracking();
			} else {
				activityTracker.stopTracking();
			}
		});

		return unsubscribe;
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />
<Toaster richColors closeButton />

<!-- SSE removed for now -->

{@render children?.()}
