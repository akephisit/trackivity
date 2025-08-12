<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import { sseClient } from '$lib/sse/client';
	import { isAuthenticated } from '$lib/stores/auth';
	import { onMount } from 'svelte';

	let { children } = $props();
	let mounted = $state(false);
	onMount(() => (mounted = true));

	// SSE Connection Status
	const connectionStatus = sseClient.connectionStatus;
	const lastEvent = sseClient.lastEvent;

	// Debug info in development
	if (typeof window !== 'undefined' && window.location.hostname === 'localhost') {
		connectionStatus.subscribe(status => {
			console.log('[SSE] Connection status:', status);
		});

		lastEvent.subscribe(event => {
			if (event) {
				console.log('[SSE] Received event:', event.event_type, event);
			}
		});
	}
</script>

<!-- Move favicon to app.html to avoid head manipulation during hydration -->

{#if mounted}
	<ModeWatcher />
	<Toaster richColors closeButton />
{/if}

<!-- SSE Connection Status Indicator (Development Only) -->
{#if mounted && typeof window !== 'undefined' && window.location.hostname === 'localhost' && $isAuthenticated}
	<div class="fixed top-4 right-4 z-50 flex items-center gap-2 text-xs bg-background/80 backdrop-blur-sm border rounded-lg px-3 py-2 shadow-sm">
		<div class="flex items-center gap-2">
			{#if $connectionStatus === 'connected'}
				<div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
				<span class="text-green-600">SSE Connected</span>
			{:else if $connectionStatus === 'connecting'}
				<div class="w-2 h-2 bg-yellow-500 rounded-full animate-pulse"></div>
				<span class="text-yellow-600">SSE Connecting...</span>
			{:else if $connectionStatus === 'reconnecting'}
				<div class="w-2 h-2 bg-orange-500 rounded-full animate-pulse"></div>
				<span class="text-orange-600">SSE Reconnecting...</span>
			{:else if $connectionStatus === 'error'}
				<div class="w-2 h-2 bg-red-500 rounded-full"></div>
				<span class="text-red-600">SSE Error</span>
			{:else}
				<div class="w-2 h-2 bg-gray-400 rounded-full"></div>
				<span class="text-gray-500">SSE Disconnected</span>
			{/if}
		</div>
	</div>
{/if}

{@render children()}
