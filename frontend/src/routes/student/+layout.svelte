<script lang="ts">
	import { currentUser, isAuthenticated } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { cn } from '$lib/utils';
	import { Button } from '$lib/components/ui/button';
	import { 
		IconHome, 
		IconCalendarEvent, 
		IconQrcode, 
		IconUser, 
		IconHistory,
		IconLogout,
		IconMenu,
		IconX
	} from '@tabler/icons-svelte';
	import { page } from '$app/stores';

	let { children } = $props();
	let mobileMenuOpen = $state(false);

	// Navigation items for students
	const navItems = [
		{ href: '/student', icon: IconHome, label: 'หน้าหลัก', exact: true },
		{ href: '/student/activities', icon: IconCalendarEvent, label: 'กิจกรรม' },
		{ href: '/student/qr', icon: IconQrcode, label: 'QR Code' },
		{ href: '/student/history', icon: IconHistory, label: 'ประวัติ' },
		{ href: '/student/profile', icon: IconUser, label: 'โปรไฟล์' }
	];

    // Check authentication and user role
    onMount(() => {
        let unsubscribe: () => void = () => {};
        let userUnsubscribe: () => void = () => {};

        (async () => {
            // Ensure we probe server first so store reflects real session
            const { auth } = await import('$lib/stores/auth');
            await auth.refreshUser();

            unsubscribe = isAuthenticated.subscribe((authenticated) => {
                if (!authenticated) {
                    goto('/login');
                    return;
                }
            });

            userUnsubscribe = currentUser.subscribe((user) => {
                if (user && user.admin_role) {
                    // If user is admin, redirect to admin panel
                    goto('/admin');
                    return;
                }
            });
        })();

        return () => {
            unsubscribe();
            userUnsubscribe();
        };
    });

	function isActiveRoute(href: string, exact = false) {
		if (exact) {
			return $page.url.pathname === href;
		}
		return $page.url.pathname.startsWith(href);
	}

	function toggleMobileMenu() {
		mobileMenuOpen = !mobileMenuOpen;
	}

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}

	async function handleLogout() {
		const { auth } = await import('$lib/stores/auth');
		await auth.logout();
	}
</script>

<div class="min-h-screen bg-background">
	<!-- Mobile Header -->
	<header class="lg:hidden bg-card border-b sticky top-0 z-40">
		<div class="flex items-center justify-between px-4 py-3">
			<div class="flex items-center gap-3">
				<Button
					variant="ghost"
					size="sm"
					onclick={toggleMobileMenu}
					class="p-2"
				>
					{#if mobileMenuOpen}
						<IconX class="size-5" />
					{:else}
						<IconMenu class="size-5" />
					{/if}
				</Button>
				<h1 class="font-semibold text-lg">Trackivity</h1>
			</div>
			
			{#if $currentUser}
				<div class="flex items-center gap-2">
					<span class="text-sm text-muted-foreground">
						{$currentUser.first_name}
					</span>
					<Button
						variant="ghost"
						size="sm"
						onclick={handleLogout}
						class="p-2"
					>
						<IconLogout class="size-4" />
					</Button>
				</div>
			{/if}
		</div>
	</header>

	<div class="flex">
		<!-- Desktop Sidebar -->
		<aside class="hidden lg:flex w-64 bg-card border-r min-h-screen flex-col">
			<!-- Logo -->
			<div class="p-6 border-b">
				<h1 class="text-xl font-bold text-foreground">Trackivity</h1>
				<p class="text-sm text-muted-foreground mt-1">Student Portal</p>
			</div>

			<!-- Navigation -->
			<nav class="flex-1 px-4 py-6 space-y-2">
				{#each navItems as item}
					<a
						href={item.href}
						class={cn(
							"flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
							isActiveRoute(item.href, item.exact)
								? "bg-primary text-primary-foreground"
								: "text-muted-foreground hover:text-foreground hover:bg-muted"
						)}
					>
						<item.icon class="size-5" />
						{item.label}
					</a>
				{/each}
			</nav>

			<!-- User Info -->
			{#if $currentUser}
				<div class="p-4 border-t">
					<div class="space-y-2">
						<p class="text-sm font-medium">
							{$currentUser.first_name} {$currentUser.last_name}
						</p>
						<p class="text-xs text-muted-foreground">
							รหัส: {$currentUser.student_id}
						</p>
						<Button
							variant="ghost"
							size="sm"
							onclick={handleLogout}
							class="w-full justify-start"
						>
							<IconLogout class="size-4 mr-2" />
							ออกจากระบบ
						</Button>
					</div>
				</div>
			{/if}
		</aside>

		<!-- Mobile Menu Overlay -->
		{#if mobileMenuOpen}
			<div 
				class="lg:hidden fixed inset-0 bg-black/20 z-50"
				role="button"
				tabindex="0"
				aria-label="Close mobile menu"
				onclick={closeMobileMenu}
				onkeydown={(e) => e.key === 'Escape' && closeMobileMenu()}
			></div>
		{/if}

		<!-- Mobile Sidebar -->
		<aside class={cn(
			"lg:hidden fixed left-0 top-0 w-64 bg-card border-r min-h-screen flex-col z-50 transform transition-transform duration-300",
			mobileMenuOpen ? "translate-x-0" : "-translate-x-full"
		)}>
			<!-- Logo -->
			<div class="p-6 border-b flex items-center justify-between">
				<div>
					<h1 class="text-xl font-bold text-foreground">Trackivity</h1>
					<p class="text-sm text-muted-foreground mt-1">Student Portal</p>
				</div>
				<Button
					variant="ghost"
					size="sm"
					onclick={closeMobileMenu}
					class="p-2"
				>
					<IconX class="size-5" />
				</Button>
			</div>

			<!-- Navigation -->
			<nav class="flex-1 px-4 py-6 space-y-2">
				{#each navItems as item}
					<a
						href={item.href}
						onclick={closeMobileMenu}
						class={cn(
							"flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
							isActiveRoute(item.href, item.exact)
								? "bg-primary text-primary-foreground"
								: "text-muted-foreground hover:text-foreground hover:bg-muted"
						)}
					>
						<item.icon class="size-5" />
						{item.label}
					</a>
				{/each}
			</nav>

			<!-- User Info -->
			{#if $currentUser}
				<div class="p-4 border-t">
					<div class="space-y-2">
						<p class="text-sm font-medium">
							{$currentUser.first_name} {$currentUser.last_name}
						</p>
						<p class="text-xs text-muted-foreground">
							รหัส: {$currentUser.student_id}
						</p>
						<Button
							variant="ghost"
							size="sm"
							onclick={handleLogout}
							class="w-full justify-start"
						>
							<IconLogout class="size-4 mr-2" />
							ออกจากระบบ
						</Button>
					</div>
				</div>
			{/if}
		</aside>

		<!-- Main Content -->
		<main class="flex-1 lg:ml-0">
			<div class="container mx-auto px-4 py-6 lg:px-6 max-w-7xl">
				{@render children?.()}
			</div>
		</main>
	</div>

	<!-- Bottom Navigation for Mobile -->
	<nav class="lg:hidden fixed bottom-0 left-0 right-0 bg-card border-t z-30">
		<div class="flex items-center justify-around py-2">
			{#each navItems.slice(0, 4) as item}
				<a
					href={item.href}
					class={cn(
						"flex flex-col items-center gap-1 px-3 py-2 text-xs font-medium transition-colors min-w-0",
						isActiveRoute(item.href, item.exact)
							? "text-primary"
							: "text-muted-foreground"
					)}
				>
					<item.icon class="size-5 flex-shrink-0" />
					<span class="truncate">{item.label}</span>
				</a>
			{/each}
		</div>
	</nav>

	<!-- Bottom padding for mobile bottom nav -->
	<div class="lg:hidden h-16"></div>
</div>

<style>
	.line-clamp-2 {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
