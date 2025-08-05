<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { authService, isLoading, authError } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { IconLoader, IconEye, IconEyeOff, IconAlertCircle } from '@tabler/icons-svelte/icons';

	const dispatch = createEventDispatcher();

	// Form state
	let email = '';
	let password = '';
	let rememberMe = false;
	let showPassword = false;
	let validationErrors: Record<string, string> = {};

	// Extract redirect URL from query params
	let redirectUrl = '/dashboard';

	onMount(() => {
		const redirect = $page.url.searchParams.get('redirect');
		if (redirect) {
			redirectUrl = decodeURIComponent(redirect);
		}
	});

	// Form validation
	function validateForm(): boolean {
		validationErrors = {};

		if (!email.trim()) {
			validationErrors.email = 'Email is required';
		} else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
			validationErrors.email = 'Please enter a valid email address';
		}

		if (!password.trim()) {
			validationErrors.password = 'Password is required';
		} else if (password.length < 6) {
			validationErrors.password = 'Password must be at least 6 characters';
		}

		return Object.keys(validationErrors).length === 0;
	}

	// Handle form submission
	async function handleSubmit(event: Event) {
		event.preventDefault();
		
		if (!validateForm()) {
			return;
		}

		try {
			const result = await authService.login({
				email: email.trim(),
				password,
				remember_me: rememberMe
			});

			if (result.success) {
				// Dispatch success event
				dispatch('success', { message: result.message });
				
				// Redirect to intended page
				await goto(redirectUrl);
			} else {
				// Handle login failure
				dispatch('error', { message: result.message });
			}
		} catch (error) {
			console.error('Login error:', error);
			dispatch('error', { 
				message: error instanceof Error ? error.message : 'An unexpected error occurred' 
			});
		}
	}

	// Handle input changes to clear validation errors
	function clearFieldError(field: string) {
		if (validationErrors[field]) {
			delete validationErrors[field];
			validationErrors = { ...validationErrors };
		}
	}

	// Toggle password visibility
	function togglePasswordVisibility() {
		showPassword = !showPassword;
	}

	// Handle Enter key on checkbox
	function handleCheckboxKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			rememberMe = !rememberMe;
		}
	}
</script>

<Card class="w-full max-w-md mx-auto">
	<CardHeader class="space-y-1">
		<CardTitle class="text-2xl font-bold text-center">เข้าสู่ระบบ</CardTitle>
		<CardDescription class="text-center">
			กรุณาป้อนอีเมลและรหัสผ่านเพื่อเข้าสู่ระบบ
		</CardDescription>
	</CardHeader>
	
	<CardContent>
		{#if $authError}
			<Alert variant="destructive" class="mb-4">
				<IconAlertCircle class="h-4 w-4" />
				<AlertDescription>{$authError}</AlertDescription>
			</Alert>
		{/if}

		<form onsubmit={handleSubmit} class="space-y-4">
			<!-- Email Field -->
			<div class="space-y-2">
				<Label for="email">อีเมล</Label>
				<Input
					id="email"
					type="email"
					placeholder="your.email@university.ac.th"
					bind:value={email}
					oninput={() => clearFieldError('email')}
					class={validationErrors.email ? 'border-red-500' : ''}
					disabled={$isLoading}
					autocomplete="username"
					required
				/>
				{#if validationErrors.email}
					<p class="text-sm text-red-500">{validationErrors.email}</p>
				{/if}
			</div>

			<!-- Password Field -->
			<div class="space-y-2">
				<Label for="password">รหัสผ่าน</Label>
				<div class="relative">
					<Input
						id="password"
						type={showPassword ? 'text' : 'password'}
						placeholder="กรุณาป้อนรหัสผ่าน"
						bind:value={password}
						oninput={() => clearFieldError('password')}
						class={validationErrors.password ? 'border-red-500 pr-10' : 'pr-10'}
						disabled={$isLoading}
						autocomplete="current-password"
						required
					/>
					<button
						type="button"
						class="absolute inset-y-0 right-0 pr-3 flex items-center"
						onclick={togglePasswordVisibility}
						disabled={$isLoading}
						aria-label={showPassword ? 'ซ่อนรหัสผ่าน' : 'แสดงรหัสผ่าน'}
					>
						{#if showPassword}
							<IconEyeOff class="h-4 w-4 text-gray-400" />
						{:else}
							<IconEye class="h-4 w-4 text-gray-400" />
						{/if}
					</button>
				</div>
				{#if validationErrors.password}
					<p class="text-sm text-red-500">{validationErrors.password}</p>
				{/if}
			</div>

			<!-- Remember Me Checkbox -->
			<div class="flex items-center space-x-2">
				<Checkbox
					id="remember"
					bind:checked={rememberMe}
					onkeydown={handleCheckboxKeydown}
					disabled={$isLoading}
				/>
				<Label 
					for="remember" 
					class="text-sm font-normal cursor-pointer select-none"
				>
					จดจำการเข้าสู่ระบบ (30 วัน)
				</Label>
			</div>

			<!-- Submit Button -->
			<Button 
				type="submit" 
				class="w-full" 
				disabled={$isLoading}
			>
				{#if $isLoading}
					<IconLoader class="mr-2 h-4 w-4 animate-spin" />
					กำลังเข้าสู่ระบบ...
				{:else}
					เข้าสู่ระบบ
				{/if}
			</Button>
		</form>

		<!-- Additional Links -->
		<div class="mt-6 text-center text-sm">
			<p class="text-gray-600">
				ยังไม่มีบัญชี? 
				<a 
					href="/register" 
					class="font-medium text-blue-600 hover:text-blue-500 transition-colors"
				>
					สมัครสมาชิก
				</a>
			</p>
			
			<div class="mt-2">
				<a 
					href="/forgot-password" 
					class="text-sm text-gray-600 hover:text-gray-900 transition-colors"
				>
					ลืมรหัสผ่าน?
				</a>
			</div>
		</div>

		<!-- Session Info -->
		{#if redirectUrl !== '/dashboard'}
			<div class="mt-4 p-3 bg-blue-50 rounded-md">
				<p class="text-sm text-blue-800">
					หลังจากเข้าสู่ระบบแล้ว คุณจะถูกนำไปยัง: 
					<span class="font-medium">{redirectUrl}</span>
				</p>
			</div>
		{/if}
	</CardContent>
</Card>

<style>
	/* Custom styles for better UX */
	:global(.login-form input:focus) {
		ring-width: 2px;
		ring-color: theme('colors.blue.500');
		border-color: theme('colors.blue.500');
	}
	
	:global(.login-form input.error) {
		border-color: theme('colors.red.500');
		ring-color: theme('colors.red.500');
	}
</style>