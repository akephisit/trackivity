<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Sun, Moon, Monitor } from 'lucide-svelte';
	import { mode, setMode } from 'mode-watcher';

	let { variant = 'outline', size = 'default' } = $props();
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger asChild let:builder>
		<Button builders={[builder]} {variant} {size}>
			<Sun class="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
			<Moon class="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
			<span class="sr-only">เปลี่ยนธีม</span>
		</Button>
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end">
		<DropdownMenu.Item on:click={() => setMode('light')}>
			<Sun class="mr-2 h-4 w-4" />
			โหมดสว่าง
		</DropdownMenu.Item>
		<DropdownMenu.Item on:click={() => setMode('dark')}>
			<Moon class="mr-2 h-4 w-4" />
			โหมดมืด
		</DropdownMenu.Item>
		<DropdownMenu.Item on:click={() => setMode('system')}>
			<Monitor class="mr-2 h-4 w-4" />
			ตามระบบ
		</DropdownMenu.Item>
	</DropdownMenu.Content>
</DropdownMenu.Root>