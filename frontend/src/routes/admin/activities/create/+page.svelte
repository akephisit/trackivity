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
	import { Separator } from '$lib/components/ui/separator';
	import { 
		IconArrowLeft,
		IconLoader, 
		IconCalendar, 
		IconClock,
		IconMapPin,
		IconUsers,
		IconPlus
	} from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { goto } from '$app/navigation';
	import type { ActivityType } from '$lib/types/activity';

	let { data } = $props();

	// Validation schema (same as server)
	const activityCreateSchema = z.object({
		activity_name: z.string().min(1, 'กรุณากรอกชื่อกิจกรรม').max(255, 'ชื่อกิจกรรมต้องไม่เกิน 255 ตัวอักษร'),
		description: z.string().max(2000, 'รายละเอียดต้องไม่เกิน 2000 ตัวอักษร').optional().or(z.literal('')),
		start_date: z.string().min(1, 'กรุณาเลือกวันที่เริ่ม'),
		end_date: z.string().min(1, 'กรุณาเลือกวันที่สิ้นสุด'),
		start_time: z.string().min(1, 'กรุณากรอกเวลาเริ่ม').regex(/^([01]?[0-9]|2[0-3]):[0-5][0-9]$/, 'รูปแบบเวลาไม่ถูกต้อง'),
		end_time: z.string().min(1, 'กรุณากรอกเวลาสิ้นสุด').regex(/^([01]?[0-9]|2[0-3]):[0-5][0-9]$/, 'รูปแบบเวลาไม่ถูกต้อง'),
		activity_type: z.enum(['Academic', 'Sports', 'Cultural', 'Social', 'Other']),
		location: z.string().min(1, 'กรุณากรอกสถานที่').max(500, 'สถานที่ต้องไม่เกิน 500 ตัวอักษร'),
		max_participants: z.string().optional(),
		hours: z
			.string()
			.min(1, 'กรุณากรอกจำนวนชั่วโมง')
			.regex(/^\d+$/, 'ชั่วโมงต้องเป็นจำนวนเต็ม')
			.refine((v) => parseInt(v) > 0, 'ชั่วโมงต้องมากกว่า 0'),
		organizer: z.string().min(1, 'กรุณากรอกหน่วยงานที่จัดกิจกรรม').max(255, 'ชื่อหน่วยงานต้องไม่เกิน 255 ตัวอักษร'),
		eligible_faculties: z.string().min(1, 'กรุณาเลือกคณะที่สามารถเข้าร่วมได้').refine(value => {
			const faculties = value.split(',').filter(f => f.trim() !== '');
			return faculties.length > 0;
		}, 'กรุณาเลือกอย่างน้อย 1 คณะ'),
		academic_year: z.string().min(1, 'กรุณาเลือกปีการศึกษา')
	});

	// Form setup
	const form = superForm(data.form, {
		validators: zodClient(activityCreateSchema),
		onResult: async ({ result }) => {
			if (result.type === 'success') {
				toast.success('สร้างกิจกรรมสำเร็จ');
				// นำทางไปหน้ารายการโดยไม่ต้องพึ่ง query param
				setTimeout(() => {
					goto('/admin/activities');
				}, 50);
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

	// แก้ไขการ parse ข้อมูลคณะ
	let actualFaculties = [];
	if (data.faculties) {
		if (Array.isArray(data.faculties)) {
			actualFaculties = data.faculties;
		} else if ((data.faculties as any).faculties && Array.isArray((data.faculties as any).faculties)) {
			actualFaculties = (data.faculties as any).faculties;
		}
	}

	const facultyOptions = actualFaculties.map((faculty: any) => ({
		value: faculty.id || faculty.faculty_id,
		label: faculty.name || faculty.faculty_name
	}));

	// Academic year options - generate +/- 2 years from current Buddhist year
	function generateAcademicYearOptions() {
		const currentYear = new Date().getFullYear();
		const currentBuddhistYear = currentYear + 543; // Convert to Buddhist year
		const options = [];
		
		for (let i = -2; i <= 2; i++) {
			const year = (currentBuddhistYear + i).toString();
			options.push({ value: year, label: year });
		}
		
		return options;
	}
	
	const academicYearOptions = generateAcademicYearOptions();

	// Selected values for selects
	let selectedActivityType = $state<{ value: ActivityType; label: string } | undefined>(undefined);
	let selectedFaculties = $state<{ value: string; label: string }[]>([]);
	let selectedAcademicYear = $state<{ value: string; label: string } | undefined>(undefined);



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
												type="single" 
												bind:value={selectedActivityType as any} 
												disabled={$submitting}
												onValueChange={(value) => {
													const newType = value as ActivityType;
													const option = activityTypeOptions.find(opt => opt.value === newType);
													if (option) {
														selectedActivityType = { value: option.value, label: option.label };
														$formData.activity_type = option.value;
													}
												}}
											>
												<Select.Trigger>
													{selectedActivityType?.label ?? "เลือกประเภทกิจกรรม"}
												</Select.Trigger>
												<Select.Content>
													{#each activityTypeOptions as option}
														<Select.Item value={option.value}>
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

							<!-- Academic Year -->
					<div>
						<Form.Field {form} name="academic_year">
							<Form.Control>
								{#snippet children({ props })}
									<Label for={props.id} class="text-base font-medium">ปีการศึกษา *</Label>
									<input type="hidden" name="academic_year" bind:value={$formData.academic_year} />
									<Select.Root 
										type="single" 
										bind:value={selectedAcademicYear as any} 
										disabled={$submitting}
										onValueChange={(value) => {
											if (value) {
												selectedAcademicYear = { value, label: value };
												$formData.academic_year = value;
											}
										}}
									>
										<Select.Trigger>
											{selectedAcademicYear?.label ?? "เลือกปีการศึกษา"}
										</Select.Trigger>
										<Select.Content>
											{#each academicYearOptions as option}
												<Select.Item value={option.value}>
													{option.label}
												</Select.Item>
											{/each}
										</Select.Content>
									</Select.Root>
								{/snippet}
							</Form.Control>
							<Form.FieldErrors />
						</Form.Field>
					</div>

					<!-- Organizer -->
							<div>
								<Form.Field {form} name="organizer">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium">หน่วยงานที่จัดกิจกรรม *</Label>
											<Input
												{...props}
												bind:value={$formData.organizer}
												placeholder="เช่น คณะวิทยาศาสตร์"
												disabled={$submitting}
												class="text-base"
											/>
										{/snippet}
									</Form.Control>
									<Form.FieldErrors />
								</Form.Field>
							</div>
						</div>

						<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
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

							<!-- Eligible Faculties -->
							<div>
								<Form.Field {form} name="eligible_faculties">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium">คณะที่สามารถเข้าร่วมได้ *</Label>
											<input type="hidden" name="eligible_faculties" bind:value={$formData.eligible_faculties} />
											<Select.Root 
												type="multiple" 
												bind:value={selectedFaculties as any} 
												disabled={$submitting}
												onValueChange={(values) => {
													if (values && Array.isArray(values)) {
														selectedFaculties = values.map(value => {
															const option = facultyOptions.find((opt: any) => opt.value === value);
															return option ? { value: option.value, label: option.label } : { value, label: value };
														});
														$formData.eligible_faculties = values.join(',');
														console.log('Updated eligible_faculties:', $formData.eligible_faculties);
													}
												}}
											>
												<Select.Trigger>
													{#if selectedFaculties.length === 0}
														เลือกคณะที่สามารถเข้าร่วมได้
													{:else if selectedFaculties.length === 1}
														{selectedFaculties[0].label}
													{:else}
														เลือกแล้ว {selectedFaculties.length} คณะ
													{/if}
												</Select.Trigger>
												<Select.Content>
													{#each facultyOptions as option}
														<Select.Item value={option.value}>
															<div class="flex items-center gap-2">
																<div class="w-4 h-4 flex items-center justify-center">
																	{#if selectedFaculties.some(f => f.value === option.value)}
																		<div class="w-3 h-3 bg-blue-600 rounded-sm"></div>
																	{:else}
																		<div class="w-3 h-3 border border-gray-300 rounded-sm"></div>
																	{/if}
																</div>
																{option.label}
															</div>
														</Select.Item>
													{/each}
												</Select.Content>
											</Select.Root>
											{#if selectedFaculties.length > 0}
												<div class="flex flex-wrap gap-1 mt-2">
													{#each selectedFaculties as faculty}
														<span class="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-800 text-sm rounded-md">
															{faculty.label}
															<button 
																type="button"
																onclick={() => {
																	selectedFaculties = selectedFaculties.filter(f => f.value !== faculty.value);
																	$formData.eligible_faculties = selectedFaculties.map(f => f.value).join(',');
																}}
																class="text-blue-600 hover:text-blue-800"
															>
																×
															</button>
														</span>
													{/each}
												</div>
											{/if}
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
										<Label for={props.id} class="text-base font-medium">รายละเอียดกิจกรรม</Label>
										<Textarea
											{...props}
											bind:value={$formData.description}
											placeholder="อธิบายรายละเอียดของกิจกรรม วัตถุประสงค์ และสิ่งที่ผู้เข้าร่วมจะได้รับ (ไม่บังคับ)"
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
												type="text"
												inputmode="numeric"
												pattern="[0-9]*"
												bind:value={$formData.max_participants}
												placeholder="ไม่จำกัด"
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

							<!-- Activity Hours -->
							<div>
								<Form.Field {form} name="hours">
									<Form.Control>
										{#snippet children({ props })}
											<Label for={props.id} class="text-base font-medium flex items-center gap-2">
												<IconClock class="h-4 w-4" />
												ชั่วโมงกิจกรรม (ชั่วโมง) *
											</Label>
											<Input
												{...props}
												type="text"
												inputmode="numeric"
												pattern="[0-9]*"
												bind:value={$formData.hours}
												placeholder="เช่น 2"
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
