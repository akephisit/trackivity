<script lang="ts">
	import { onMount, createEventDispatcher } from 'svelte';
	import { slide, fade } from 'svelte/transition';
	import { quintOut } from 'svelte/easing';
	import { formatDistanceToNow } from 'date-fns';
	import { th } from 'date-fns/locale';
	
	import { sseService, notifications, unreadCount, isConnected } from '$lib/stores/sse';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { ScrollArea } from '$lib/components/ui/scroll-area';
	import { Separator } from '$lib/components/ui/separator';
	import { 
		IconBell, 
		IconBellRinging, 
		IconX, 
		IconAlertCircle, 
		IconCircleCheck, 
		IconInfoCircle, 
		IconAlertTriangle,
		IconTrash,
		IconCheck,
		IconExternalLink,
		IconWifi,
		IconWifiOff
	} from '@tabler/icons-svelte/icons';

	const dispatch = createEventDispatcher();

	// Component props
	export let showConnectionStatus = true;
	export let maxHeight = '400px';
	export let autoClose = true;
	export let autoCloseDelay = 5000;

	// Component state
	let isOpen = false;
	let autoCloseTimeouts = new Map<number, NodeJS.Timeout>();

	// Auto-close notifications
	$: if (autoClose && $notifications.length > 0) {
		$notifications.forEach((notification, index) => {
			if (!autoCloseTimeouts.has(index) && !notification.expires_at) {
				const timeout = setTimeout(() => {
					removeNotification(index);
				}, autoCloseDelay);
				autoCloseTimeouts.set(index, timeout);
			}
		});
	}

	// Clean up timeouts when notifications change
	$: if ($notifications.length === 0) {
		autoCloseTimeouts.forEach(timeout => clearTimeout(timeout));
		autoCloseTimeouts.clear();
	}

	onMount(() => {
		return () => {
			// Clean up timeouts on component destroy
			autoCloseTimeouts.forEach(timeout => clearTimeout(timeout));
		};
	});

	// Toggle notification center
	function toggleNotifications() {
		isOpen = !isOpen;
		dispatch('toggle', { isOpen });
	}

	// Remove specific notification
	function removeNotification(index: number) {
		if (autoCloseTimeouts.has(index)) {
			clearTimeout(autoCloseTimeouts.get(index)!);
			autoCloseTimeouts.delete(index);
		}
		sseService.removeNotification(index);
		dispatch('notificationRemoved', { index });
	}

	// Mark notification as read
	function markAsRead(index: number) {
		sseService.markAsRead(index);
		dispatch('notificationRead', { index });
	}

	// Clear all notifications
	function clearAllNotifications() {
		autoCloseTimeouts.forEach(timeout => clearTimeout(timeout));
		autoCloseTimeouts.clear();
		sseService.clearNotifications();
		dispatch('allCleared');
	}

	// Handle notification click
	function handleNotificationClick(notification: any, index: number) {
		if (notification.action_url) {
			window.open(notification.action_url, '_blank');
		}
		markAsRead(index);
		dispatch('notificationClicked', { notification, index });
	}

	// Get notification icon based on type
	function getNotificationIcon(type: string) {
		switch (type) {
			case 'success':
				return IconCircleCheck;
			case 'error':
				return IconAlertCircle;
			case 'warning':
				return IconAlertTriangle;
			case 'info':
			default:
				return IconInfoCircle;
		}
	}

	// Get notification color classes
	function getNotificationClasses(type: string) {
		switch (type) {
			case 'success':
				return 'border-l-green-500 bg-green-50 text-green-900';
			case 'error':
				return 'border-l-red-500 bg-red-50 text-red-900';
			case 'warning':
				return 'border-l-yellow-500 bg-yellow-50 text-yellow-900';
			case 'info':
			default:
				return 'border-l-blue-500 bg-blue-50 text-blue-900';
		}
	}

	// Format notification time
	function formatNotificationTime(timestamp: string): string {
		try {
			return formatDistanceToNow(new Date(timestamp), { 
				addSuffix: true, 
				locale: th 
			});
		} catch {
			return 'เมื่อสักครู่';
		}
	}

	// Check if notification is expired
	function isNotificationExpired(notification: any): boolean {
		return notification.expires_at && new Date(notification.expires_at) <= new Date();
	}
</script>

<!-- Notification Bell Button -->
<div class="relative">
	<Button
		variant="ghost"
		size="sm"
		class="relative p-2"
		onclick={toggleNotifications}
		aria-label="การแจ้งเตือน"
	>
		{#if $unreadCount > 0}
			<IconBellRinging class="h-5 w-5" />
			<Badge 
				variant="destructive" 
				class="absolute -top-1 -right-1 h-5 w-5 p-0 flex items-center justify-center text-xs"
			>
				{$unreadCount > 99 ? '99+' : $unreadCount}
			</Badge>
		{:else}
			<IconBell class="h-5 w-5" />
		{/if}
	</Button>

	<!-- Connection Status Indicator -->
	{#if showConnectionStatus}
		<div class="absolute -bottom-1 -right-1">
			{#if $isConnected}
				<div class="h-2 w-2 bg-green-500 rounded-full" title="เชื่อมต่อแล้ว"></div>
			{:else}
				<div class="h-2 w-2 bg-red-500 rounded-full animate-pulse" title="ไม่ได้เชื่อมต่อ"></div>
			{/if}
		</div>
	{/if}

	<!-- Notification Panel -->
	{#if isOpen}
		<div 
			class="absolute right-0 top-full mt-2 w-80 z-50"
			transition:slide={{ duration: 200, easing: quintOut }}
		>
			<Card class="border shadow-lg">
				<!-- Header -->
				<div class="p-4 border-b">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-2">
							<h3 class="font-semibold">การแจ้งเตือน</h3>
							{#if $unreadCount > 0}
								<Badge variant="secondary" class="text-xs">
									{$unreadCount}
								</Badge>
							{/if}
						</div>
						<div class="flex items-center gap-1">
							<!-- Connection Status -->
							{#if showConnectionStatus}
								<div class="flex items-center gap-1 text-xs">
									{#if $isConnected}
										<IconWifi class="h-3 w-3 text-green-600" />
										<span class="text-green-600">เชื่อมต่อ</span>
									{:else}
										<IconWifiOff class="h-3 w-3 text-red-600" />
										<span class="text-red-600">ขาดการเชื่อมต่อ</span>
									{/if}
								</div>
								<Separator orientation="vertical" class="h-4 mx-2" />
							{/if}
							
							<!-- Clear All Button -->
							{#if $notifications.length > 0}
								<Button
									variant="ghost"
									size="sm"
									class="h-6 w-6 p-0"
									onclick={clearAllNotifications}
									title="ลบทั้งหมด"
								>
									<IconTrash class="h-3 w-3" />
								</Button>
							{/if}
							
							<!-- Close Button -->
							<Button
								variant="ghost"
								size="sm"
								class="h-6 w-6 p-0"
								onclick={toggleNotifications}
								title="ปิด"
							>
								<IconX class="h-3 w-3" />
							</Button>
						</div>
					</div>
				</div>

				<!-- Notifications List -->
				<CardContent class="p-0">
					<ScrollArea class="max-h-[{maxHeight}]">
						{#if $notifications.length === 0}
							<div class="p-8 text-center text-gray-500">
								<IconBell class="h-12 w-12 mx-auto mb-4 opacity-50" />
								<p class="text-sm">ไม่มีการแจ้งเตือน</p>
							</div>
						{:else}
							<div class="divide-y">
								{#each $notifications as notification, index (index)}
									{@const expired = isNotificationExpired(notification)}
									{#if !expired}
										<div 
											class="p-4 hover:bg-gray-50 cursor-pointer transition-colors {getNotificationClasses(notification.notification_type)} border-l-4"
											onclick={() => handleNotificationClick(notification, index)}
											onkeydown={(e) => e.key === 'Enter' && handleNotificationClick(notification, index)}
											role="button"
											tabindex="0"
											transition:fade={{ duration: 150 }}
										>
											<div class="flex items-start gap-3">
												<!-- Icon -->
												<div class="mt-0.5">
													<svelte:component 
														this={getNotificationIcon(notification.notification_type)} 
														class="h-4 w-4" 
													/>
												</div>

												<!-- Content -->
												<div class="flex-1 min-w-0">
													<div class="flex items-start justify-between">
														<div class="flex-1">
															<h4 class="font-medium text-sm mb-1">
																{notification.title}
															</h4>
															<p class="text-sm opacity-90 mb-2">
																{notification.message}
															</p>
															<div class="flex items-center justify-between">
																<span class="text-xs opacity-75">
																	{formatNotificationTime(notification.timestamp || new Date().toISOString())}
																</span>
																{#if notification.action_url}
																	<IconExternalLink class="h-3 w-3 opacity-50" />
																{/if}
															</div>
														</div>

														<!-- Actions -->
														<div class="ml-2 flex items-center gap-1">
															<Button
																variant="ghost"
																size="sm"
																class="h-6 w-6 p-0 opacity-50 hover:opacity-100"
																onclick={(e) => { e.stopPropagation(); markAsRead(index); }}
																title="ทำเครื่องหมายว่าอ่านแล้ว"
															>
																<IconCheck class="h-3 w-3" />
															</Button>
															<Button
																variant="ghost"
																size="sm"
																class="h-6 w-6 p-0 opacity-50 hover:opacity-100 text-red-600"
																onclick={(e) => { e.stopPropagation(); removeNotification(index); }}
																title="ลบ"
															>
																<IconX class="h-3 w-3" />
															</Button>
														</div>
													</div>
												</div>
											</div>
										</div>
									{/if}
								{/each}
							</div>
						{/if}
					</ScrollArea>
				</CardContent>

				<!-- Footer -->
				{#if $notifications.length > 0}
					<div class="p-3 border-t bg-gray-50">
						<div class="flex items-center justify-between text-xs text-gray-600">
							<span>{$notifications.length} การแจ้งเตือน</span>
							{#if $unreadCount > 0}
								<span>{$unreadCount} ยังไม่ได้อ่าน</span>
							{/if}
						</div>
					</div>
				{/if}
			</Card>
		</div>
	{/if}
</div>

<!-- Click outside to close -->
{#if isOpen}
	<div 
		class="fixed inset-0 z-40" 
		onclick={toggleNotifications}
		onkeydown={(e) => e.key === 'Escape' && toggleNotifications()}
		role="button"
		tabindex="-1"
		aria-label="ปิดการแจ้งเตือน"
	></div>
{/if}

<style>
	/* Custom scrollbar for notification area */
	:global(.notification-scroll) {
		scrollbar-width: thin;
		scrollbar-color: rgb(203 213 225) transparent;
	}
	
	:global(.notification-scroll::-webkit-scrollbar) {
		width: 6px;
	}
	
	:global(.notification-scroll::-webkit-scrollbar-track) {
		background: transparent;
	}
	
	:global(.notification-scroll::-webkit-scrollbar-thumb) {
		background-color: rgb(203 213 225);
		border-radius: 3px;
	}
	
	:global(.notification-scroll::-webkit-scrollbar-thumb:hover) {
		background-color: rgb(148 163 184);
	}
</style>