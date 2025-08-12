<script lang="ts">
	import ActivitiesTable from "$lib/components/activities-table.svelte";
	import { mockActivities } from "$lib/data/activities.js";
	import { Button } from "$lib/components/ui/button";
	import { IconUser, IconLogout } from '@tabler/icons-svelte/icons';
	import type { PageData } from './$types';
    import { toast } from 'svelte-sonner';
    import { goto } from '$app/navigation';

	let { data }: { data: PageData } = $props();

	async function handleLogout() {
		try {
			const response = await fetch('/api/auth/logout', {
				method: 'POST'
			});

			if (response.ok) {
				toast.success('ออกจากระบบสำเร็จ');
				goto('/login');
			} else {
				toast.error('เกิดข้อผิดพลาดในการออกจากระบบ');
			}
		} catch (error) {
			console.error('Logout error:', error);
			toast.error('เกิดข้อผิดพลาดในการออกจากระบบ');
		}
	}
</script>

<div class="min-h-screen bg-background">
	<!-- Header -->
	<header class="border-b bg-card">
		<div class="container mx-auto px-4 py-6">
			<div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
				<div>
					<h1 class="text-3xl font-bold text-foreground">ระบบติดตามกิจกรรม</h1>
					<p class="text-muted-foreground mt-1">
						จัดการและติดตามกิจกรรมทั้งหมดของมหาวิทยาลัยในที่เดียว
					</p>
					{#if data.user}
						<p class="text-sm text-muted-foreground mt-2">
							ยินดีต้อนรับ, {data.user.first_name} {data.user.last_name}
						</p>
					{/if}
				</div>
				<div class="flex flex-col sm:flex-row items-end sm:items-center gap-3">
					<div class="text-sm text-muted-foreground">
						{new Date().toLocaleDateString('th-TH', {
							year: 'numeric',
							month: 'long',
							day: 'numeric',
							weekday: 'long'
						})}
					</div>
					{#if data.user}
						<div class="flex items-center gap-2">
							{#if data.user.admin_role}
								<Button href="/admin" variant="outline" size="sm" class="flex items-center gap-2">
									<IconUser class="h-4 w-4" />
									แอดมิน
								</Button>
							{:else}
								<Button href="/student" variant="outline" size="sm" class="flex items-center gap-2">
									<IconUser class="h-4 w-4" />
									นักศึกษา
								</Button>
							{/if}
							<Button onclick={handleLogout} variant="outline" size="sm" class="flex items-center gap-2">
								<IconLogout class="h-4 w-4" />
								ออกจากระบบ
							</Button>
						</div>
					{:else}
						<Button href="/login" variant="outline" size="sm" class="flex items-center gap-2">
							<IconUser class="h-4 w-4" />
							เข้าสู่ระบบ
						</Button>
					{/if}
				</div>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<main class="container mx-auto px-4 py-8">
		<ActivitiesTable activities={mockActivities} />
	</main>

	<!-- Footer -->
	<footer class="border-t bg-muted/30 mt-16">
		<div class="container mx-auto px-4 py-6">
			<div class="text-center text-sm text-muted-foreground">
				<p>© 2024 ระบบติดตามกิจกรรม - มหาวิทยาลัยเทคโนโลยีพระจอมเกล้าธนบุรี</p>
			</div>
		</div>
	</footer>
</div>
