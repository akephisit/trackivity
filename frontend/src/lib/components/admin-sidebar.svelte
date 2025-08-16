<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";
	import { 
		IconLayoutDashboard, 
		IconUsers, 
		IconBuilding, 
		IconSettings, 
		IconLogout, 
		IconShield,
		IconBuildingStore,
		IconUserCog,
		IconSun,
		IconMoon,
		IconQrcode,
		IconCalendarEvent
	} from '@tabler/icons-svelte/icons';
	import { AdminLevel } from '$lib/types/admin';
	import { mode, setMode } from 'mode-watcher';
	import { toast } from 'svelte-sonner';
	import type { ComponentProps } from "svelte";

	interface AdminSidebarProps extends ComponentProps<typeof Sidebar.Root> {
		user?: any;
		admin_role?: {
			admin_level: AdminLevel;
			faculty_id?: string;
			role_name?: string;
		};
	}

	let { user, admin_role, ...restProps }: AdminSidebarProps = $props();

	// Navigation items based on admin level
	let navigationItems = $derived(getNavigationItems(admin_role?.admin_level));
	
	// Check if we have user data
	let hasUser = $derived(user != null);

	interface NavigationItem {
		title: string;
		href: string;
		icon: any;
		active: boolean;
		description?: string;
	}

	function getNavigationItems(adminLevel?: AdminLevel): NavigationItem[] {
		const baseItems: NavigationItem[] = [
			{
				title: 'แดชบอร์ด',
				href: '/admin',
				icon: IconLayoutDashboard,
				active: page.url.pathname === '/admin'
			},
			{
				title: 'จัดการกิจกรรม',
				href: '/admin/activities',
				icon: IconCalendarEvent,
				active: page.url.pathname.startsWith('/admin/activities'),
				description: 'จัดการกิจกรรมและการเข้าร่วม'
			},
			{
				title: 'สแกน QR Code',
				href: '/admin/qr-scanner',
				icon: IconQrcode,
				active: page.url.pathname.startsWith('/admin/qr-scanner'),
				description: 'สแกน QR Code เพื่อบันทึกการเข้าร่วมกิจกรรม'
			}
		];

		// เพิ่มรายการเมนูตามระดับแอดมิน
		if (adminLevel === AdminLevel.SuperAdmin) {
			baseItems.push(
				{
					title: 'จัดการผู้ใช้',
					href: '/admin/users',
					icon: IconUsers,
					active: page.url.pathname.startsWith('/admin/users'),
					description: 'จัดการผู้ใช้ทั้งหมดในระบบ'
				},
				{
					title: 'จัดการคณะ',
					href: '/admin/faculties',
					icon: IconBuilding,
					active: page.url.pathname.startsWith('/admin/faculties'),
					description: 'จัดการข้อมูลคณะต่างๆ'
				},
				{
					title: 'จัดการแอดมิน',
					href: '/admin/admins',
					icon: IconShield,
					active: page.url.pathname.startsWith('/admin/admins'),
					description: 'จัดการผู้ดูแลระบบ'
				}
			);
		} else if (adminLevel === AdminLevel.FacultyAdmin) {
			baseItems.push(
				{
					title: 'จัดการภาควิชา',
					href: '/admin/departments',
					icon: IconBuildingStore,
					active: page.url.pathname.startsWith('/admin/departments'),
					description: 'จัดการภาควิชาในคณะ'
				},
				{
					title: 'จัดการผู้ใช้คณะ',
					href: '/admin/faculty-users',
					icon: IconUsers,
					active: page.url.pathname.startsWith('/admin/faculty-users'),
					description: 'จัดการผู้ใช้ในคณะ'
				},
				{
					title: 'จัดการแอดมินคณะ',
					href: '/admin/faculty-admins',
					icon: IconUserCog,
					active: page.url.pathname.startsWith('/admin/faculty-admins'),
					description: 'จัดการผู้ดูแลคณะ'
				}
			);
		}

		baseItems.push({
			title: 'ตั้งค่า',
			href: '/admin/settings',
			icon: IconSettings,
			active: page.url.pathname.startsWith('/admin/settings'),
			description: 'ตั้งค่าระบบ'
		});

		return baseItems;
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

	function getAdminLevelText(level?: string) {
		if (level === 'SuperAdmin' || level === 'super_admin') return 'ซุปเปอร์แอดมิน';
		if (level === 'FacultyAdmin' || level === 'faculty_admin') return 'แอดมินคณะ';
		return 'แอดมินทั่วไป';
	}

	function getRoleDisplayName(level?: AdminLevel, facultyName?: string) {
		if (level === AdminLevel.SuperAdmin) {
			return 'ซุปเปอร์แอดมิน';
		} else if (level === AdminLevel.FacultyAdmin) {
			return facultyName ? `แอดมินคณะ${facultyName}` : 'แอดมินคณะ';
		}
		return 'แอดมิน';
	}
</script>

<Sidebar.Root collapsible="offcanvas" {...restProps}>
	<!-- Header -->
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton class="data-[slot=sidebar-menu-button]:!p-3">
					{#snippet child({ props })}
						<a href="/admin" {...props} class="flex items-center space-x-3">
							<div class="flex-shrink-0 w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
								<IconShield class="!w-5 !h-5 text-primary-foreground" />
							</div>
							<div class="flex flex-col items-start">
								<span class="text-base font-bold">Admin Panel</span>
								<span class="text-xs text-muted-foreground">
									{getRoleDisplayName(admin_role?.admin_level)}
								</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>

	<!-- Content -->
	<Sidebar.Content>
		<!-- User Info Section -->
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<div class="p-3 border border-border rounded-lg bg-muted/50">
					<div class="flex items-center space-x-3">
						<div class="flex-shrink-0">
							<div class="h-10 w-10 rounded-full bg-primary flex items-center justify-center">
								<span class="text-white font-medium text-sm">
									{hasUser ? (user.first_name?.charAt(0) || user.email?.charAt(0))?.toUpperCase() : 'A'}
								</span>
							</div>
						</div>
						<div class="flex-1 min-w-0">
							<p class="text-sm font-medium truncate">
								{hasUser ? `${user.first_name || ''} ${user.last_name || ''}`.trim() || user.email : 'Admin'}
							</p>
							<p class="text-xs text-muted-foreground truncate">
								{getAdminLevelText(admin_role?.admin_level)}
							</p>
						</div>
					</div>
				</div>
			</Sidebar.GroupContent>
		</Sidebar.Group>

		<!-- Navigation Menu -->
		<Sidebar.Group>
			<Sidebar.GroupLabel>เมนูหลัก</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each navigationItems as item}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton isActive={item.active}>
								{#snippet child({ props })}
									<a href={item.href} {...props} class="flex items-center space-x-3">
										{#if item.icon}
											{@const IconComponent = item.icon}
											<IconComponent class="!w-4 !h-4 flex-shrink-0" />
										{/if}
										<div class="flex flex-col items-start">
											<span class="text-sm font-medium">{item.title}</span>
											{#if item.description}
												<span class="text-xs text-muted-foreground">{item.description}</span>
											{/if}
										</div>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>

		<!-- Quick Actions for Faculty Admin -->
		{#if admin_role?.admin_level === AdminLevel.FacultyAdmin}
			<Sidebar.Group>
				<Sidebar.GroupLabel>การดำเนินการด่วน</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href="/admin/activities/create" {...props} class="flex items-center space-x-3">
										<IconCalendarEvent class="!w-4 !h-4 flex-shrink-0" />
										<span class="text-sm">สร้างกิจกรรมใหม่</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href="/admin/reports" {...props} class="flex items-center space-x-3">
										<IconUsers class="!w-4 !h-4 flex-shrink-0" />
										<span class="text-sm">รายงานคณะ</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/if}
	</Sidebar.Content>

	<!-- Footer -->
	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton onclick={toggleTheme}>
					{#snippet child({ props })}
						<button {...props} class="flex items-center space-x-3 w-full">
							{#if mode.current === 'light'}
								<IconMoon class="!w-4 !h-4 flex-shrink-0" />
								<span class="text-sm">โหมดมืด</span>
							{:else}
								<IconSun class="!w-4 !h-4 flex-shrink-0" />
								<span class="text-sm">โหมดสว่าง</span>
							{/if}
						</button>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton onclick={handleLogout}>
					{#snippet child({ props })}
						<button {...props} class="flex items-center space-x-3 w-full text-destructive hover:text-destructive">
							<IconLogout class="!w-4 !h-4 flex-shrink-0" />
							<span class="text-sm">ออกจากระบบ</span>
						</button>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>
</Sidebar.Root>