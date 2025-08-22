<script lang="ts">
	import { currentUser } from '$lib/stores/auth';
	import { useQRCode } from '$lib/qr/client';
	import type { Activity, ActivityParticipation } from '$lib/types';

	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import QRCodeGenerator from '$lib/components/qr/QRCodeGenerator.svelte';

	import {
		IconQrcode,
		IconCalendarEvent,
		IconHistory,
		IconTrendingUp,
		IconUsers,
		IconClock,
		IconMapPin,
		IconChevronRight,
		IconAlertCircle
	} from '@tabler/icons-svelte';

	// Component state
	const {
		recentActivities = [] as Activity[],
		participationHistory = [] as ActivityParticipation[],
		stats = {
			totalParticipations: 0,
			thisMonthParticipations: 0,
			upcomingActivities: 0
		}
	} = $props<{
		recentActivities: Activity[];
		participationHistory: ActivityParticipation[];
		stats: {
			totalParticipations: number;
			thisMonthParticipations: number;
			upcomingActivities: number;
		};
	}>();
	let loading = {
		activities: false,
		history: false,
		stats: false
	};
	let error: string | null = null;

	// QR Code integration
	const { qrCode, status: qrStatus } = useQRCode();

	// Data is passed from server; QR related UI stays client-side

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function getActivityBadgeVariant(type: string): 'default' | 'secondary' | 'outline' {
		switch (type) {
			case 'lecture':
				return 'default';
			case 'workshop':
				return 'secondary';
			case 'exam':
				return 'outline';
			default:
				return 'outline';
		}
	}

	function getActivityTypeText(type: string): string {
		const types: Record<string, string> = {
			lecture: 'บรรยาย',
			workshop: 'ปฏิบัติการ',
			seminar: 'สัมมนา',
			exam: 'สอบ',
			meeting: 'ประชุม',
			event: 'งาน'
		};
		return types[type] || type;
	}
</script>

<div class="space-y-6">
	<!-- Welcome Section -->
	<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
		<!-- Welcome Card -->
		<Card class="md:col-span-2 lg:col-span-2">
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<IconUsers class="size-5" />
					ยินดีต้อนรับ, {$currentUser?.first_name}!
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="space-y-2">
					<p class="text-muted-foreground text-sm">
						รหัสนักศึกษา: <span class="font-medium">{$currentUser?.student_id}</span>
					</p>
					<p class="text-muted-foreground text-sm">
						อีเมล: <span class="font-medium">{$currentUser?.email}</span>
					</p>
					<div class="flex gap-2 pt-2">
						<Button size="sm" href="/qr">
							<IconQrcode class="mr-2 size-4" />
							ดู QR Code
						</Button>
						<Button size="sm" variant="outline" href="/activities">
							<IconCalendarEvent class="mr-2 size-4" />
							กิจกรรมของฉัน
						</Button>
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- QR Status Card -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center justify-between">
					<span class="text-base">QR Code Status</span>
					<Badge variant={$qrStatus === 'ready' ? 'default' : 'secondary'}>
						{$qrStatus === 'ready'
							? 'พร้อมใช้'
							: $qrStatus === 'generating'
								? 'กำลังสร้าง'
								: 'ไม่พร้อม'}
					</Badge>
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="space-y-3">
					{#if $qrCode && $qrStatus === 'ready'}
						<div class="space-y-1 text-sm">
							<p class="text-muted-foreground">สร้างเมื่อ:</p>
							<p class="text-xs">{formatDate($qrCode.created_at)}</p>
						</div>
						<div class="space-y-1 text-sm">
							<p class="text-muted-foreground">หมดอายุ:</p>
							<p class="text-xs">{formatDate($qrCode.expires_at)}</p>
						</div>
					{:else}
						<p class="text-muted-foreground text-sm">QR Code ยังไม่พร้อมใช้งาน</p>
					{/if}

					<Button size="sm" href="/qr" class="w-full">
						<IconQrcode class="mr-2 size-4" />
						ไปที่ QR Code
					</Button>
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Statistics Section -->
	<div class="grid gap-6 md:grid-cols-3">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">การเข้าร่วมทั้งหมด</CardTitle>
				<IconTrendingUp class="text-muted-foreground size-4" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">
					{loading.stats ? '--' : stats.totalParticipations}
				</div>
				<p class="text-muted-foreground text-xs">กิจกรรมที่เข้าร่วม</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">เดือนนี้</CardTitle>
				<IconCalendarEvent class="text-muted-foreground size-4" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">
					{loading.stats ? '--' : stats.thisMonthParticipations}
				</div>
				<p class="text-muted-foreground text-xs">กิจกรรมในเดือนนี้</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">กิจกรรมที่กำลังจะมา</CardTitle>
				<IconClock class="text-muted-foreground size-4" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">
					{loading.stats ? '--' : stats.upcomingActivities}
				</div>
				<p class="text-muted-foreground text-xs">กิจกรรมใน 7 วันข้างหน้า</p>
			</CardContent>
		</Card>
	</div>

	<!-- Recent Activities and History -->
	<div class="grid gap-6 lg:grid-cols-2">
		<!-- Recent Activities -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center justify-between">
					<span class="flex items-center gap-2">
						<IconCalendarEvent class="size-5" />
						กิจกรรมล่าสุด
					</span>
					<Button size="sm" variant="outline" href="/activities">
						ดูทั้งหมด
						<IconChevronRight class="ml-1 size-4" />
					</Button>
				</CardTitle>
			</CardHeader>
			<CardContent>
				{#if error}
					<Alert variant="destructive">
						<IconAlertCircle class="size-4" />
						<AlertDescription>{error}</AlertDescription>
					</Alert>
				{:else if loading.activities}
					<div class="space-y-3">
						{#each Array(3) as _}
							<div class="space-y-2">
								<Skeleton class="h-4 w-3/4" />
								<Skeleton class="h-3 w-1/2" />
								<Skeleton class="h-3 w-full" />
							</div>
						{/each}
					</div>
				{:else if recentActivities.length === 0}
					<div class="text-muted-foreground py-6 text-center">
						<IconCalendarEvent class="mx-auto mb-2 size-8 opacity-50" />
						<p>ยังไม่มีกิจกรรม</p>
					</div>
				{:else}
					<div class="space-y-4">
						{#each recentActivities as activity}
							<div class="space-y-2 rounded-lg border p-3">
								<div class="flex items-start justify-between">
									<h4 class="text-sm font-medium">{activity.name}</h4>
									<Badge variant={getActivityBadgeVariant(activity.activity_type)}>
										{getActivityTypeText(activity.activity_type)}
									</Badge>
								</div>

								{#if activity.description}
									<p class="text-muted-foreground line-clamp-2 text-xs">
										{activity.description}
									</p>
								{/if}

								<div class="text-muted-foreground flex items-center gap-4 text-xs">
									<span class="flex items-center gap-1">
										<IconClock class="size-3" />
										{formatDate(activity.start_date)}
									</span>
									{#if activity.max_participants}
										<span class="flex items-center gap-1">
											<IconUsers class="size-3" />
											{activity.current_participants}/{activity.max_participants}
										</span>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Participation History -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center justify-between">
					<span class="flex items-center gap-2">
						<IconHistory class="size-5" />
						ประวัติการเข้าร่วม
					</span>
					<Button size="sm" variant="outline" href="/activities?tab=history">
						ดูทั้งหมด
						<IconChevronRight class="ml-1 size-4" />
					</Button>
				</CardTitle>
			</CardHeader>
			<CardContent>
				{#if loading.history}
					<div class="space-y-3">
						{#each Array(3) as _}
							<div class="space-y-2">
								<Skeleton class="h-4 w-3/4" />
								<Skeleton class="h-3 w-1/2" />
							</div>
						{/each}
					</div>
				{:else if participationHistory.length === 0}
					<div class="text-muted-foreground py-6 text-center">
						<IconHistory class="mx-auto mb-2 size-8 opacity-50" />
						<p>ยังไม่มีประวัติการเข้าร่วม</p>
						<p class="mt-1 text-xs">เมื่อคุณเข้าร่วมกิจกรรมแล้ว ประวัติจะแสดงที่นี่</p>
					</div>
				{:else}
					<div class="space-y-3">
						{#each participationHistory as participation}
							<div class="space-y-2 rounded-lg border p-3">
								<h4 class="text-sm font-medium">{participation.activity?.name}</h4>
								<div class="text-muted-foreground flex items-center gap-4 text-xs">
									<span class="flex items-center gap-1">
										<IconClock class="size-3" />
										{formatDate(participation.participated_at)}
									</span>
									{#if participation.qr_scan_location}
										<span class="flex items-center gap-1">
											<IconMapPin class="size-3" />
											{participation.qr_scan_location}
										</span>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</CardContent>
		</Card>
	</div>

	<!-- Quick Actions -->
	<Card>
		<CardHeader>
			<CardTitle>การดำเนินการด่วน</CardTitle>
		</CardHeader>
		<CardContent>
			<div class="grid gap-3 md:grid-cols-3">
				<Button href="/qr" class="justify-start">
					<IconQrcode class="mr-2 size-4" />
					ดู QR Code ของฉัน
				</Button>
				<Button href="/activities" variant="outline" class="justify-start">
					<IconCalendarEvent class="mr-2 size-4" />
					ดูกิจกรรมทั้งหมด
				</Button>
				<Button href="/profile" variant="outline" class="justify-start">
					<IconUsers class="mr-2 size-4" />
					แก้ไขโปรไฟล์
				</Button>
			</div>
		</CardContent>
	</Card>
</div>
