<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import * as Select from '$lib/components/ui/select';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Separator } from '$lib/components/ui/separator';
	import { 
		IconArrowLeft,
		IconLoader, 
		IconCalendar, 
		IconClock,
		IconMapPin,
		IconUsers,
		IconAward,
		IconPlus
	} from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { goto } from '$app/navigation';
	import type { ActivityType } from '$lib/types/activity';

	let { data } = $props();

	// Validation schema (same as server)
	const activityCreateSchema = z.object({
		activity_name: z.string().min(1, 'กรุณากรอกชื่อกิจกรรม').max(255, 'ชื่อกิจกรรมต้องไม่เกิน 255 ตัวอักษร'),
		description: z.string().min(1, 'กรุณากรอกรายละเอียดกิจกรรม').max(2000, 'รายละเอียดต้องไม่เกิน 2000 ตัวอักษร'),
		start_date: z.string().min(1, 'กรุณาเลือกวันที่เริ่ม'),
		end_date: z.string().min(1, 'กรุณาเลือกวันที่สิ้นสุด'),
		start_time: z.string().min(1, 'กรุณากรอกเวลาเริ่ม').regex(/^([01]?[0-9]|2[0-3]):[0-5][0-9]$/, 'รูปแบบเวลาไม่ถูกต้อง'),
		end_time: z.string().min(1, 'กรุณากรอกเวลาสิ้นสุด').regex(/^([01]?[0-9]|2[0-3]):[0-5][0-9]$/, 'รูปแบบเวลาไม่ถูกต้อง'),
		activity_type: z.enum(['Academic', 'Sports', 'Cultural', 'Social', 'Other']),
		location: z.string().min(1, 'กรุณากรอกสถานที่').max(500, 'สถานที่ต้องไม่เกิน 500 ตัวอักษร'),
		max_participants: z.number().int().min(1, 'จำนวนผู้เข้าร่วมต้องมากกว่า 0').optional().or(z.literal('')),
		require_score: z.boolean().default(false)
	});

	// Form setup
	const form = superForm(data.form, {
		validators: zodClient(activityCreateSchema),
		onResult: async ({ result }) => {
			if (result.type === 'success') {
				toast.success('สร้างกิจกรรมสำเร็จ');
			} else if (result.type === 'failure') {
				toast.error(result.data?.error || 'เกิดข้อผิดพลาดในการสร้างกิจกรรม');
			}
		}
	});

	const { form: formData, enhance, errors, submitting } = form;

	// Activity type options
	const activityTypeOptions: { value: ActivityType; label: string; description: string }[] = [
		{ value: 'Academic', label: 'วิชาการ', description: 'กิจกรรมทางการศึกษาและการเรียนรู้' },
		{ value: 'Sports', label: 'กีฬา', description: 'กิจกรรมกีฬาและการออกกำลังกาย' },
		{ value: 'Cultural', label: 'วัฒนธรรม', description: 'กิจกรรมด้านศิลปะและวัฒนธรรม' },
		{ value: 'Social', label: 'สังคม', description: 'กิจกรรมเพื่อสังคมและการพัฒนาชุมชน' },
		{ value: 'Other', label: 'อื่นๆ', description: 'กิจกรรมประเภทอื่นๆ' }
	];

	let selectedActivityType = $state(activityTypeOptions.find(opt => opt.value === $formData.activity_type));

	// Helper functions
	function goBack() {
		goto('/admin/activities');
	}

	function formatDate(date: string): string {
		if (!date) return '';
		return new Date(date).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	function getTodayDate(): string {
		return new Date().toISOString().split('T')[0];
	}

	// Reactive validation for dates
	$effect(() => {
		if ($formData.start_date && $formData.end_date) {
			const startDate = new Date($formData.start_date);
			const endDate = new Date($formData.end_date);
			
			if (endDate < startDate) {
				// This will be caught by the schema validation
			}
		}
	});
</script>

<svelte:head>
	<title>สร้างกิจกรรมใหม่ - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<Button variant="ghost" size="sm" onclick={goBack} class="text-gray-600 hover:text-gray-800">
				<IconArrowLeft class="h-4 w-4 mr-2" />
				กลับ
			</Button>
			<div>
				<h1 class="text-4xl font-bold text-gray-900 dark:text-white">
					สร้างกิจกรรมใหม่
				</h1>
				<p class="mt-2 text-lg text-gray-600 dark:text-gray-400">
					กรอกข้อมูลเพื่อสร้างกิจกรรมใหม่ในระบบ
				</p>
			</div>
		</div>
	</div>

	<!-- Main Form -->
	<div class="max-w-4xl">
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-3">
					<IconPlus class="h-6 w-6 text-blue-600" />
					รายละเอียดกิจกรรม
				</CardTitle>
				<CardDescription>
					กรอกข้อมูลทั่วไปของกิจกรรม
				</CardDescription>
			</CardHeader>
			<CardContent>
				<form method="POST" use:enhance class="space-y-6">
					<!-- Error Display -->
					{#if $errors._errors}
						<Alert variant="destructive">
							<AlertDescription>
								{$errors._errors[0]}
							</AlertDescription>
						</Alert>
					{/if}

					<!-- Basic Information -->
					<div class="space-y-6">
						<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
							<!-- Activity Name -->
							<div class="lg:col-span-2">
								<Form.Field {form} name="activity_name">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium">ชื่อกิจกรรม *</Label>
											<Input
												{...props}
												bind:value={$formData.activity_name}
												placeholder="เช่น การบรรยายพิเศษเรื่องเทคโนโลยีใหม่"
												disabled={$submitting}
												class="text-base"
											/>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>

							<!-- Activity Type -->
							<div>
								<Form.Field {form} name="activity_type">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium">ประเภทกิจกรรม *</Label>
											<Select.Root 
												selected={selectedActivityType}
												onSelectedChange={(v) => {
													if (v) {
														selectedActivityType = v;
														$formData.activity_type = v.value;
													}
												}}
												disabled={$submitting}
											>
												<Select.Trigger class="text-base">
													<Select.Value placeholder="เลือกประเภทกิจกรรม" />
												</Select.Trigger>
												<Select.Content>
													{#each activityTypeOptions as option}
														<Select.Item value={option.value} label={option.label}>
															<div class="flex flex-col">
																<span class="font-medium">{option.label}</span>
																<span class="text-sm text-gray-500">{option.description}</span>
															</div>
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>

							<!-- Location -->
							<div>
								<Form.Field {form} name="location">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium flex items-center gap-2">
												<IconMapPin class="h-4 w-4" />
												สถานที่ *
											</Label>
											<Input
												{...props}
												bind:value={$formData.location}
												placeholder="เช่น ห้องประชุมใหญ่ อาคาร A"
												disabled={$submitting}
												class="text-base"
											/>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
						</div>

						<!-- Description -->
						<div>
							<Form.Field {form} name="description">
								<Form.Control>
									{#snippet children({ props })}
										<Label for={props.id} class="text-base font-medium">รายละเอียดกิจกรรม *</Label>
										<Textarea
											{...props}
											bind:value={$formData.description}
											placeholder="อธิบายรายละเอียดของกิจกรรม วัตถุประสงค์ และสิ่งที่ผู้เข้าร่วมจะได้รับ"
											disabled={$submitting}
											rows={4}
											class="text-base"
										/>
									{/snippet}
								</Form.Control>
								<Form.FieldErrors />
							</Form.Field>
						</div>
					</div>

					<Separator />

					<!-- Date and Time -->
					<div class="space-y-6">
						<h3 class="text-lg font-semibold flex items-center gap-2">
							<IconCalendar class="h-5 w-5 text-blue-600" />
							วันที่และเวลา
						</h3>
						
						<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
							<!-- Start Date -->
							<div>
								<Form.Field {form} name="start_date">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium">วันที่เริ่ม *</Label>
											<Input
												{...props}
												type="date"
												bind:value={$formData.start_date}
												min={getTodayDate()}
												disabled={$submitting}
												class="text-base"
											/>
											{#if $formData.start_date}
												<p class="text-sm text-gray-500 mt-1">
													{formatDate($formData.start_date)}
												</p>
											{/if}
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>

							<!-- End Date -->
							<div>
								<Form.Field {form} name="end_date">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium">วันที่สิ้นสุด *</Label>
											<Input
												{...props}
												type="date"
												bind:value={$formData.end_date}
												min={$formData.start_date || getTodayDate()}
												disabled={$submitting}
												class="text-base"
											/>
											{#if $formData.end_date}
												<p class="text-sm text-gray-500 mt-1">
													{formatDate($formData.end_date)}
												</p>
											{/if}
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>

							<!-- Start Time -->
							<div>
								<Form.Field {form} name="start_time">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium flex items-center gap-2">
												<IconClock class="h-4 w-4" />
												เวลาเริ่ม *
											</Label>
											<Input
												{...props}
												type="time"
												bind:value={$formData.start_time}
												disabled={$submitting}
												class="text-base"
											/>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>

							<!-- End Time -->
							<div>
								<Form.Field {form} name="end_time">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium flex items-center gap-2">
												<IconClock class="h-4 w-4" />
												เวลาสิ้นสุด *
											</Label>
											<Input
												{...props}
												type="time"
												bind:value={$formData.end_time}
												disabled={$submitting}
												class="text-base"
											/>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
						</div>
					</div>

					<Separator />

					<!-- Additional Settings -->
					<div class="space-y-6">
						<h3 class="text-lg font-semibold flex items-center gap-2">
							<IconUsers class="h-5 w-5 text-blue-600" />
							การตั้งค่าเพิ่มเติม
						</h3>

						<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
							<!-- Max Participants -->
							<div>
								<Form.Field {form} name="max_participants">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium flex items-center gap-2">
												<IconUsers class="h-4 w-4" />
												จำนวนผู้เข้าร่วมสูงสุด
											</Label>
											<Input
												{...props}
												type="number"
												bind:value={$formData.max_participants}
												placeholder="ไม่จำกัด"
												min="1"
												disabled={$submitting}
												class="text-base"
											/>
											<p class="text-sm text-gray-500 mt-1">
												หากไม่กรอก จะถือว่าไม่จำกัดจำนวน
											</p>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>

							<!-- Require Score -->
							<div class="flex flex-col justify-center">
								<Form.Field {form} name="require_score">
									<Form.Control>
										{#snippet children({ props })}
											<div class="flex items-center space-x-3">
												<Checkbox
													{...props}
													bind:checked={$formData.require_score}
													disabled={$submitting}
												/>
												<div class="grid gap-1.5 leading-none">
													<Label for={props.id} class="text-base font-medium flex items-center gap-2">
														<IconAward class="h-4 w-4" />
														ต้องการคะแนน
													</Label>
													<p class="text-sm text-gray-500">
														กิจกรรมนี้จะให้คะแนนแก่ผู้เข้าร่วม
													</p>
												</div>
											</div>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
						</div>
					</div>

					<!-- Submit Buttons -->
					<div class="flex justify-end gap-4 pt-6">
						<Button type="button" variant="outline" onclick={goBack} disabled={$submitting}>
							ยกเลิก
						</Button>
						<Button type="submit" disabled={$submitting} class="bg-blue-600 hover:bg-blue-700">
							{#if $submitting}
								<IconLoader class="mr-2 h-4 w-4 animate-spin" />
								กำลังสร้าง...
							{:else}
								<IconPlus class="mr-2 h-4 w-4" />
								สร้างกิจกรรม
							{/if}
						</Button>
					</div>
				</form>
			</CardContent>
		</Card>
	</div>
</div>