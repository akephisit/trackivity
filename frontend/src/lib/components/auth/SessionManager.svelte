<script lang="ts">
	import { onMount, createEventDispatcher } from 'svelte';
	import { formatDistanceToNow, format } from 'date-fns';
	import { th } from 'date-fns/locale';
	import { authService, user, type SessionInfo } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from '$lib/components/ui/dialog';
	import { 
		IconDeviceDesktop, 
		IconDeviceMobile, 
		IconDeviceTablet, 
		IconClock, 
		IconMapPin, 
		IconTrash, 
		IconShield, 
		IconRefresh,
		IconAlertTriangle,
		IconCalendar,
		IconUser
	} from '@tabler/icons-svelte/icons';

	const dispatch = createEventDispatcher();

	// Component state
	let sessions: SessionInfo[] = [];
	let loading = true;
	let error = '';
	let revoking = new Set<string>();
	let extending = false;

	// Load sessions on mount
	onMount(async () => {
		await loadSessions();
	});

	// Load user sessions
	async function loadSessions() {
		loading = true;
		error = '';
		
		try {
			sessions = await authService.getSessions();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load sessions';
			console.error('Failed to load sessions:', err);
		} finally {
			loading = false;
		}
	}

	// Revoke a specific session
	async function revokeSession(sessionId: string) {
		if (revoking.has(sessionId)) return;
		
		revoking.add(sessionId);
		revoking = new Set(revoking); // Trigger reactivity
		
		try {
			const success = await authService.revokeSession(sessionId);
			
			if (success) {
				// Remove session from list
				sessions = sessions.filter(s => s.session_id !== sessionId);
				dispatch('sessionRevoked', { sessionId });
			} else {
				throw new Error('Failed to revoke session');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to revoke session';
			console.error('Failed to revoke session:', err);
		} finally {
			revoking.delete(sessionId);
			revoking = new Set(revoking);
		}
	}

	// Extend current session
	async function extendCurrentSession() {
		extending = true;
		
		try {
			const success = await authService.extendSession(24);
			
			if (success) {
				await loadSessions(); // Reload to get updated expiry times
				dispatch('sessionExtended');
			} else {
				throw new Error('Failed to extend session');
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to extend session';
			console.error('Failed to extend session:', err);
		} finally {
			extending = false;
		}
	}

	// Get device icon based on type
	function getDeviceIcon(deviceInfo: Record<string, any>) {
		const deviceType = deviceInfo.device_type?.toLowerCase() || 'unknown';
		
		switch (deviceType) {
			case 'mobile':
				return IconDeviceMobile;
			case 'tablet':
				return IconDeviceTablet;
			case 'desktop':
			case 'web':
			default:
				return IconDeviceDesktop;
		}
	}

	// Get device display name
	function getDeviceDisplayName(deviceInfo: Record<string, any>) {
		const deviceType = deviceInfo.device_type || 'Unknown';
		const os = deviceInfo.os;
		const browser = deviceInfo.browser;
		
		if (os && browser) {
			return `${deviceType} - ${os} (${browser})`;
		} else if (os) {
			return `${deviceType} - ${os}`;
		} else if (browser) {
			return `${deviceType} - ${browser}`;
		}
		
		return deviceType;
	}

	// Check if session is current
	function isCurrentSession(sessionId: string): boolean {
		return $user?.session_id === sessionId;
	}

	// Check if session is expiring soon (within 1 hour)
	function isExpiringSoon(expiresAt: string): boolean {
		const expiry = new Date(expiresAt);
		const now = new Date();
		const diff = expiry.getTime() - now.getTime();
		return diff > 0 && diff < 60 * 60 * 1000; // 1 hour in milliseconds
	}

	// Check if session is expired
	function isExpired(expiresAt: string): boolean {
		return new Date(expiresAt) <= new Date();
	}

	// Format relative time
	function formatRelativeTime(dateString: string): string {
		try {
			return formatDistanceToNow(new Date(dateString), { 
				addSuffix: true, 
				locale: th 
			});
		} catch {
			return 'ไม่ทราบ';
		}
	}

	// Format absolute time
	function formatAbsoluteTime(dateString: string): string {
		try {
			return format(new Date(dateString), 'dd MMM yyyy, HH:mm', { locale: th });
		} catch {
			return 'ไม่ทราบ';
		}
	}

	// Get session badge variant based on status
	function getSessionBadgeVariant(session: SessionInfo) {
		if (isCurrentSession(session.session_id)) {
			return 'default';
		} else if (isExpired(session.expires_at)) {
			return 'destructive';
		} else if (isExpiringSoon(session.expires_at)) {
			return 'outline';
		}
		return 'secondary';
	}

	// Get session status text
	function getSessionStatusText(session: SessionInfo): string {
		if (isCurrentSession(session.session_id)) {
			return 'เซสชันปัจจุบัน';
		} else if (isExpired(session.expires_at)) {
			return 'หมดอายุแล้ว';
		} else if (isExpiringSoon(session.expires_at)) {
			return 'กำลังจะหมดอายุ';
		}
		return 'ใช้งานได้';
	}
</script>

<Card class="w-full">
	<CardHeader>
		<div class="flex items-center justify-between">
			<div>
				<CardTitle class="flex items-center gap-2">
					<IconShield class="h-5 w-5" />
					การจัดการเซสชัน
				</CardTitle>
				<CardDescription>
					จัดการเซสชันการเข้าสู่ระบบของคุณในอุปกรณ์ต่างๆ
				</CardDescription>
			</div>
			<div class="flex gap-2">
				<Button variant="outline" size="sm" onclick={loadSessions} disabled={loading}>
					<IconRefresh class="h-4 w-4 mr-2 {loading ? 'animate-spin' : ''}" />
					รีเฟรช
				</Button>
				<Button 
					variant="outline" 
					size="sm" 
					onclick={extendCurrentSession}
					disabled={extending}
				>
					<IconClock class="h-4 w-4 mr-2" />
					ขยายเซสชัน
				</Button>
			</div>
		</div>
	</CardHeader>

	<CardContent class="space-y-4">
		{#if error}
			<Alert variant="destructive">
				<IconAlertTriangle class="h-4 w-4" />
				<AlertDescription>{error}</AlertDescription>
			</Alert>
		{/if}

		{#if loading}
			<div class="flex items-center justify-center py-8">
				<IconRefresh class="h-6 w-6 animate-spin mr-2" />
				<span>กำลังโหลดเซสชัน...</span>
			</div>
		{:else if sessions.length === 0}
			<div class="text-center py-8 text-gray-500">
				<IconShield class="h-12 w-12 mx-auto mb-4 opacity-50" />
				<p>ไม่พบเซสชันที่ใช้งานอยู่</p>
			</div>
		{:else}
			<div class="space-y-3">
				{#each sessions as session (session.session_id)}
					<Card class="border-l-4 {isCurrentSession(session.session_id) ? 'border-l-blue-500 bg-blue-50/50' : isExpired(session.expires_at) ? 'border-l-red-500' : isExpiringSoon(session.expires_at) ? 'border-l-yellow-500' : 'border-l-gray-300'}">
						<CardContent class="p-4">
							<div class="flex items-start justify-between">
								<div class="flex items-start space-x-3 flex-1">
									<!-- Device Icon -->
									<div class="mt-1">
										<svelte:component 
											this={getDeviceIcon(session.device_info)} 
											class="h-5 w-5 text-gray-600" 
										/>
									</div>

									<!-- Session Details -->
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-2 mb-2">
											<h4 class="font-medium text-sm truncate">
												{getDeviceDisplayName(session.device_info)}
											</h4>
											<Badge variant={getSessionBadgeVariant(session)} class="text-xs">
												{getSessionStatusText(session)}
											</Badge>
										</div>

										<div class="space-y-1 text-sm text-gray-600">
											{#if session.ip_address}
												<div class="flex items-center gap-1">
													<IconMapPin class="h-3 w-3" />
													<span>{session.ip_address}</span>
												</div>
											{/if}

											<div class="flex items-center gap-1">
												<IconUser class="h-3 w-3" />
												<span>เข้าสู่ระบบ: {formatRelativeTime(session.created_at)}</span>
											</div>

											<div class="flex items-center gap-1">
												<IconClock class="h-3 w-3" />
												<span>ใช้งานล่าสุด: {formatRelativeTime(session.last_accessed)}</span>
											</div>

											<div class="flex items-center gap-1">
												<IconCalendar class="h-3 w-3" />
												<span class="{isExpired(session.expires_at) ? 'text-red-600 font-medium' : isExpiringSoon(session.expires_at) ? 'text-yellow-600 font-medium' : ''}">
													หมดอายุ: {formatAbsoluteTime(session.expires_at)}
												</span>
											</div>
										</div>
									</div>
								</div>

								<!-- Actions -->
								<div class="ml-4">
									{#if !isCurrentSession(session.session_id) && !isExpired(session.expires_at)}
										<Dialog>
											<DialogTrigger>
												{#snippet child({ props })}
													<Button 
														{...props}
														variant="outline" 
														size="sm" 
														class="text-red-600 hover:text-red-700 hover:bg-red-50"
														disabled={revoking.has(session.session_id)}
													>
														{#if revoking.has(session.session_id)}
															<IconRefresh class="h-3 w-3 mr-1 animate-spin" />
														{:else}
															<IconTrash class="h-3 w-3 mr-1" />
														{/if}
														ยกเลิก
													</Button>
												{/snippet}
											</DialogTrigger>
											<DialogContent>
												<DialogHeader>
													<DialogTitle>ยืนยันการยกเลิกเซสชัน</DialogTitle>
													<DialogDescription>
														คุณแน่ใจหรือไม่ที่จะยกเลิกเซสชันนี้? การดำเนินการนี้จะทำให้อุปกรณ์นี้ออกจากระบบทันที
													</DialogDescription>
												</DialogHeader>
												<div class="flex justify-end space-x-2">
													<Button variant="outline" size="sm">
														ยกเลิก
													</Button>
													<Button 
														variant="destructive" 
														size="sm"
														onclick={() => revokeSession(session.session_id)}
														disabled={revoking.has(session.session_id)}
													>
														{#if revoking.has(session.session_id)}
															<IconRefresh class="h-3 w-3 mr-1 animate-spin" />
														{/if}
														ยืนยัน
													</Button>
												</div>
											</DialogContent>
										</Dialog>
									{:else if isCurrentSession(session.session_id)}
										<Badge variant="default" class="text-xs">
											เซสชันปัจจุบัน
										</Badge>
									{:else}
										<Badge variant="destructive" class="text-xs">
											หมดอายุแล้ว
										</Badge>
									{/if}
								</div>
							</div>
						</CardContent>
					</Card>
				{/each}
			</div>

			<!-- Session Statistics -->
			<div class="mt-6 pt-4 border-t">
				<div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
					<div>
						<div class="text-2xl font-bold text-blue-600">{sessions.length}</div>
						<div class="text-sm text-gray-600">เซสชันทั้งหมด</div>
					</div>
					<div>
						<div class="text-2xl font-bold text-green-600">
							{sessions.filter(s => !isExpired(s.expires_at)).length}
						</div>
						<div class="text-sm text-gray-600">ใช้งานได้</div>
					</div>
					<div>
						<div class="text-2xl font-bold text-yellow-600">
							{sessions.filter(s => isExpiringSoon(s.expires_at)).length}
						</div>
						<div class="text-sm text-gray-600">กำลังจะหมดอายุ</div>
					</div>
					<div>
						<div class="text-2xl font-bold text-red-600">
							{sessions.filter(s => isExpired(s.expires_at)).length}
						</div>
						<div class="text-sm text-gray-600">หมดอายุแล้ว</div>
					</div>
				</div>
			</div>
		{/if}
	</CardContent>
</Card>