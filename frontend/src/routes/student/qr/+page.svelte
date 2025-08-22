<script lang="ts">
	import { onMount } from 'svelte';
	import { currentUser } from '$lib/stores/auth';
	import { useQRCode } from '$lib/qr/client';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import QRCodeGenerator from '$lib/components/qr/QRCodeGenerator.svelte';
	import {
		IconQrcode,
		IconRefresh,
		IconCopy,
		IconCheck,
		IconAlertCircle,
		IconShieldCheck,
		IconClock,
		IconInfoCircle
	} from '@tabler/icons-svelte';
	import { toast } from 'svelte-sonner';

	const { qrCode, status: qrStatus, generate } = useQRCode();

	// State for initial loading
	let initialLoadComplete = $state(false);

	async function refreshQR() {
		await generate();
	}

	let copied = $state(false);
	let refreshing = $state(false);

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		});
	}

	function formatTimeRemaining(expiresAt: string): string {
		const now = new Date();
		const expiry = new Date(expiresAt);
		const diff = expiry.getTime() - now.getTime();

		if (diff <= 0) return 'หมดอายุแล้ว';

		const minutes = Math.floor(diff / (1000 * 60));
		const hours = Math.floor(minutes / 60);
		const days = Math.floor(hours / 24);

		if (days > 0) return `เหลือ ${days} วัน`;
		if (hours > 0) return `เหลือ ${hours} ชั่วโมง`;
		return `เหลือ ${minutes} นาที`;
	}

	async function copyQRData() {
		if (!$qrCode) return;

		try {
			await navigator.clipboard.writeText($qrCode.id);
			copied = true;
			toast.success('คัดลอก QR Code ID แล้ว');
			setTimeout(() => {
				copied = false;
			}, 2000);
		} catch (err) {
			toast.error('ไม่สามารถคัดลอกได้');
		}
	}

	async function handleRefreshQR() {
		refreshing = true;
		try {
			console.log('[QR Page] Manual refresh requested');
			await refreshQR();
			toast.success('รีเฟรช QR Code แล้ว');
		} catch (err) {
			console.error('[QR Page] Manual refresh failed:', err);
			toast.error(
				'ไม่สามารถรีเฟรช QR Code ได้: ' +
					(err instanceof Error ? err.message : 'เกิดข้อผิดพลาดไม่ทราบสาเหตุ')
			);
		} finally {
			refreshing = false;
		}
	}

	function getStatusBadge(status: string) {
		switch (status) {
			case 'ready':
				return { variant: 'default' as const, text: 'พร้อมใช้', icon: IconShieldCheck };
			case 'generating':
				return { variant: 'secondary' as const, text: 'กำลังสร้าง', icon: IconClock };
			case 'expired':
				return { variant: 'destructive' as const, text: 'หมดอายุ', icon: IconAlertCircle };
			default:
				return { variant: 'outline' as const, text: 'ไม่พร้อม', icon: IconAlertCircle };
		}
	}

	// Auto-refresh every minute to update time remaining
	onMount(() => {
		// Generate QR code when page loads if user is available
		if ($currentUser && !$qrCode) {
			console.log('[QR Page] User available, generating QR code on mount');
			generate().catch((error) => {
				console.error('[QR Page] Failed to generate QR on mount:', error);
			});
		}
		initialLoadComplete = true;

		const interval = setInterval(() => {
			// Trigger reactivity for time remaining
			if ($qrCode) {
				qrCode.set($qrCode);
			}
		}, 60000);

		return () => clearInterval(interval);
	});

	// Watch for user authentication changes using $effect
	let lastUserCheck = $state('');
	$effect(() => {
		const userKey = $currentUser?.user_id || '';
		if (
			userKey &&
			userKey !== lastUserCheck &&
			initialLoadComplete &&
			!$qrCode &&
			$qrStatus === 'idle'
		) {
			lastUserCheck = userKey;
			console.log('[QR Page] User authenticated, generating QR code');
			generate().catch((error) => {
				console.error('[QR Page] Failed to generate QR after user auth:', error);
			});
		}
	});
</script>

<svelte:head>
	<title>QR Code - Trackivity Student</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no" />
</svelte:head>

<div class="space-y-6 p-4 sm:p-6">
	<!-- Header -->
	<div class="space-y-2 text-center">
		<h1 class="text-2xl font-bold lg:text-3xl">QR Code ของฉัน</h1>
		<p class="text-muted-foreground">ใช้ QR Code นี้ในการเข้าร่วมกิจกรรมต่างๆ</p>
	</div>

	<!-- User Info Card -->
	{#if $currentUser}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2 text-lg">
					<IconShieldCheck class="size-5" />
					ข้อมูลการระบุตัวตน
				</CardTitle>
			</CardHeader>
			<CardContent class="space-y-3">
				<div class="grid grid-cols-1 gap-4 text-sm sm:grid-cols-2">
					<div>
						<span class="text-muted-foreground">ชื่อ:</span>
						<span class="ml-2 font-medium">
							{$currentUser.first_name}
							{$currentUser.last_name}
						</span>
					</div>
					<div>
						<span class="text-muted-foreground">รหัสนักศึกษา:</span>
						<span class="ml-2 font-medium">
							{$currentUser.student_id}
						</span>
					</div>
					<div class="sm:col-span-2">
						<span class="text-muted-foreground">อีเมล:</span>
						<span class="ml-2 font-medium">{$currentUser.email}</span>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- QR Code Status -->
	{#if $qrStatus}
		{@const statusInfo = getStatusBadge($qrStatus)}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center justify-between">
					<span class="flex items-center gap-2">
						<IconQrcode class="size-5" />
						สถานะ QR Code
					</span>
					<Badge variant={statusInfo.variant}>
						<statusInfo.icon class="mr-1 size-3" />
						{statusInfo.text}
					</Badge>
				</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				{#if $qrCode && $qrStatus === 'ready'}
					<div class="grid grid-cols-1 gap-4 text-sm sm:grid-cols-2">
						<div>
							<span class="text-muted-foreground">สร้างเมื่อ:</span>
							<p class="mt-1 text-xs">
								{formatDate($qrCode.created_at)}
							</p>
						</div>
						<div>
							<span class="text-muted-foreground">หมดอายุ:</span>
							<p class="mt-1 text-xs">
								{formatDate($qrCode.expires_at)}
							</p>
						</div>
						<div class="sm:col-span-2">
							<span class="text-muted-foreground">เวลาที่เหลือ:</span>
							<p class="text-primary mt-1 font-medium">
								{formatTimeRemaining($qrCode.expires_at)}
							</p>
						</div>
					</div>

					<div class="flex flex-col gap-2 sm:flex-row">
						<Button
							variant="outline"
							size="sm"
							onclick={copyQRData}
							disabled={copied}
							class="flex-1 sm:flex-none"
						>
							{#if copied}
								<IconCheck class="mr-2 size-4" />
								คัดลอกแล้ว
							{:else}
								<IconCopy class="mr-2 size-4" />
								คัดลอก ID
							{/if}
						</Button>
						<Button
							variant="outline"
							size="sm"
							onclick={handleRefreshQR}
							disabled={refreshing}
							class="flex-1 sm:flex-none"
						>
							<IconRefresh class={`mr-2 size-4 ${refreshing ? 'animate-spin' : ''}`} />
							รีเฟรช
						</Button>
					</div>
				{:else if $qrStatus === 'generating'}
					<div class="flex items-center justify-center py-6">
						<div class="space-y-2 text-center">
							<IconClock class="text-muted-foreground mx-auto size-8 animate-pulse" />
							<p class="text-muted-foreground">กำลังสร้าง QR Code...</p>
						</div>
					</div>
				{:else}
					<Alert>
						<IconAlertCircle class="size-4" />
						<AlertDescription>
							QR Code ไม่พร้อมใช้งาน กรุณารอสักครู่หรือลองรีเฟรชหน้า
						</AlertDescription>
					</Alert>
					<Button onclick={handleRefreshQR} disabled={refreshing} class="w-full sm:w-auto">
						<IconRefresh class={`mr-2 size-4 ${refreshing ? 'animate-spin' : ''}`} />
						ลองอีกครั้ง
					</Button>
				{/if}
			</CardContent>
		</Card>
	{/if}

	<!-- QR Code Display -->
	{#if $qrCode && $qrStatus === 'ready'}
		<div class="flex justify-center">
			<QRCodeGenerator size="large" showStatus={false} />
		</div>
		<div class="text-muted-foreground space-y-1 text-center text-xs">
			<p>ID: <span>{$qrCode.id}</span></p>
			<p class="text-muted-foreground/70">แสดง QR Code นี้ให้เจ้าหน้าที่สแกนเพื่อเข้าร่วมกิจกรรม</p>
		</div>
	{/if}

	<!-- Instructions -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-2">
				<IconInfoCircle class="size-5" />
				วิธีการใช้งาน
			</CardTitle>
		</CardHeader>
		<CardContent>
			<ol class="text-muted-foreground list-inside list-decimal space-y-2 text-sm">
				<li>แสดง QR Code นี้ให้เจ้าหน้าที่ที่กิจกรรม</li>
				<li>เจ้าหน้าที่จะสแกน QR Code เพื่อบันทึกการเข้าร่วม</li>
				<li>QR Code จะหมดอายุและสร้างใหม่อัตโนมัติเพื่อความปลอดภัย</li>
				<li>คุณสามารถใช้ QR Code เดียวกันสำหรับกิจกรรมหลายๆ กิจกรรม</li>
				<li>หาก QR Code หมดอายุ กรุณากด "รีเฟรช" เพื่อสร้างใหม่</li>
			</ol>
		</CardContent>
	</Card>

	<!-- Tips for Mobile -->
	<Card class="border-primary/20 bg-primary/5 lg:hidden">
		<CardHeader>
			<CardTitle class="text-primary flex items-center gap-2 text-sm">
				<IconInfoCircle class="size-4" />
				เคล็ดลับสำหรับมือถือ
			</CardTitle>
		</CardHeader>
		<CardContent class="text-muted-foreground space-y-1 text-xs">
			<p>• เพิ่มความสว่างของหน้าจอให้เต็มที่เมื่อแสดง QR Code</p>
			<p>• ถือโทรศัพท์ให้มั่นคงเมื่อเจ้าหน้าที่กำลังสแกน</p>
			<p>• สามารถจับภาพหน้าจอ QR Code เก็บไว้ได้</p>
		</CardContent>
	</Card>
</div>
