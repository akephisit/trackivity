<script lang="ts">
	import '../app.css';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from 'svelte-sonner';
</script>

<ModeWatcher />
<Toaster richColors />

<main class="min-h-screen bg-background">
	<slot />
</main>