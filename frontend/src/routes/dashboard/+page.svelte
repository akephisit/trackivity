<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { authStore } from '$lib/stores/auth';
	import { apiClient } from '$lib/api/client';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	let user: any = null;

	onMount(() => {
		const unsubscribe = authStore.subscribe(state => {
			user = state.user;
			if (!state.isAuthenticated && !state.isLoading) {
				goto('/auth/login');
			}
		});

		return unsubscribe;
	});

	async function handleLogout() {
		try {
			await apiClient.logout();
			authStore.logout();
			toast.success('ออกจากระบบสำเร็จ');
			goto('/');
		} catch (error) {
			console.error('Logout error:', error);
			// Even if logout fails on server, clear local state
			authStore.logout();
			goto('/');
		}
	}
</script>

<svelte:head>
	<title>แดชบอร์ด - Trackivity</title>
</svelte:head>

{#if user}
	<div class="container mx-auto px-4 py-8">
		<header class="flex justify-between items-center mb-8">
			<div>
				<h1 class="text-3xl font-bold">แดชบอร์ด</h1>
				<p class="text-muted-foreground">
					ยินดีต้อนรับ, {user.first_name} {user.last_name}
				</p>
			</div>
			<Button variant="outline" on:click={handleLogout}>
				ออกจากระบบ
			</Button>
		</header>

		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
			<Card>
				<CardHeader>
					<CardTitle>กิจกรรมของฉัน</CardTitle>
					<CardDescription>
						กิจกรรมที่เข้าร่วมและจะเข้าร่วม
					</CardDescription>
				</CardHeader>
				<CardContent>
					<p class="text-2xl font-bold">0</p>
					<p class="text-sm text-muted-foreground">กิจกรรมทั้งหมด</p>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>กิจกรรมที่เปิดรับสมัคร</CardTitle>
					<CardDescription>
						กิจกรรมที่สามารถลงทะเบียนได้
					</CardDescription>
				</CardHeader>
				<CardContent>
					<p class="text-2xl font-bold">0</p>
					<p class="text-sm text-muted-foreground">กิจกรรมเปิดรับสมัคร</p>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>QR Code ของฉัน</CardTitle>
					<CardDescription>
						QR Code สำหรับเช็คอินกิจกรรม
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Button class="w-full">ดู QR Code</Button>
				</CardContent>
			</Card>
		</div>

		<div class="mt-8">
			<h2 class="text-2xl font-bold mb-4">กิจกรรมล่าสุด</h2>
			<Card>
				<CardContent class="p-6">
					<p class="text-center text-muted-foreground">
						ยังไม่มีกิจกรรม
					</p>
				</CardContent>
			</Card>
		</div>
	</div>
{:else}
	<div class="container mx-auto px-4 py-8 flex items-center justify-center min-h-screen">
		<div class="text-center">
			<p>กำลังโหลด...</p>
		</div>
	</div>
{/if}