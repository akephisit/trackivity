<script lang="ts">
	import { page } from '$app/state';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AdminSidebar from '$lib/components/admin-sidebar.svelte';

	let { data, children } = $props();

	let isLoginPage = $derived(page.url.pathname === '/admin/login');
</script>

{#if isLoginPage}
	<div class="min-h-screen bg-background">
		{@render children()}
	</div>
{:else}
	<Sidebar.Provider>
		<AdminSidebar user={data.user} admin_role={data.admin_role} />
		<Sidebar.Inset>
			<!-- Header -->
			<header class="flex h-16 shrink-0 items-center gap-2 px-4 border-b">
				<Sidebar.Trigger class="md:hidden" />
				<div class="flex items-center justify-between w-full">
					<div class="flex items-center gap-2">
						<h1 class="text-lg font-semibold">
							{#if page.url.pathname === '/admin'}
								แดชบอร์ด
							{:else if page.url.pathname.startsWith('/admin/activities')}
								จัดการกิจกรรม
							{:else if page.url.pathname.startsWith('/admin/faculties')}
								จัดการคณะ
							{:else if page.url.pathname.startsWith('/admin/admins')}
								จัดการแอดมิน
							{:else if page.url.pathname.startsWith('/admin/departments')}
								จัดการภาควิชา
							{:else if page.url.pathname.startsWith('/admin/faculty-users')}
								จัดการผู้ใช้คณะ
							{:else if page.url.pathname.startsWith('/admin/faculty-admins')}
								จัดการแอดมินคณะ
							{:else if page.url.pathname.startsWith('/admin/settings')}
								ตั้งค่า
							{:else}
								Admin Panel
							{/if}
						</h1>
					</div>
					<div class="flex items-center space-x-4">
						<span class="text-sm text-muted-foreground">
							{new Date().toLocaleDateString('th-TH', { 
								weekday: 'long',
								year: 'numeric',
								month: 'long',
								day: 'numeric'
							})}
						</span>
					</div>
				</div>
			</header>

			<!-- Main Content -->
			<main class="p-6">
				{@render children()}
			</main>
		</Sidebar.Inset>
	</Sidebar.Provider>
{/if}