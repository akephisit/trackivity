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
	import { auth } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let { data } = $props();

	const form = superForm(data.form, {
		validators: zodClient(adminLoginSchema),
		onResult: async ({ result }) => {
			if (result.type === 'failure') {
				toast.error('การเข้าสู่ระบบไม่สำเร็จ');
			} else if (result.type === 'success' && result.data?.loginSuccess) {
				toast.success('เข้าสู่ระบบสำเร็จ');
				
				// Sync with client-side auth store after successful server-side login
				console.log('[Admin Login] Server login successful, refreshing client auth...');
				try {
					// Wait a bit for cookie to be properly set
					await new Promise(resolve => setTimeout(resolve, 100));
					
					// Try admin-specific endpoint first for admin login
					console.log('[Admin Login] Starting admin auth check...');
					let user;
					try {
						console.log('[Admin Login] Calling adminMe()...');
						const adminResponse = await import('$lib/api/client').then(m => m.apiClient.adminMe());
						console.log('[Admin Login] Admin response received:', adminResponse);
						
						// Admin endpoint returns different format: {user: {...}, admin_role: {...}, permissions: [...]}
						console.log('[Admin Login] Checking response format...');
						console.log('[Admin Login] Has adminResponse.user?', !!adminResponse.user);
						console.log('[Admin Login] Has adminResponse.success?', !!adminResponse.success);
						
						if (adminResponse && adminResponse.user) {
							// Extract user data from admin response  
							const userData = adminResponse.user;
							console.log('[Admin Login] Extracted user data:', userData);
							
							if (userData) {
								// Merge admin_role and permissions into user object
								const completeUser = {
									...userData,
									admin_role: adminResponse.admin_role || userData.admin_role,
									permissions: adminResponse.permissions || userData.permissions
								};
								
								console.log('[Admin Login] Complete user object:', completeUser);
								
								// Update auth store manually for admin user
								auth.setUser(completeUser);
								user = completeUser;
								console.log('[Admin Login] Admin auth successful via /api/admin/auth/me');
							}
						} else {
							console.log('[Admin Login] Admin response format not recognized, trying fallback...');
						}
					} catch (error) {
						console.log('[Admin Login] Admin auth failed, trying regular auth:', error);
						user = await auth.refreshUser();
					}
					if (user) {
						console.log('[Admin Login] Client auth synced successfully');
						
						// Verify cookie is properly set
						const cookieCheck = document.cookie.match(/session_id=([^;]+)/);
						console.log('[Admin Login] Current session cookie:', cookieCheck ? cookieCheck[1] : 'NOT FOUND');
						
						// Add even longer delay before navigation to avoid race conditions with server auth check
						setTimeout(async () => {
							console.log('[Admin Login] Navigating to /admin after delay...');
							// Double-check cookie before navigation
							const finalCookieCheck = document.cookie.match(/session_id=([^;]+)/);
							console.log('[Admin Login] Session cookie at navigation:', finalCookieCheck ? finalCookieCheck[1] : 'NOT FOUND');
							
							// Test server-side admin endpoint manually before navigation
							console.log('[Admin Login] Testing server-side admin auth before navigation...');
							try {
								const testResponse = await fetch('/api/admin/auth/me', {
									credentials: 'include',
									headers: {
										'Cookie': document.cookie
									}
								});
								console.log('[Admin Login] Manual server test status:', testResponse.status);
								console.log('[Admin Login] Manual server test headers:', Object.fromEntries(testResponse.headers.entries()));
								
								if (testResponse.ok) {
									const testData = await testResponse.json();
									console.log('[Admin Login] Manual server test data:', testData);
								} else {
									const testError = await testResponse.text();
									console.log('[Admin Login] Manual server test error:', testError);
								}
							} catch (error) {
								console.error('[Admin Login] Manual server test failed:', error);
							}
							
							// Try full page navigation instead of client-side goto
							console.log('[Admin Login] Using window.location for full page navigation...');
							console.log('[Admin Login] All cookies before navigation:', document.cookie);
							
							// Check cookie details
							const allCookies = document.cookie.split(';').map(c => c.trim());
							console.log('[Admin Login] Cookie details:', allCookies);
							
							// Add a small delay to see if cookie disappears
							setTimeout(() => {
								console.log('[Admin Login] Cookies after 500ms delay:', document.cookie);
								
								// Instead of automatic navigation, let user manually test
								console.log('[Admin Login] Ready for navigation - try visiting /admin manually in new tab');
								console.log('[Admin Login] Or wait 3 seconds for automatic navigation...');
								
								setTimeout(() => {
									console.log('[Admin Login] Automatic navigation starting...');
									window.location.href = '/admin?debug=login-redirect';
								}, 3000);
							}, 500);
						}, 2000);
					} else {
						console.log('[Admin Login] Client auth sync failed');
						// Fallback - try direct navigation since server login was successful
						setTimeout(() => {
							console.log('[Admin Login] Fallback navigation to /admin...');
							goto('/admin');
						}, 500);
					}
				} catch (error) {
					console.error('[Admin Login] Auth sync error:', error);
					// Fallback - try direct navigation since server login was successful
					goto('/admin');
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