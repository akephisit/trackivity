<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import { 
		IconPlus, 
		IconCalendarEvent,
		IconMapPin,
		IconUsers,
		IconClock,
		IconEdit,
		IconTrash,
		IconEye,
		IconAward,
		IconRefresh
	} from '@tabler/icons-svelte/icons';
	import { goto, invalidateAll } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';

	import type { Activity } from '$lib/types/activity';

	let { data } = $props();

	// Stats calculation
	let stats = $derived({
		total: data.activities.length,
		academic: data.activities.filter(a => a.activity_type === 'Academic').length,
		sports: data.activities.filter(a => a.activity_type === 'Sports').length,
		cultural: data.activities.filter(a => a.activity_type === 'Cultural').length,
		social: data.activities.filter(a => a.activity_type === 'Social').length,
		other: data.activities.filter(a => a.activity_type === 'Other').length
	});

	function goToCreate() {
		goto('/admin/activities/create');
	}

	function formatDate(dateString: string): string {
		if (!dateString) return '-';
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function formatTime(timeString: string | undefined): string {
		if (!timeString) return '-';
		return timeString;
	}

	function getActivityTypeLabel(type: string): string {
		const labels: Record<string, string> = {
			'Academic': 'วิชาการ',
			'Sports': 'กีฬา',
			'Cultural': 'วัฒนธรรม',
			'Social': 'สังคม',
			'Other': 'อื่นๆ'
		};
		return labels[type] || type;
	}

	function getActivityTypeBadgeVariant(type: string): "default" | "secondary" | "destructive" | "outline" {
		const variants: Record<string, "default" | "secondary" | "destructive" | "outline"> = {
			'Academic': 'default',
			'Sports': 'secondary',
			'Cultural': 'outline',
			'Social': 'default',
			'Other': 'secondary'
		};
		return variants[type] || 'outline';
	}

	function getActivityStatus(activity: Activity): { label: string; variant: "default" | "secondary" | "destructive" | "outline" } {
		// ใช้สถานะจากฐานข้อมูลแทนการคำนวณเอง
		const status = activity.status;
		
		switch (status) {
			case 'draft':
				return { label: 'แบบร่าง', variant: 'outline' };
			case 'published':
				return { label: 'เผยแพร่แล้ว', variant: 'default' };
			case 'ongoing':
				return { label: 'กำลังดำเนินการ', variant: 'default' };
			case 'completed':
				return { label: 'เสร็จสิ้น', variant: 'secondary' };
			case 'cancelled':
				return { label: 'ยกเลิกแล้ว', variant: 'destructive' };
			default:
				// Fallback: คำนวณจากวันที่และเวลาถ้าไม่มีสถานะในฐานข้อมูล
				if (activity.start_date && activity.end_date) {
					const now = new Date();
					const startDate = new Date(activity.start_date);
					const endDate = new Date(activity.end_date);

					if (now < startDate) {
						return { label: 'รอดำเนินการ', variant: 'outline' };
					} else if (now >= startDate && now <= endDate) {
						return { label: 'กำลังดำเนินการ', variant: 'default' };
					} else {
						return { label: 'เสร็จสิ้น', variant: 'secondary' };
					}
				}
				return { label: 'ไม่ระบุ', variant: 'outline' };
		}
	}

	function viewActivity(activityId: string) {
		goto(`/admin/activities/${activityId}`);
	}

	function editActivity(activityId: string) {
		goto(`/admin/activities/${activityId}/edit`);
	}

	let deleteDialogOpen = $state(false);
	let activityToDelete: { id: string; name: string } | null = $state(null);
	let deleting = $state(false);
	let updatingStatuses = $state(false);

	function confirmDelete(activityId: string, name: string | undefined) {
		// Normalize to a non-empty string for the dialog
		const resolvedName = (name && name.trim()) || 'กิจกรรม';
		activityToDelete = { id: activityId, name: resolvedName };
		deleteDialogOpen = true;
	}

	async function performDelete() {
		if (!activityToDelete) return;
		deleting = true;
		try {
			const fd = new FormData();
			fd.append('activityId', activityToDelete.id);
			const res = await fetch('?/delete', { method: 'POST', body: fd });
			const result = await res.json().catch(() => ({}));
			if (res.ok && (result.type === 'success' || result.success === true)) {
				toast.success('ลบกิจกรรมสำเร็จ');
				deleteDialogOpen = false;
				activityToDelete = null;
				// Refresh list by re-running load
				await invalidateAll();
			} else {
				toast.error(result.error || 'ลบกิจกรรมไม่สำเร็จ');
			}
		} catch (e) {
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		} finally {
			deleting = false;
		}
	}

	async function updateActivityStatuses() {
		updatingStatuses = true;
		try {
			const res = await fetch('/api/admin/activities/update-statuses', { 
				method: 'POST',
				headers: { 'Content-Type': 'application/json' }
			});
			const result = await res.json().catch(() => ({}));
			
			if (res.ok && result.status === 'success') {
				toast.success('อัพเดตสถานะกิจกรรมสำเร็จ');
				// Refresh list by re-running load
				await invalidateAll();
			} else {
				toast.error(result.message || 'อัพเดตสถานะกิจกรรมไม่สำเร็จ');
			}
		} catch (e) {
			console.error('Error updating activity statuses:', e);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		} finally {
			updatingStatuses = false;
		}
	}

</script>

<svelte:head>
	<title>จัดการกิจกรรม - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-4xl font-bold text-gray-900 dark:text-white">
				จัดการกิจกรรม
			</h1>
			<p class="mt-3 text-lg text-gray-600 dark:text-gray-400">
				จัดการกิจกรรมทั้งหมดในระบบ รวมถึงการสร้าง แก้ไข และลบกิจกรรม
			</p>
		</div>
		<div class="flex flex-col sm:flex-row gap-3">
			<Button 
				onclick={updateActivityStatuses} 
				disabled={updatingStatuses}
				class="bg-green-600 hover:bg-green-700 text-white px-6 py-3 text-base font-medium"
			>
				<IconRefresh class="h-5 w-5 mr-2 {updatingStatuses ? 'animate-spin' : ''}" />
				{updatingStatuses ? 'กำลังอัพเดต...' : 'อัพเดตสถานะ'}
			</Button>
			{#if data.canCreateActivity}
				<Button onclick={goToCreate} class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 text-base font-medium">
					<IconPlus class="h-5 w-5 mr-2" />
					สร้างกิจกรรมใหม่
				</Button>
			{/if}
		</div>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-6 gap-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">กิจกรรมทั้งหมด</CardTitle>
				<IconCalendarEvent class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats.total}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">วิชาการ</CardTitle>
				<IconCalendarEvent class="h-4 w-4 text-blue-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{stats.academic}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">กีฬา</CardTitle>
				<IconCalendarEvent class="h-4 w-4 text-green-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">{stats.sports}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">วัฒนธรรม</CardTitle>
				<IconCalendarEvent class="h-4 w-4 text-purple-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-purple-600">{stats.cultural}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">สังคม</CardTitle>
				<IconCalendarEvent class="h-4 w-4 text-orange-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-orange-600">{stats.social}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">อื่นๆ</CardTitle>
				<IconCalendarEvent class="h-4 w-4 text-gray-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-gray-600">{stats.other}</div>
			</CardContent>
		</Card>
	</div>

	<!-- Activities Table -->
	<div class="space-y-6">
		{#if data.activities.length === 0}
			<div class="text-center py-16 text-gray-500 dark:text-gray-400">
				<IconCalendarEvent class="h-16 w-16 mx-auto mb-6 opacity-50" />
				<h3 class="text-xl font-semibold mb-2">ยังไม่มีกิจกรรมในระบบ</h3>
				<p class="text-gray-400 mb-6">เริ่มต้นด้วยการสร้างกิจกรรมแรก</p>
				{#if data.canCreateActivity}
					<Button onclick={goToCreate} class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3">
						<IconPlus class="h-5 w-5 mr-2" />
						สร้างกิจกรรมแรก
					</Button>
				{/if}
			</div>
		{:else}
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-3">
						<IconCalendarEvent class="h-6 w-6 text-blue-600" />
						รายการกิจกรรมทั้งหมด
					</CardTitle>
					<CardDescription>
						จัดการกิจกรรมต่างๆ ในระบบ
					</CardDescription>
				</CardHeader>
				<CardContent class="p-0">
					<div class="overflow-hidden">
						<Table.Root>
							<Table.Header>
								<Table.Row class="bg-gray-50 dark:bg-gray-800">
									<Table.Head class="font-semibold">ชื่อกิจกรรม</Table.Head>
									<Table.Head class="font-semibold">ประเภท</Table.Head>
									<Table.Head class="font-semibold">วันที่</Table.Head>
									<Table.Head class="font-semibold">เวลา</Table.Head>
									<Table.Head class="font-semibold">สถานที่</Table.Head>
									<Table.Head class="font-semibold">ผู้เข้าร่วม</Table.Head>
									<Table.Head class="font-semibold">สถานะ</Table.Head>
									<Table.Head class="text-right font-semibold">การดำเนินการ</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each data.activities as activity (activity.id)}
									{@const status = getActivityStatus(activity)}
									<Table.Row class="hover:bg-gray-50/50 dark:hover:bg-gray-800/50">
										<Table.Cell class="font-medium py-4">
											<div class="flex items-center gap-3">
												<div class="w-10 h-10 bg-blue-100 dark:bg-blue-900 rounded-lg flex items-center justify-center">
													<IconCalendarEvent class="h-5 w-5 text-blue-600 dark:text-blue-400" />
												</div>
												<div>
													<div class="font-semibold text-gray-900 dark:text-gray-100">
														{activity.activity_name}
													</div>
													<div class="text-sm text-gray-500 line-clamp-1">
														{activity.description}
													</div>
												</div>
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<Badge variant={getActivityTypeBadgeVariant(activity.activity_type)}>
												{getActivityTypeLabel(activity.activity_type)}
											</Badge>
										</Table.Cell>
										<Table.Cell class="py-4">
											<div class="flex flex-col text-sm">
												<span class="font-medium">{formatDate(activity.start_date)}</span>
												{#if activity.start_date !== activity.end_date}
													<span class="text-gray-500">ถึง {formatDate(activity.end_date)}</span>
												{/if}
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<div class="flex items-center gap-1 text-sm">
												<IconClock class="h-3 w-3" />
												<span>{formatTime(activity.start_time || activity.start_time_only)} - {formatTime(activity.end_time || activity.end_time_only)}</span>
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<div class="flex items-center gap-1 text-sm max-w-xs">
												<IconMapPin class="h-3 w-3 flex-shrink-0" />
												<span class="truncate">{activity.location}</span>
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<div class="flex items-center gap-1 text-sm">
												<IconUsers class="h-3 w-3" />
												<span>{activity.max_participants ? `0/${activity.max_participants}` : 'ไม่จำกัด'}</span>
												{#if activity.require_score}
														<span title="มีคะแนน">
															<IconAward class="h-3 w-3 text-yellow-500 ml-1" />
														</span>
												{/if}
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<Badge variant={status.variant}>
												{status.label}
											</Badge>
										</Table.Cell>
										<Table.Cell class="text-right py-4">
											<div class="flex items-center gap-1 justify-end">
												<Button 
													variant="ghost" 
													size="sm" 
													onclick={() => viewActivity(activity.id)}
													class="text-blue-600 hover:text-blue-700 hover:bg-blue-50"
													title="ดูรายละเอียด"
												>
													<IconEye class="h-4 w-4" />
												</Button>
												{#if data.canCreateActivity}
													<Button 
														variant="ghost" 
														size="sm" 
														onclick={() => editActivity(activity.id)}
														class="text-green-600 hover:text-green-700 hover:bg-green-50"
														title="แก้ไข"
													>
														<IconEdit class="h-4 w-4" />
													</Button>
													<Button
														variant="ghost"
														size="sm"
														onclick={() => confirmDelete(activity.id, activity.activity_name || activity.name || activity.title)}
														class="text-red-600 hover:text-red-700 hover:bg-red-50"
														title="ลบ"
													>
														<IconTrash class="h-4 w-4" />
													</Button>
												{/if}
											</div>
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</div>
				</CardContent>
			</Card>
		{/if}
	</div>
</div>

<!-- Delete Confirmation Dialog -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>ยืนยันการลบกิจกรรม</AlertDialog.Title>
			<AlertDialog.Description>
				{#if activityToDelete}
					คุณแน่ใจหรือไม่ที่จะลบกิจกรรม "{activityToDelete.name}"?
					<br />การดำเนินการนี้ไม่สามารถยกเลิกได้
				{:else}
					กำลังโหลดข้อมูล...
				{/if}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel
				onclick={() => {
					deleteDialogOpen = false;
					activityToDelete = null;
				}}
			>
				ยกเลิก
			</AlertDialog.Cancel>
			<AlertDialog.Action onclick={performDelete} class="bg-red-600 text-white hover:bg-red-700" disabled={deleting}>
				{deleting ? 'กำลังลบ...' : 'ลบกิจกรรม'}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
