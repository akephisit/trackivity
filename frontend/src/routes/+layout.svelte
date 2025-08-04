<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth';
	import { apiClient } from '$lib/api/client';

	let { children } = $props();

	onMount(async () => {
		// Check if user is already logged in
		try {
			const user = await apiClient.me();
			authStore.login(user);
		} catch (error) {
			// Not logged in, that's fine
			console.log('Not authenticated');
		}
	});
</script>

<svelte:head>
	<title>Trackivity - ระบบเก็บกิจกรรมมหาวิทยาลัย</title>
	<meta name="description" content="ระบบจัดการกิจกรรมมหาวิทยาลัย" />
</svelte:head>

{@render children?.()}