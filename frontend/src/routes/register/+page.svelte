<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { registerSchema } from '$lib/schemas/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import * as Select from '$lib/components/ui/select';
	import { IconLoader, IconEye, IconEyeOff, IconUser, IconMail, IconLock, IconAlertTriangle, IconWifi, IconWifiOff } from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';

	let { data } = $props();

	const form = superForm(data.form, {
		validators: zodClient(registerSchema),
		onResult: ({ result }) => {
			if (result.type === 'failure') {
				toast.error('การสมัครสมาชิกไม่สำเร็จ');
			} else if (result.type === 'redirect') {
				toast.success('สมัครสมาชิกสำเร็จ กรุณาเข้าสู่ระบบ');
			}
		}
	});

	const { form: formData, enhance, errors, submitting } = form;

	let showPassword = $state(false);
	let showConfirmPassword = $state(false);

	// Student registration state variables
	let selectedDepartment = $state('');

	function togglePasswordVisibility() {
		showPassword = !showPassword;
	}

	function toggleConfirmPasswordVisibility() {
		showConfirmPassword = !showConfirmPassword;
	}

	// Department options for student registration
	let departmentOptions = $derived(Array.isArray(data.faculties) ? data.faculties.map(faculty => ({
		value: faculty.id,
		label: faculty.name
	})) : []);

	// Handle department selection
	$effect(() => {
		if (selectedDepartment) {
			$formData.department_id = parseInt(selectedDepartment);
		}
	});
</script>

<svelte:head>
	<title>สมัครสมาชิก - Trackivity</title>
	<meta name="description" content="สมัครสมาชิกสำหรับนักศึกษา" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8 overflow-y-auto">
		<div class="text-center">
			<div class="mx-auto h-16 w-16 bg-green-600 rounded-full flex items-center justify-center mb-4">
				<IconUser class="h-8 w-8 text-white" />
			</div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Trackivity
			</h1>
			<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
				สมัครสมาชิกสำหรับนักศึกษา
			</p>
		</div>

		<Card class="w-full">
			<CardHeader class="space-y-1">
				<CardTitle class="text-2xl text-center">สมัครสมาชิก</CardTitle>
				<CardDescription class="text-center">
					กรุณากรอกข้อมูลเพื่อสร้างบัญชีนักศึกษาใหม่
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<!-- แสดงสถานะการเชื่อมต่อ Backend -->
				{#if !data.backendAvailable}
					<Alert variant="destructive">
						<IconWifiOff class="h-4 w-4" />
						<AlertDescription>
							<div class="space-y-2">
								<p class="font-medium">ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้</p>
								<p class="text-sm">
									{data.backendErrorMessage || 'เซิร์ฟเวอร์ไม่พร้อมใช้งานในขณะนี้'} 
									ระบบจะใช้ข้อมูลคณะแบบออฟไลน์แทน
								</p>
							</div>
						</AlertDescription>
					</Alert>
				{:else if !data.facultiesFromBackend}
					<Alert>
						<IconAlertTriangle class="h-4 w-4" />
						<AlertDescription>
							<div class="space-y-2">
								<p class="font-medium">ใช้ข้อมูลคณะแบบออฟไลน์</p>
								<p class="text-sm">
									ไม่สามารถโหลดข้อมูลคณะจากเซิร์ฟเวอร์ได้ ระบบจะใช้ข้อมูลสำรองแทน
								</p>
							</div>
						</AlertDescription>
					</Alert>
				{:else}
					<Alert>
						<IconWifi class="h-4 w-4" />
						<AlertDescription>
							<p class="text-sm">เชื่อมต่อกับเซิร์ฟเวอร์สำเร็จ - ข้อมูลคณะเป็นปัจจุบัน</p>
						</AlertDescription>
					</Alert>
				{/if}

				<form method="POST" use:enhance class="space-y-4">
					{#if $errors._errors}
						<Alert variant="destructive">
							<IconAlertTriangle class="h-4 w-4" />
							<AlertDescription>
								<div class="space-y-2">
									<p class="font-medium">เกิดข้อผิดพลาดในการสมัครสมาชิก</p>
									<p class="text-sm">{$errors._errors[0]}</p>
									{#if !data.backendAvailable}
										<p class="text-xs opacity-75">
											หมายเหตุ: ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้ กรุณาตรวจสอบการเชื่อมต่ออินเทอร์เน็ต
										</p>
									{/if}
								</div>
							</AlertDescription>
						</Alert>
					{/if}

					<Form.Field {form} name="student_id">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconUser class="h-4 w-4" />
									รหัสนักศึกษา
								</Label>
								<Input
									{...props}
									type="text"
									bind:value={$formData.student_id}
									placeholder="64123456789"
									disabled={$submitting}
									class="w-full"
									maxlength="12"
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="first_name">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconUser class="h-4 w-4" />
									ชื่อจริง
								</Label>
								<Input
									{...props}
									type="text"
									bind:value={$formData.first_name}
									placeholder="ชื่อจริงของคุณ"
									disabled={$submitting}
									class="w-full"
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="last_name">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconUser class="h-4 w-4" />
									นามสกุล
								</Label>
								<Input
									{...props}
									type="text"
									bind:value={$formData.last_name}
									placeholder="นามสกุลของคุณ"
									disabled={$submitting}
									class="w-full"
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="email">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconMail class="h-4 w-4" />
									อีเมล
								</Label>
								<Input
									{...props}
									type="email"
									bind:value={$formData.email}
									placeholder="your@email.com"
									disabled={$submitting}
									class="w-full"
								/>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="password">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconLock class="h-4 w-4" />
									รหัสผ่าน
								</Label>
								<div class="relative">
									<Input
										{...props}
										type={showPassword ? 'text' : 'password'}
										bind:value={$formData.password}
										placeholder="รหัสผ่านของคุณ"
										disabled={$submitting}
										class="w-full pr-10"
									/>
									<button
										type="button"
										onclick={togglePasswordVisibility}
										class="absolute inset-y-0 right-0 pr-3 flex items-center"
										tabindex="-1"
									>
										{#if showPassword}
											<IconEyeOff class="h-4 w-4 text-gray-400" />
										{:else}
											<IconEye class="h-4 w-4 text-gray-400" />
										{/if}
									</button>
								</div>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="confirmPassword">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconLock class="h-4 w-4" />
									ยืนยันรหัสผ่าน
								</Label>
								<div class="relative">
									<Input
										{...props}
										type={showConfirmPassword ? 'text' : 'password'}
										bind:value={$formData.confirmPassword}
										placeholder="ยืนยันรหัสผ่านของคุณ"
										disabled={$submitting}
										class="w-full pr-10"
									/>
									<button
										type="button"
										onclick={toggleConfirmPasswordVisibility}
										class="absolute inset-y-0 right-0 pr-3 flex items-center"
										tabindex="-1"
									>
										{#if showConfirmPassword}
											<IconEyeOff class="h-4 w-4 text-gray-400" />
										{:else}
											<IconEye class="h-4 w-4 text-gray-400" />
										{/if}
									</button>
								</div>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Form.Field {form} name="department_id">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id} class="flex items-center gap-2">
									<IconUser class="h-4 w-4" />
									สาขาวิชา (ไม่บังคับ)
								</Label>
								<Select.Root type="single" bind:value={selectedDepartment} disabled={$submitting}>
									<Select.Trigger class="w-full">
										{departmentOptions.find(opt => opt.value.toString() === selectedDepartment)?.label ?? "เลือกสาขาวิชา"}
									</Select.Trigger>
									<Select.Content>
										{#each departmentOptions as option}
											<Select.Item value={option.value.toString()} label={option.label}>
												{option.label}
											</Select.Item>
										{/each}
									</Select.Content>
								</Select.Root>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<div class="text-xs text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 p-3 rounded-lg">
						<p class="font-medium mb-1">หมายเหตุ:</p>
						<ul class="space-y-1 list-disc list-inside">
							<li>รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร</li>
							<li>ต้องมีตัวพิมพ์เล็ก พิมพ์ใหญ่ และตัวเลข</li>
							<li>หากเลือกแอดมินคณะ จำเป็นต้องระบุคณะ</li>
						</ul>
					</div>

					<Button 
						type="submit" 
						class="w-full" 
						disabled={$submitting}
						variant={!data.backendAvailable ? "secondary" : "default"}
					>
						{#if $submitting}
							<IconLoader class="mr-2 h-4 w-4 animate-spin" />
							กำลังสมัครสมาชิก...
						{:else if !data.backendAvailable}
							<IconWifiOff class="mr-2 h-4 w-4" />
							ลองสมัครสมาชิก (อาจไม่สำเร็จ)
						{:else}
							<IconWifi class="mr-2 h-4 w-4" />
							สมัครสมาชิก
						{/if}
					</Button>
				</form>

				<div class="text-center">
					<p class="text-sm text-gray-600 dark:text-gray-400">
						มีบัญชีแล้ว? 
						<a href="/login" class="font-medium text-primary hover:text-primary/90">
							เข้าสู่ระบบ
						</a>
					</p>
				</div>
			</CardContent>
		</Card>

		<div class="text-center text-xs text-gray-500 dark:text-gray-400">
			<p>© 2025 Admin Management System. All rights reserved.</p>
		</div>
	</div>
</div>

<style>
	:global(body) {
		background-color: rgb(249 250 251);
	}
	:global(.dark body) {
		background-color: rgb(17 24 39);
	}
</style>