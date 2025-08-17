<script lang="ts">
	import type { Activity, ActivityStatus } from '$lib/types/activity';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { 
		Select, 
		SelectContent, 
		SelectItem, 
		SelectTrigger
	} from '$lib/components/ui/select';
	import {
		IconArrowLeft,
		IconDeviceFloppy,
		IconAlertCircle,
		IconInfoCircle
	} from '@tabler/icons-svelte';
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';

	const { data, form } = $props<{ 
		data: { activity: Activity; faculties: any[] }; 
		form?: { error?: string; formData?: any; } 
	}>();
	
	const { activity, faculties } = data;

	let submitting = $state(false);
	let selectedStatus = $state(activity.status);
	let selectedFaculty = $state(activity.faculty_id || '');

	// Format datetime for input fields
	function formatDateTimeForInput(dateString: string): string {
		const date = new Date(dateString);
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		const hours = String(date.getHours()).padStart(2, '0');
		const minutes = String(date.getMinutes()).padStart(2, '0');
		
		return `${year}-${month}-${day}T${hours}:${minutes}`;
	}

	function goBack() {
		goto(`/student/activities/${activity.id}`);
	}

	function goToList() {
		goto('/student/activities');
	}

	const statusOptions: { value: ActivityStatus; label: string }[] = [
		{ value: 'draft', label: 'ร่าง' },
		{ value: 'published', label: 'เผยแพร่แล้ว' },
		{ value: 'ongoing', label: 'กำลังดำเนินการ' },
		{ value: 'completed', label: 'เสร็จสิ้น' },
		{ value: 'cancelled', label: 'ยกเลิก' }
	];
</script>

<svelte:head>
	<title>แก้ไข{activity.title} - Trackivity Student</title>
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
			<h1 class="text-2xl lg:text-3xl font-bold">แก้ไขกิจกรรม</h1>
			<p class="text-muted-foreground">{activity.title}</p>
		</div>
	</div>

	<!-- Error Alert -->
	{#if form?.error}
		<Alert variant="destructive">
			<IconAlertCircle class="size-4" />
			<AlertDescription>{form.error}</AlertDescription>
		</Alert>
	{/if}

	<!-- Edit Form -->
	<Card>
		<CardHeader>
			<CardTitle>รายละเอียดกิจกรรม</CardTitle>
		</CardHeader>
		
		<CardContent>
			<form 
				method="POST" 
				action="?/update"
				use:enhance={() => {
					submitting = true;
					return async ({ update }) => {
						submitting = false;
						await update();
					};
				}}
				class="space-y-6"
			>
				<!-- Title -->
				<div class="space-y-2">
					<Label for="title">ชื่อกิจกรรม *</Label>
					<Input
						id="title"
						name="title"
						type="text"
						required
						placeholder="ชื่อกิจกรรม"
						value={form?.formData?.title || activity.title}
					/>
				</div>

				<!-- Description -->
				<div class="space-y-2">
					<Label for="description">คำอธิบาย</Label>
					<Textarea
						id="description"
						name="description"
						placeholder="คำอธิบายเกี่ยวกับกิจกรรม"
						value={form?.formData?.description || activity.description}
						rows={4}
					/>
				</div>

				<!-- Location -->
				<div class="space-y-2">
					<Label for="location">สถานที่ *</Label>
					<Input
						id="location"
						name="location"
						type="text"
						required
						placeholder="สถานที่จัดกิจกรรม"
						value={form?.formData?.location || activity.location}
					/>
				</div>

				<!-- Date and Time -->
				<div class="grid gap-4 md:grid-cols-2">
					<div class="space-y-2">
						<Label for="start_time">วันที่และเวลาเริ่ม *</Label>
						<Input
							id="start_time"
							name="start_time"
							type="datetime-local"
							required
							value={form?.formData?.start_time || formatDateTimeForInput(activity.start_time)}
						/>
					</div>

					<div class="space-y-2">
						<Label for="end_time">วันที่และเวลาสิ้นสุด *</Label>
						<Input
							id="end_time"
							name="end_time"
							type="datetime-local"
							required
							value={form?.formData?.end_time || formatDateTimeForInput(activity.end_time)}
						/>
					</div>
				</div>

				<!-- Max Participants -->
				<div class="space-y-2">
					<Label for="max_participants">จำนวนผู้เข้าร่วมสูงสุด</Label>
					<Input
						id="max_participants"
						name="max_participants"
						type="number"
						min="1"
						placeholder="ไม่จำกัด"
						value={form?.formData?.max_participants || activity.max_participants || ''}
					/>
					<p class="text-sm text-muted-foreground">
						เว้นว่างหากไม่ต้องการจำกัดจำนวนผู้เข้าร่วม
					</p>
				</div>

				<!-- Status -->
				<div class="space-y-2">
					<Label for="status">สถานะ</Label>
					<Select type="single" name="status" bind:value={selectedStatus}>
						<SelectTrigger>
							{selectedStatus ? statusOptions.find(opt => opt.value === selectedStatus)?.label : 'เลือกสถานะ'}
						</SelectTrigger>
						<SelectContent>
							{#each statusOptions as option}
								<SelectItem value={option.value}>{option.label}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</div>

				<!-- Faculty -->
				{#if faculties.length > 0}
					<div class="space-y-2">
						<Label for="faculty_id">คณะ</Label>
						<Select type="single" name="faculty_id" bind:value={selectedFaculty}>
							<SelectTrigger>
								{selectedFaculty ? faculties.find((f: any) => f.id === selectedFaculty)?.name : 'เลือกคณะ (ไม่บังคับ)'}
							</SelectTrigger>
							<SelectContent>
								<SelectItem value="">ไม่ระบุคณะ</SelectItem>
								{#each faculties as faculty}
									<SelectItem value={faculty.id}>{faculty.name}</SelectItem>
								{/each}
							</SelectContent>
						</Select>
					</div>
				{/if}

				<!-- Info Alert -->
				<Alert>
					<IconInfoCircle class="size-4" />
					<AlertDescription>
						การแก้ไขกิจกรรมที่มีผู้เข้าร่วมแล้วอาจส่งผลกระทบต่อผู้ที่ลงทะเบียนไว้ 
						กรุณาตรวจสอบข้อมูลให้ถูกต้องก่อนบันทึก
					</AlertDescription>
				</Alert>

				<!-- Action Buttons -->
				<div class="flex flex-col sm:flex-row gap-4 pt-4">
					<Button 
						type="submit" 
						disabled={submitting}
						class="flex-1 sm:flex-none"
					>
						<IconDeviceFloppy class="size-4 mr-2" />
						{submitting ? 'กำลังบันทึก...' : 'บันทึกการเปลี่ยนแปลง'}
					</Button>
					
					<Button 
						type="button" 
						variant="outline" 
						onclick={goBack}
						disabled={submitting}
					>
						ยกเลิก
					</Button>
					
					<Button 
						type="button" 
						variant="ghost" 
						onclick={goToList}
						disabled={submitting}
					>
						กลับสู่รายการกิจกรรม
					</Button>
				</div>
			</form>
		</CardContent>
	</Card>
</div>

<style>
	:global(input[type="datetime-local"]) {
		font-size: 0.875rem; /* text-sm */
	}
</style>