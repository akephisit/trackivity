<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { loginSchema } from '$lib/schemas/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import { IconLoader, IconEye, IconEyeOff } from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';

	let { data } = $props();

	const form = superForm(data.form, {
		validators: zodClient(loginSchema),
		onResult: ({ result }) => {
			if (result.type === 'failure') {
				toast.error('เข้าสู่ระบบไม่สำเร็จ');
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
	<title>เข้าสู่ระบบ - Admin Panel</title>
	<meta name="description" content="เข้าสู่ระบบสำหรับผู้ดูแลระบบ" />
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4 sm:px-6 lg:px-8">
	<div class="max-w-md w-full space-y-8">
		<div class="text-center">
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				Admin Panel
			</h1>
			<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">
				เข้าสู่ระบบสำหรับผู้ดูแลระบบ
			</p>
		</div>

		<Card class="w-full">
			<CardHeader class="space-y-1">
				<CardTitle class="text-2xl text-center">เข้าสู่ระบบ</CardTitle>
				<CardDescription class="text-center">
					กรุณาใส่อีเมลและรหัสผ่านของคุณ
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<form method="POST" use:enhance class="space-y-4">
					<Form.Field {form} name="email">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id}>อีเมล</Label>
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

					{#if $errors.email && $errors.email.includes('อีเมลหรือรหัสผ่านไม่ถูกต้อง')}
						<Alert variant="destructive">
							<AlertDescription>
								อีเมลหรือรหัสผ่านไม่ถูกต้อง กรุณาตรวจสอบและลองใหม่อีกครั้ง
							</AlertDescription>
						</Alert>
					{/if}

					<Button type="submit" class="w-full" disabled={$submitting}>
						{#if $submitting}
							<IconLoader class="mr-2 h-4 w-4 animate-spin" />
							กำลังเข้าสู่ระบบ...
						{:else}
							เข้าสู่ระบบ
						{/if}
					</Button>
				</form>

				<div class="text-center">
					<p class="text-sm text-gray-600 dark:text-gray-400">
						ยังไม่มีบัญชี? 
						<a href="/register" class="font-medium text-primary hover:text-primary/90">
							สมัครสมาชิก
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