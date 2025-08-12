<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { loginSchema } from '$lib/schemas/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import {
		IconLoader,
		IconEye,
		IconEyeOff,
		IconUser,
		IconSchool
	} from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let { data } = $props();

	const form = superForm(data.form, {
		validators: zodClient(loginSchema),
		onResult: async ({ result }) => {
			if (result.type === 'failure') {
				toast.error('เข้าสู่ระบบไม่สำเร็จ');
			} else if (result.type === 'success' && result.data?.loginSuccess) {
				toast.success('เข้าสู่ระบบสำเร็จ');
				
				// Sync with client-side auth store after successful server-side login
				console.log('[User Login] Server login successful, refreshing client auth...');
				try {
					// Wait a bit for cookie to be properly set
					await new Promise(resolve => setTimeout(resolve, 100));
					
					// Try direct API call first like admin login does
					console.log('[User Login] Starting user auth check...');
					let user;
					try {
						console.log('[User Login] Calling me() endpoint...');
						const userResponse = await import('$lib/api/client').then(m => m.apiClient.me());
						console.log('[User Login] User response received:', userResponse);
						
						if (userResponse && userResponse.data) {
							user = userResponse.data;
							console.log('[User Login] Extracted user data:', user);
							
							// Update auth store manually
							auth.setUser(user);
							console.log('[User Login] User auth successful via /api/auth/me');
						} else {
							console.log('[User Login] User response format not recognized, trying fallback...');
						}
					} catch (error) {
						console.log('[User Login] Direct API call failed, trying auth.refreshUser:', error);
						user = await auth.refreshUser();
					}
					
					if (user) {
						console.log('[User Login] Client auth synced successfully');
						
						// Verify cookie is properly set
						const cookieCheck = document.cookie.match(/session_id=([^;]+)/);
						console.log('[User Login] Current session cookie:', cookieCheck ? cookieCheck[1] : 'NOT FOUND');
						
						// Add extended delay and testing like admin login
						setTimeout(async () => {
							console.log('[User Login] Navigating to home after delay...');
							// Double-check cookie before navigation
							const finalCookieCheck = document.cookie.match(/session_id=([^;]+)/);
							console.log('[User Login] Session cookie at navigation:', finalCookieCheck ? finalCookieCheck[1] : 'NOT FOUND');
							
							// Test server-side user endpoint manually before navigation
							console.log('[User Login] Testing server-side user auth before navigation...');
							try {
								const testResponse = await fetch('/api/auth/me', {
									credentials: 'include',
									headers: {
										'Cookie': document.cookie
									}
								});
								console.log('[User Login] Manual server test status:', testResponse.status);
								console.log('[User Login] Manual server test headers:', Object.fromEntries(testResponse.headers.entries()));
								
								if (testResponse.ok) {
									const testData = await testResponse.json();
									console.log('[User Login] Manual server test data:', testData);
								} else {
									const testError = await testResponse.text();
									console.log('[User Login] Manual server test error:', testError);
								}
							} catch (error) {
								console.error('[User Login] Manual server test failed:', error);
							}
							
							// Navigate to redirect target
							const redirectTo = result.data.redirectTo || '/';
							console.log('[User Login] Navigating to:', redirectTo);
							setTimeout(() => {
								console.log('[User Login] Final navigation to:', redirectTo);
								goto(redirectTo);
							}, 500);
						}, 2000);
					} else {
						console.log('[User Login] Client auth sync failed');
						// Fallback navigation
						const redirectTo = result.data.redirectTo || '/';
						setTimeout(() => {
							console.log('[User Login] Fallback navigation to:', redirectTo);
							goto(redirectTo);
						}, 500);
					}
				} catch (error) {
					console.error('[User Login] Auth sync error:', error);
					// Fallback navigation
					const redirectTo = result.data.redirectTo || '/';
					goto(redirectTo);
				}
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
	<title>เข้าสู่ระบบ - Trackivity</title>
	<meta name="description" content="เข้าสู่ระบบสำหรับนักเรียน" />
</svelte:head>

<div
	class="flex min-h-screen items-center justify-center bg-gray-50 px-4 py-12 sm:px-6 lg:px-8 dark:bg-gray-900"
>
	<div class="w-full max-w-md space-y-8">
		<div class="text-center">
			<div
				class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-green-600"
			>
				<IconSchool class="h-8 w-8 text-white" />
			</div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">Trackivity</h1>
			<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">เข้าสู่ระบบสำหรับนักเรียน</p>
		</div>

		<Card class="w-full">
			<CardHeader class="space-y-1">
				<CardTitle class="flex items-center justify-center gap-2 text-center text-2xl">
					<IconUser class="h-5 w-5" />
					เข้าสู่ระบบ
				</CardTitle>
				<CardDescription class="text-center">สำหรับนักเรียนและผู้เข้าร่วมกิจกรรม</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<form method="POST" use:enhance class="space-y-4">
					<Form.Field {form} name="student_id">
						<Form.Control>
							{#snippet children({ props })}
								<Label for={props.id}>รหัสนักศึกษา</Label>
								<Input
									{...props}
									type="text"
									bind:value={$formData.student_id}
									placeholder="64123456789"
									disabled={$submitting}
									class="w-full"
									maxlength={12}
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
										class="absolute inset-y-0 right-0 flex items-center pr-3"
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

					{#if $errors.student_id && $errors.student_id.includes('รหัสนักศึกษาหรือรหัสผ่านไม่ถูกต้อง')}
						<Alert variant="destructive">
							<AlertDescription>
								รหัสนักศึกษาหรือรหัสผ่านไม่ถูกต้อง กรุณาตรวจสอบและลองใหม่อีกครั้ง
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

				<div class="space-y-3">
					<div class="relative">
						<div class="absolute inset-0 flex items-center">
							<span class="w-full border-t"></span>
						</div>
						<div class="relative flex justify-center text-xs uppercase">
							<span class="bg-background text-muted-foreground px-2">หรือ</span>
						</div>
					</div>

					<div class="space-y-2 text-center">
						<p class="text-sm text-gray-600 dark:text-gray-400">
							ยังไม่มีบัญชี?
							<a href="/register" class="font-medium text-green-600 hover:text-green-500">
								สมัครสมาชิก
							</a>
						</p>
						<p class="text-sm text-gray-600 dark:text-gray-400">
							ผู้ดูแลระบบ?
							<a href="/admin/login" class="font-medium text-blue-600 hover:text-blue-500">
								เข้าสู่ระบบ Admin
							</a>
						</p>
					</div>
				</div>
			</CardContent>
		</Card>

		<div class="text-center text-xs text-gray-500 dark:text-gray-400">
			<p>© 2025 Trackivity System. All rights reserved.</p>
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
