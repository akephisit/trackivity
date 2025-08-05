<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { 
		IconLayoutDashboard, 
		IconUsers, 
		IconBuilding, 
		IconSettings, 
		IconLogout, 
		IconShield,
		IconMenu2,
		IconX,
		IconSun,
		IconMoon
	} from '@tabler/icons-svelte/icons';
	import { AdminLevel } from '$lib/types/admin';
	import { mode, setMode } from 'mode-watcher';
	import { toast } from 'svelte-sonner';

	let { data, children } = $props();

	let sidebarOpen = $state(false);

	// Navigation items based on admin level
	let navigationItems = $derived(getNavigationItems(data.admin_role?.admin_level));

	function getNavigationItems(adminLevel?: AdminLevel) {
		const baseItems = [
			{
				title: 'แดชบอร์ด',
				href: '/admin',
				icon: IconLayoutDashboard,
				active: $page.url.pathname === '/admin'
			}
		];

		// เพิ่มรายการเมนูตามระดับแอดมิน
		if (adminLevel === AdminLevel.SuperAdmin) {
			baseItems.push(
				{
					title: 'จัดการผู้ใช้',
					href: '/admin/users',
					icon: IconUsers,
					active: $page.url.pathname.startsWith('/admin/users')
				},
				{
					title: 'จัดการคณะ',
					href: '/admin/faculties',
					icon: IconBuilding,
					active: $page.url.pathname.startsWith('/admin/faculties')
				},
				{
					title: 'จัดการแอดมิน',
					href: '/admin/admins',
					icon: IconShield,
					active: $page.url.pathname.startsWith('/admin/admins')
				}
			);
		} else if (adminLevel === AdminLevel.FacultyAdmin) {
			baseItems.push(
				{
					title: 'จัดการผู้ใช้คณะ',
					href: '/admin/users',
					icon: IconUsers,
					active: $page.url.pathname.startsWith('/admin/users')
				}
			);
		}

		baseItems.push({
			title: 'ตั้งค่า',
			href: '/admin/settings',
			icon: IconSettings,
			active: $page.url.pathname.startsWith('/admin/settings')
		});

		return baseItems;
	}

	function toggleSidebar() {
		sidebarOpen = !sidebarOpen;
	}

	function closeSidebar() {
		sidebarOpen = false;
	}

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

	function toggleTheme() {
		setMode(mode.current === 'light' ? 'dark' : 'light');
	}
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900">
	<!-- Mobile sidebar overlay -->
	{#if sidebarOpen}
		<div class="fixed inset-0 z-40 md:hidden">
			<button 
				class="fixed inset-0 bg-black bg-opacity-50 w-full h-full cursor-pointer" 
				onclick={closeSidebar}
				aria-label="Close sidebar"
			></button>
		</div>
	{/if}

	<!-- Sidebar -->
	<div class="fixed inset-y-0 left-0 z-50 w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 transform transition-transform duration-300 ease-in-out md:translate-x-0 {sidebarOpen ? 'translate-x-0' : '-translate-x-full'} md:static md:inset-0">
		<div class="flex flex-col h-full">
			<!-- Header -->
			<div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
				<h1 class="text-xl font-bold text-gray-900 dark:text-white">
					Admin Panel
				</h1>
				<button
					onclick={closeSidebar}
					class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 md:hidden"
				>
					<IconX class="h-5 w-5" />
				</button>
			</div>

			<!-- User info -->
			<div class="p-4 border-b border-gray-200 dark:border-gray-700">
				<div class="flex items-center space-x-3">
					<div class="flex-shrink-0">
						<div class="h-10 w-10 rounded-full bg-primary flex items-center justify-center">
							<span class="text-white font-medium">
								{data.user.name?.charAt(0).toUpperCase()}
							</span>
						</div>
					</div>
					<div class="flex-1 min-w-0">
						<p class="text-sm font-medium text-gray-900 dark:text-white truncate">
							{data.user.name}
						</p>
						<p class="text-xs text-gray-500 dark:text-gray-400 truncate">
							{data.admin_role?.admin_level === AdminLevel.SuperAdmin ? 'ซุปเปอร์แอดมิน' :
							 data.admin_role?.admin_level === AdminLevel.FacultyAdmin ? 'แอดมินคณะ' :
							 'แอดมินทั่วไป'}
						</p>
					</div>
				</div>
			</div>

			<!-- Navigation -->
			<nav class="flex-1 p-4 space-y-2">
				{#each navigationItems as item}
					<a
						href={item.href}
						onclick={closeSidebar}
						class="flex items-center space-x-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors
							{item.active ? 
								'bg-primary text-primary-foreground' : 
								'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
							}"
					>
						{#if item.icon}
							{@const IconComponent = item.icon}
							<IconComponent class="h-5 w-5" />
						{/if}
						<span>{item.title}</span>
					</a>
				{/each}
			</nav>

			<!-- Footer actions -->
			<div class="p-4 border-t border-gray-200 dark:border-gray-700 space-y-2">
				<Button
					variant="ghost"
					size="sm"
					onclick={toggleTheme}
					class="w-full justify-start"
				>
					{#if mode.current === 'light'}
						<IconMoon class="h-4 w-4 mr-2" />
						โหมดมืด
					{:else}
						<IconSun class="h-4 w-4 mr-2" />
						โหมดสว่าง
					{/if}
				</Button>
				
				<Button
					variant="ghost"
					size="sm"
					onclick={handleLogout}
					class="w-full justify-start text-red-600 hover:text-red-700 hover:bg-red-50 dark:hover:bg-red-900/20"
				>
					<IconLogout class="h-4 w-4 mr-2" />
					ออกจากระบบ
				</Button>
			</div>
		</div>
	</div>

	<!-- Main content -->
	<div class="md:pl-64">
		<!-- Top bar -->
		<header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-3">
			<div class="flex items-center justify-between">
				<button
					onclick={toggleSidebar}
					class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 md:hidden"
				>
					<IconMenu2 class="h-5 w-5" />
				</button>

				<div class="flex items-center space-x-4">
					<span class="text-sm text-gray-500 dark:text-gray-400">
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

		<!-- Page content -->
		<main class="p-6">
			{@render children()}
		</main>
	</div>
</div>