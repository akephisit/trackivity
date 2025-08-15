<script lang="ts">
	import { onMount } from 'svelte';
	import { currentUser } from '$lib/stores/auth';
	import { useQRCode } from '$lib/qr/client';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
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
			setTimeout(() => { copied = false; }, 2000);
		} catch (err) {
			toast.error('ไม่สามารถคัดลอกได้');
		}
	}

	async function handleRefreshQR() {
		refreshing = true;
		try {
			await refreshQR();
			toast.success('รีเฟรช QR Code แล้ว');
		} catch (err) {
			toast.error('ไม่สามารถรีเฟรช QR Code ได้');
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
		const interval = setInterval(() => {
			// Trigger reactivity for time remaining
			if ($qrCode) {
				qrCode.set($qrCode);
			}
		}, 60000);

		return () => clearInterval(interval);
	});
</script>

<svelte:head>
	<title>QR Code - Trackivity Student</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no" />
</svelte:head>

<div class="space-y-6 p-4 sm:p-6">
	<!-- Header -->
	<div class="text-center space-y-2">
		<h1 class="text-2xl lg:text-3xl font-bold">QR Code ของฉัน</h1>
		<p class="text-muted-foreground">
			ใช้ QR Code นี้ในการเข้าร่วมกิจกรรมต่างๆ
		</p>
	</div>

	<!-- User Info Card -->
	{#if $currentUser}
		<Card>
			<CardHeader>
				<CardTitle class="text-lg flex items-center gap-2">
					<IconShieldCheck class="size-5" />
					ข้อมูลการระบุตัวตน
				</CardTitle>
			</CardHeader>
			<CardContent class="space-y-3">
				<div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
					<div>
						<span class="text-muted-foreground">ชื่อ:</span>
						<span class="font-medium ml-2">
							{$currentUser.first_name} {$currentUser.last_name}
						</span>
					</div>
					<div>
						<span class="text-muted-foreground">รหัสนักศึกษา:</span>
						<span class="font-mono font-medium ml-2">
							{$currentUser.student_id}
						</span>
					</div>
					<div class="sm:col-span-2">
						<span class="text-muted-foreground">อีเมล:</span>
						<span class="font-medium ml-2">{$currentUser.email}</span>
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
						<statusInfo.icon class="size-3 mr-1" />
						{statusInfo.text}
					</Badge>
				</CardTitle>
			</CardHeader>
			<CardContent class="space-y-4">
				{#if $qrCode && $qrStatus === 'ready'}
					<div class="grid grid-cols-1 sm:grid-cols-2 gap-4 text-sm">
						<div>
							<span class="text-muted-foreground">สร้างเมื่อ:</span>
							<p class="font-mono text-xs mt-1">
								{formatDate($qrCode.created_at)}
							</p>
						</div>
						<div>
							<span class="text-muted-foreground">หมดอายุ:</span>
							<p class="font-mono text-xs mt-1">
								{formatDate($qrCode.expires_at)}
							</p>
						</div>
						<div class="sm:col-span-2">
							<span class="text-muted-foreground">เวลาที่เหลือ:</span>
							<p class="font-medium text-primary mt-1">
								{formatTimeRemaining($qrCode.expires_at)}
							</p>
						</div>
					</div>

					<div class="flex flex-col sm:flex-row gap-2">
						<Button 
							variant="outline" 
							size="sm"
							onclick={copyQRData}
							disabled={copied}
							class="flex-1 sm:flex-none"
						>
							{#if copied}
								<IconCheck class="size-4 mr-2" />
								คัดลอกแล้ว
							{:else}
								<IconCopy class="size-4 mr-2" />
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
							<IconRefresh class={`size-4 mr-2 ${refreshing ? 'animate-spin' : ''}`} />
							รีเฟรช
						</Button>
					</div>
				{:else if $qrStatus === 'generating'}
					<div class="flex items-center justify-center py-6">
						<div class="text-center space-y-2">
							<IconClock class="size-8 mx-auto animate-pulse text-muted-foreground" />
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
						<IconRefresh class={`size-4 mr-2 ${refreshing ? 'animate-spin' : ''}`} />
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
		<div class="text-center text-xs text-muted-foreground space-y-1">
			<p>ID: <span class="font-mono">{$qrCode.id}</span></p>
			<p class="text-muted-foreground/70">
				แสดง QR Code นี้ให้เจ้าหน้าที่สแกนเพื่อเข้าร่วมกิจกรรม
			</p>
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
			<ol class="list-decimal list-inside space-y-2 text-sm text-muted-foreground">
				<li>แสดง QR Code นี้ให้เจ้าหน้าที่ที่กิจกรรม</li>
				<li>เจ้าหน้าที่จะสแกน QR Code เพื่อบันทึกการเข้าร่วม</li>
				<li>QR Code จะหมดอายุและสร้างใหม่อัตโนมัติเพื่อความปลอดภัย</li>
				<li>คุณสามารถใช้ QR Code เดียวกันสำหรับกิจกรรมหลายๆ กิจกรรม</li>
				<li>หาก QR Code หมดอายุ กรุณากด "รีเฟรช" เพื่อสร้างใหม่</li>
			</ol>
		</CardContent>
	</Card>

	<!-- Tips for Mobile -->
	<Card class="lg:hidden border-primary/20 bg-primary/5">
		<CardHeader>
			<CardTitle class="text-sm flex items-center gap-2 text-primary">
				<IconInfoCircle class="size-4" />
				เคล็ดลับสำหรับมือถือ
			</CardTitle>
		</CardHeader>
		<CardContent class="text-xs space-y-1 text-muted-foreground">
			<p>• เพิ่มความสว่างของหน้าจอให้เต็มที่เมื่อแสดง QR Code</p>
			<p>• ถือโทรศัพท์ให้มั่นคงเมื่อเจ้าหน้าที่กำลังสแกน</p>
			<p>• สามารถจับภาพหน้าจอ QR Code เก็บไว้ได้</p>
		</CardContent>
	</Card>
</div>