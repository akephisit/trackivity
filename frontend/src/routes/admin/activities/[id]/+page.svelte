<script lang="ts">
	import type { Activity, Participation, ParticipationStatus, ActivityStatus } from '$lib/types/activity';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { 
		Select, 
		SelectContent, 
		SelectItem, 
		SelectTrigger
	} from '$lib/components/ui/select';
	import { 
		Table, 
		TableBody, 
		TableCell, 
		TableHead, 
		TableHeader, 
		TableRow 
	} from '$lib/components/ui/table';
	import {
		IconClock,
		IconUsers,
		IconMapPin,
		IconArrowLeft,
		IconEdit,
		IconTrash,
		IconUserCheck,
		IconInfoCircle,
		IconBuildingBank,
		IconUser,
		IconSettings,
		IconDownload,
		IconEye,
		IconEyeOff,
		IconCheck,
		IconChartBar,
		IconCategory,
		IconCalendar,
		IconClockHour3,
		IconUserHeart
	} from '@tabler/icons-svelte';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Dialog from '$lib/components/ui/dialog';
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	const { data, form } = $props<{ 
		data: { 
			activity: Activity; 
			participations: Participation[];
			participationStats: any;
			faculties: any[];
			user: any;
		}; 
		form?: any;
	}>();
	
	const { activity, participations, participationStats } = data;

	let showParticipations = $state(true);
	let showStats = $state(true);
	let updatingStatus = $state(false);
	let selectedStatus = $state(activity.status);
	
	// Participant management states
	let editingParticipant: Participation | null = $state(null);
	let participantStatus = $state('');
	let participantNotes = $state('');
	let updatingParticipant = $state(false);
	
	// Delete confirmation states
	let deleteDialogOpen = $state(false);
	let deleteParticipantId = $state('');
	let deleteActivityDialogOpen = $state(false);

	function formatDateTime(dateString: string): string {
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatDateRange(start: string, end: string): string {
		const startDate = new Date(start);
		const endDate = new Date(end);
		
		if (startDate.toDateString() === endDate.toDateString()) {
			return `${startDate.toLocaleDateString('th-TH', {
				year: 'numeric',
				month: 'long',
				day: 'numeric'
			})} ${startDate.toLocaleTimeString('th-TH', {
				hour: '2-digit',
				minute: '2-digit'
			})}-${endDate.toLocaleTimeString('th-TH', {
				hour: '2-digit',
				minute: '2-digit'
			})}`;
		}
		
		return `${formatDateTime(start)} - ${formatDateTime(end)}`;
	}

	function getStatusBadge(status: string): { text: string; variant: 'default' | 'secondary' | 'outline' | 'destructive' } {
		switch (status) {
			case 'draft':
				return { text: 'ร่าง', variant: 'outline' };
			case 'published':
				return { text: 'เผยแพร่แล้ว', variant: 'default' };
			case 'ongoing':
				return { text: 'กำลังดำเนินการ', variant: 'secondary' };
			case 'completed':
				return { text: 'เสร็จสิ้น', variant: 'outline' };
			case 'cancelled':
				return { text: 'ยกเลิก', variant: 'destructive' };
			default:
				return { text: status, variant: 'outline' };
		}
	}

	function getParticipationStatusBadge(status: string): { text: string; variant: 'default' | 'secondary' | 'outline' | 'destructive' } {
		switch (status) {
			case 'registered':
				return { text: 'ลงทะเบียนแล้ว', variant: 'outline' };
			case 'checked_in':
				return { text: 'เช็คอินแล้ว', variant: 'secondary' };
			case 'checked_out':
				return { text: 'เช็คเอาต์แล้ว', variant: 'default' };
			case 'completed':
				return { text: 'เสร็จสิ้น', variant: 'default' };
			default:
				return { text: status, variant: 'outline' };
		}
	}

	function goBack() {
		goto('/admin/activities');
	}

	function goToEdit() {
		goto(`/admin/activities/${activity.id}/edit`);
	}

	function toggleParticipations() {
		showParticipations = !showParticipations;
	}

	function toggleStats() {
		showStats = !showStats;
	}

	// Participant management functions
	function startEditingParticipant(participant: Participation) {
		editingParticipant = participant;
		participantStatus = participant.status;
		participantNotes = participant.notes || '';
	}

	function cancelEditingParticipant() {
		editingParticipant = null;
		participantStatus = '';
		participantNotes = '';
	}

	function confirmDeleteParticipant(participationId: string) {
		deleteParticipantId = participationId;
		deleteDialogOpen = true;
	}

	function confirmDeleteActivity() {
		deleteActivityDialogOpen = true;
	}

	// Export participants data
	function exportParticipants() {
		const csv = [
			['ชื่อ', 'รหัสนักศึกษา', 'อีเมล', 'สาขา', 'สถานะ', 'ลงทะเบียนเมื่อ', 'เช็คอินเมื่อ', 'เช็คเอาต์เมื่อ', 'หมายเหตุ'],
			...participations.map((p: Participation) => [
				p.user_name,
				p.student_id,
				p.email,
				p.department_name || '-',
				getParticipationStatusBadge(p.status).text,
				formatDateTime(p.registered_at),
				p.checked_in_at ? formatDateTime(p.checked_in_at) : '-',
				p.checked_out_at ? formatDateTime(p.checked_out_at) : '-',
				p.notes || '-'
			])
		].map(row => row.map((cell: string) => `"${cell}"`).join(',')).join('\n');

		const blob = new Blob(['\uFEFF' + csv], { type: 'text/csv;charset=utf-8;' });
		const link = document.createElement('a');
		link.href = URL.createObjectURL(blob);
		link.download = `รายชื่อผู้เข้าร่วม-${activity.title}-${new Date().toISOString().split('T')[0]}.csv`;
		link.click();
	}

	const statusOptions: { value: ActivityStatus; label: string }[] = [
		{ value: 'draft', label: 'ร่าง' },
		{ value: 'published', label: 'เผยแพร่แล้ว' },
		{ value: 'ongoing', label: 'กำลังดำเนินการ' },
		{ value: 'completed', label: 'เสร็จสิ้น' },
		{ value: 'cancelled', label: 'ยกเลิก' }
	];

	const participationStatusOptions: { value: ParticipationStatus; label: string }[] = [
		{ value: 'registered', label: 'ลงทะเบียนแล้ว' },
		{ value: 'checked_in', label: 'เช็คอินแล้ว' },
		{ value: 'checked_out', label: 'เช็คเอาต์แล้ว' },
		{ value: 'completed', label: 'เสร็จสิ้น' }
	];

	// Handle form responses
	$effect(() => {
		if (form?.success) {
			toast.success(form.message || 'ดำเนินการสำเร็จ');
			invalidateAll();
		} else if (form?.error) {
			toast.error(form.error);
		}
	});
</script>

<svelte:head>
	<title>{activity.title} - จัดการกิจกรรม (Admin)</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no" />
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center gap-4">
		<Button variant="ghost" size="sm" onclick={goBack}>
			<IconArrowLeft class="size-4 mr-2" />
			กลับ
		</Button>
		<div class="flex-1">
			<h1 class="text-2xl lg:text-3xl font-bold">{activity.title}</h1>
			<p class="text-muted-foreground">จัดการกิจกรรม - Admin Panel</p>
		</div>
		<div class="flex gap-2">
			<Button variant="outline" size="sm" onclick={goToEdit}>
				<IconEdit class="size-4 mr-2" />
				แก้ไข
			</Button>
			<Button variant="outline" size="sm" class="text-red-600 hover:text-red-700" onclick={confirmDeleteActivity}>
				<IconTrash class="size-4 mr-2" />
				ลบ
			</Button>
		</div>
	</div>

	<!-- Quick Stats Cards -->
	{#if showStats}
		<div class="grid gap-4 md:grid-cols-4">
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">ผู้เข้าร่วมทั้งหมด</CardTitle>
					<IconUsers class="h-4 w-4 text-muted-foreground" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold">{participationStats.total}</div>
					{#if activity.max_participants}
						<p class="text-xs text-muted-foreground">
							จาก {activity.max_participants} คน
						</p>
					{/if}
				</CardContent>
			</Card>

			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">ลงทะเบียนแล้ว</CardTitle>
					<IconUserCheck class="h-4 w-4 text-blue-500" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-blue-600">{participationStats.registered}</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">เช็คอินแล้ว</CardTitle>
					<IconCheck class="h-4 w-4 text-green-500" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-green-600">{participationStats.checked_in}</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium">เสร็จสิ้น</CardTitle>
					<IconChartBar class="h-4 w-4 text-purple-500" />
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-purple-600">{participationStats.completed}</div>
				</CardContent>
			</Card>
		</div>
	{/if}

	<!-- Activity Details Card -->
	<Card>
		<CardHeader>
			<div class="flex items-start justify-between gap-4">
				<div class="flex-1">
					<CardTitle class="text-xl">{activity.title}</CardTitle>
					{#if activity.description}
						<p class="text-muted-foreground mt-2">{activity.description}</p>
					{/if}
				</div>
				{#snippet statusBadge()}
					{@const status = getStatusBadge(activity.status)}
					<Badge variant={status.variant}>
						{status.text}
					</Badge>
				{/snippet}
				{@render statusBadge()}
			</div>
		</CardHeader>
		
		<CardContent class="space-y-6">
			<!-- Activity Info Grid -->
			<div class="grid gap-4 md:grid-cols-2">
				<!-- Date and Time -->
				<div class="flex items-start gap-3">
					<IconClock class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
					<div>
						<p class="font-medium">วันที่และเวลา</p>
						<p class="text-sm text-muted-foreground">
							{formatDateRange(activity.start_time, activity.end_time)}
						</p>
					</div>
				</div>

				<!-- Location -->
				<div class="flex items-start gap-3">
					<IconMapPin class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
					<div>
						<p class="font-medium">สถานที่</p>
						<p class="text-sm text-muted-foreground">{activity.location}</p>
					</div>
				</div>

				<!-- Participants -->
				<div class="flex items-start gap-3">
					<IconUsers class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
					<div>
						<p class="font-medium">ผู้เข้าร่วม</p>
						<p class="text-sm text-muted-foreground">
							{activity.current_participants} 
							{#if activity.max_participants}
								/ {activity.max_participants}
							{/if}
							คน
						</p>
					</div>
				</div>

				<!-- Faculty -->
				{#if activity.faculty_name}
					<div class="flex items-start gap-3">
						<IconBuildingBank class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
						<div>
							<p class="font-medium">คณะ</p>
							<p class="text-sm text-muted-foreground">{activity.faculty_name}</p>
						</div>
					</div>
				{/if}

				<!-- Creator -->
				<div class="flex items-start gap-3">
					<IconUser class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
					<div>
						<p class="font-medium">ผู้สร้าง</p>
						<p class="text-sm text-muted-foreground">{activity.created_by_name}</p>
					</div>
				</div>

				<!-- Created/Updated -->
				<div class="flex items-start gap-3">
					<IconInfoCircle class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
					<div>
						<p class="font-medium">สร้างเมื่อ</p>
						<p class="text-sm text-muted-foreground">{formatDateTime(activity.created_at)}</p>
						{#if activity.updated_at !== activity.created_at}
							<p class="text-xs text-muted-foreground">
								อัปเดตล่าสุด: {formatDateTime(activity.updated_at)}
							</p>
						{/if}
					</div>
				</div>
			</div>

			<!-- Extra admin fields -->
			<div class="grid gap-4 md:grid-cols-2">
				{#if activity.activity_type}
					<div class="flex items-start gap-3">
						<IconCategory class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
						<div>
							<p class="font-medium">ประเภทกิจกรรม</p>
							<p class="text-sm text-muted-foreground">{activity.activity_type}</p>
						</div>
					</div>
				{/if}
				{#if activity.academic_year}
					<div class="flex items-start gap-3">
						<IconCalendar class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
						<div>
							<p class="font-medium">ปีการศึกษา</p>
							<p class="text-sm text-muted-foreground">{activity.academic_year}</p>
						</div>
					</div>
				{/if}
				{#if activity.hours}
					<div class="flex items-start gap-3">
						<IconClockHour3 class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
						<div>
							<p class="font-medium">ชั่วโมงกิจกรรม</p>
							<p class="text-sm text-muted-foreground">{activity.hours}</p>
						</div>
					</div>
				{/if}
				{#if activity.organizer}
					<div class="flex items-start gap-3">
						<IconUserHeart class="size-5 mt-0.5 text-muted-foreground flex-shrink-0" />
						<div>
							<p class="font-medium">ผู้จัด</p>
							<p class="text-sm text-muted-foreground">{activity.organizer}</p>
						</div>
					</div>
				{/if}
			</div>

			<Separator />

			<!-- Quick Status Update -->
			<div class="space-y-4">
				<h3 class="text-lg font-semibold flex items-center gap-2">
					<IconSettings class="size-5" />
					จัดการสถานะกิจกรรม
				</h3>
				
				<form 
					method="POST" 
					action="?/updateStatus"
					use:enhance={() => {
						updatingStatus = true;
						return async ({ update }) => {
							updatingStatus = false;
							await update();
						};
					}}
					class="flex items-center gap-4"
				>
					<div class="flex-1">
						<Select type="single" name="status" bind:value={selectedStatus}>
							<SelectTrigger class="w-full">
								{selectedStatus ? statusOptions.find(s => s.value === selectedStatus)?.label || 'เลือกสถานะ' : 'เลือกสถานะ'}
							</SelectTrigger>
							<SelectContent>
								{#each statusOptions as option}
									<SelectItem value={option.value}>{option.label}</SelectItem>
								{/each}
							</SelectContent>
						</Select>
					</div>
					<Button type="submit" disabled={updatingStatus || selectedStatus === activity.status}>
						{updatingStatus ? 'กำลังอัปเดต...' : 'อัปเดตสถานะ'}
					</Button>
				</form>
			</div>
		</CardContent>
	</Card>

	<!-- Participants Management -->
	{#if participations.length > 0}
		<Card>
			<CardHeader>
				<div class="flex items-center justify-between">
					<CardTitle class="flex items-center gap-2">
						<IconUsers class="size-5" />
						ผู้เข้าร่วม ({participations.length} คน)
					</CardTitle>
					<div class="flex gap-2">
						<Button variant="outline" size="sm" onclick={exportParticipants}>
							<IconDownload class="size-4 mr-2" />
							ส่งออก CSV
						</Button>
						<Button variant="outline" size="sm" onclick={toggleStats}>
							{#if showStats}
								<IconEyeOff class="size-4 mr-2" />
							{:else}
								<IconEye class="size-4 mr-2" />
							{/if}
							{showStats ? 'ซ่อน' : 'แสดง'}สถิติ
						</Button>
						<Button variant="outline" size="sm" onclick={toggleParticipations}>
							{#if showParticipations}
								<IconEyeOff class="size-4 mr-2" />
							{:else}
								<IconEye class="size-4 mr-2" />
							{/if}
							{showParticipations ? 'ซ่อน' : 'แสดง'}รายชื่อ
						</Button>
					</div>
				</div>
			</CardHeader>
			
			{#if showParticipations}
				<CardContent class="p-0">
					<div class="overflow-hidden">
						<Table>
							<TableHeader>
								<TableRow class="bg-muted/50">
									<TableHead>ชื่อ</TableHead>
									<TableHead>รหัสนักศึกษา</TableHead>
									<TableHead>สาขา</TableHead>
									<TableHead>สถานะ</TableHead>
									<TableHead>ลงทะเบียนเมื่อ</TableHead>
									<TableHead>เช็คอิน</TableHead>
									<TableHead>เช็คเอาต์</TableHead>
									<TableHead class="text-right">การดำเนินการ</TableHead>
								</TableRow>
							</TableHeader>
							<TableBody>
								{#each participations as participation}
									<TableRow>
										<TableCell class="font-medium">
											<div>
												<div class="font-semibold">{participation.user_name}</div>
												<div class="text-sm text-muted-foreground">{participation.email}</div>
											</div>
										</TableCell>
										<TableCell>{participation.student_id}</TableCell>
										<TableCell>{participation.department_name || '-'}</TableCell>
										<TableCell>
											{@const status = getParticipationStatusBadge(participation.status)}
											<Badge variant={status.variant}>
												{status.text}
											</Badge>
										</TableCell>
										<TableCell class="text-sm">
											{formatDateTime(participation.registered_at)}
										</TableCell>
										<TableCell class="text-sm">
											{participation.checked_in_at ? formatDateTime(participation.checked_in_at) : '-'}
										</TableCell>
										<TableCell class="text-sm">
											{participation.checked_out_at ? formatDateTime(participation.checked_out_at) : '-'}
										</TableCell>
										<TableCell class="text-right">
											<div class="flex items-center gap-1 justify-end">
												<Button 
													variant="ghost" 
													size="sm" 
													onclick={() => startEditingParticipant(participation)}
													class="text-blue-600 hover:text-blue-700"
													title="แก้ไขสถานะ"
												>
													<IconEdit class="h-4 w-4" />
												</Button>
												<Button 
													variant="ghost" 
													size="sm" 
													onclick={() => confirmDeleteParticipant(participation.id)}
													class="text-red-600 hover:text-red-700"
													title="ลบผู้เข้าร่วม"
												>
													<IconTrash class="h-4 w-4" />
												</Button>
											</div>
										</TableCell>
									</TableRow>
								{/each}
							</TableBody>
						</Table>
					</div>
				</CardContent>
			{/if}
		</Card>
	{:else}
		<Card>
			<CardContent class="text-center py-8">
				<IconUsers class="size-12 mx-auto mb-4 text-muted-foreground" />
				<h3 class="text-lg font-semibold mb-2">ยังไม่มีผู้เข้าร่วม</h3>
				<p class="text-muted-foreground">ยังไม่มีผู้ใดลงทะเบียนเข้าร่วมกิจกรรมนี้</p>
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Edit Participant Dialog -->
<Dialog.Root open={!!editingParticipant} onOpenChange={(open) => { if (!open) editingParticipant = null; }}>
	<Dialog.Content class="sm:max-w-[425px]">
		<Dialog.Header>
			<Dialog.Title>แก้ไขสถานะผู้เข้าร่วม</Dialog.Title>
			<Dialog.Description>
				{#if editingParticipant}
					แก้ไขสถานะของ {editingParticipant.user_name}
				{/if}
			</Dialog.Description>
		</Dialog.Header>
		
		{#if editingParticipant}
			<form 
				method="POST" 
				action="?/updateParticipant"
				use:enhance={() => {
					updatingParticipant = true;
					return async ({ update }) => {
						updatingParticipant = false;
						editingParticipant = null;
						await update();
					};
				}}
				class="space-y-4"
			>
				<input type="hidden" name="participationId" value={editingParticipant.id} />
				
				<div class="space-y-2">
					<Label for="participantStatus">สถานะ</Label>
					<Select type="single" name="participantStatus" bind:value={participantStatus}>
						<SelectTrigger>
							{participantStatus ? participationStatusOptions.find(s => s.value === participantStatus)?.label || 'เลือกสถานะ' : 'เลือกสถานะ'}
						</SelectTrigger>
						<SelectContent>
							{#each participationStatusOptions as option}
								<SelectItem value={option.value}>{option.label}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</div>

				<div class="space-y-2">
					<Label for="notes">หมายเหตุ</Label>
					<Textarea
						id="notes"
						name="notes"
						placeholder="หมายเหตุเพิ่มเติม (ไม่บังคับ)"
						bind:value={participantNotes}
						rows={3}
					/>
				</div>

				<Dialog.Footer>
					<Button type="button" variant="outline" onclick={cancelEditingParticipant} disabled={updatingParticipant}>
						ยกเลิก
					</Button>
					<Button type="submit" disabled={updatingParticipant}>
						{updatingParticipant ? 'กำลังบันทึก...' : 'บันทึก'}
					</Button>
				</Dialog.Footer>
			</form>
		{/if}
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Participant Confirmation -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>ยืนยันการลบผู้เข้าร่วม</AlertDialog.Title>
			<AlertDialog.Description>
				คุณแน่ใจหรือไม่ที่จะลบผู้เข้าร่วมคนนี้ออกจากกิจกรรม?
				<br />การดำเนินการนี้ไม่สามารถยกเลิกได้
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => deleteDialogOpen = false}>
				ยกเลิก
			</AlertDialog.Cancel>
			<AlertDialog.Action 
				class="bg-red-600 text-white hover:bg-red-700"
				onclick={async () => {
					const formData = new FormData();
					formData.append('participationId', deleteParticipantId);
					await fetch('?/removeParticipant', {
						method: 'POST',
						body: formData
					});
					deleteDialogOpen = false;
					invalidateAll();
				}}
			>
				ลบผู้เข้าร่วม
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<!-- Delete Activity Confirmation -->
<AlertDialog.Root bind:open={deleteActivityDialogOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>ยืนยันการลบกิจกรรม</AlertDialog.Title>
			<AlertDialog.Description>
				คุณแน่ใจหรือไม่ที่จะลบกิจกรรม "{activity.title}"?
				<br />การดำเนินการนี้จะลบข้อมูลทั้งหมดรวมถึงผู้เข้าร่วม และไม่สามารถยกเลิกได้
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => deleteActivityDialogOpen = false}>
				ยกเลิก
			</AlertDialog.Cancel>
			<AlertDialog.Action 
				class="bg-red-600 text-white hover:bg-red-700"
				onclick={async () => {
					const response = await fetch('?/deleteActivity', {
						method: 'POST'
					});
					if (response.redirected) {
						goto(response.url);
					}
				}}
			>
				ลบกิจกรรม
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<style>
	:global(.line-clamp-1) {
		display: -webkit-box;
		-webkit-line-clamp: 1;
		line-clamp: 1;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
	
	:global(.line-clamp-2) {
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}
</style>
