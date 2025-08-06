<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { adminLoginSchema } from '$lib/schemas/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { IconLoader, IconEye, IconEyeOff, IconShield, IconAlertTriangle, IconKey } from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';

	let { data } = $props();

	const form = superForm(data.form, {
		validators: zodClient(adminLoginSchema),
		onResult: ({ result }) => {
			if (result.type === 'failure') {
				toast.error('การเข้าสู่ระบบไม่สำเร็จ');
			} else if (result.type === 'redirect') {
				toast.success('เข้าสู่ระบบสำเร็จ');
			}
		}
	});

	const { form: formData, enhance, errors, submitting } = form;

	let showPassword = $state(false);

	function togglePasswordVisibility() {
		showPassword = !showPassword;
	}
</script>

<svelte:head>
	<title>Admin Login - Trackivity</title>
	<meta name="description" content="Admin login portal for Trackivity system" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-blue-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div class="text-center">
			<div class="mx-auto h-16 w-16 bg-blue-600 rounded-full flex items-center justify-center mb-4">
				<IconShield class="h-8 w-8 text-white" />
			</div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Admin Portal
			</h1>
			<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
				เข้าสู่ระบบจัดการสำหรับผู้ดูแลระบบ
			</p>
		</div>

		<Card class="w-full">
			<CardHeader class="space-y-1">
				<CardTitle class="text-2xl text-center flex items-center justify-center gap-2">
					<IconKey class="h-5 w-5" />
					Admin Login
				</CardTitle>
				<CardDescription class="text-center">
					สำหรับผู้ดูแลระบบเท่านั้น
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<form method="POST" use:enhance class="space-y-4">
					{#if $errors._errors}
						<Alert variant="destructive">
							<IconAlertTriangle class="h-4 w-4" />
							<AlertDescription>
								<div class="space-y-2">
									<p class="font-medium">เกิดข้อผิดพลาดในการเข้าสู่ระบบ</p>
									<p class="text-sm">{$errors._errors[0]}</p>
								</div>
							</AlertDescription>
						</Alert>
					{/if}

					<Form.Field {form} name="email">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id}>อีเมล</Label>
								<Input
									{...props}
									type="email"
									bind:value={$formData.email}
									placeholder="admin@example.com"
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
								<Label for={props.id}>รหัสผ่าน</Label>
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

					<Form.Field {form} name="remember_me">
						<Form.Control>
							{#snippet children({ props })}
								<div class="flex items-center space-x-2">
									<Checkbox
										{...props}
										bind:checked={$formData.remember_me}
										disabled={$submitting}
									/>
									<Label for={props.id} class="text-sm">
										จดจำการเข้าสู่ระบบ (30 วัน)
									</Label>
								</div>
							{/snippet}
						</Form.Control>
						<Form.FieldErrors />
					</Form.Field>

					<Button 
						type="submit" 
						class="w-full bg-blue-600 hover:bg-blue-700" 
						disabled={$submitting}
					>
						{#if $submitting}
							<IconLoader class="mr-2 h-4 w-4 animate-spin" />
							กำลังเข้าสู่ระบบ...
						{:else}
							<IconShield class="mr-2 h-4 w-4" />
							เข้าสู่ระบบ Admin
						{/if}
					</Button>
				</form>

				<div class="space-y-3">
					<div class="relative">
						<div class="absolute inset-0 flex items-center">
							<span class="w-full border-t"></span>
						</div>
						<div class="relative flex justify-center text-xs uppercase">
							<span class="bg-background px-2 text-muted-foreground">หรือ</span>
						</div>
					</div>
					
					<div class="text-center">
						<p class="text-sm text-gray-600 dark:text-gray-400">
							นักเรียน? 
							<a href="/login" class="font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400">
								เข้าสู่ระบบนักเรียน
							</a>
						</p>
					</div>
				</div>

				<!-- Default Admin Info (for development) -->
				{#if data.isDevelopment}
					<Alert>
						<IconAlertTriangle class="h-4 w-4" />
						<AlertDescription>
							<div class="space-y-2">
								<p class="font-medium text-sm">Development Mode - Default Admin:</p>
								<p class="text-xs font-mono">Email: admin@trackivity.local</p>
								<p class="text-xs font-mono">Password: admin123!</p>
								<p class="text-xs text-orange-600">⚠️ Change password after first login!</p>
							</div>
						</AlertDescription>
					</Alert>
				{/if}
			</CardContent>
		</Card>

		<div class="text-center text-xs text-gray-500 dark:text-gray-400">
			<p>© 2025 Trackivity Admin System. All rights reserved.</p>
		</div>
	</div>
</div>

<style>
	:global(body) {
		background: linear-gradient(135deg, #f0f4f8 0%, #e2e8f0 100%);
	}
	:global(.dark body) {
		background: linear-gradient(135deg, #1a202c 0%, #2d3748 100%);
	}
</style>