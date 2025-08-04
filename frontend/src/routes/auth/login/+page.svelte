<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { apiClient, type LoginRequest } from '$lib/api/client';
	import { authStore } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let form: LoginRequest = {
		student_id: '',
		password: ''
	};
	let loading = false;
	let error = '';

	onMount(() => {
		// If already authenticated, redirect to dashboard
		if ($authStore.isAuthenticated) {
			goto('/dashboard');
		}
	});

	async function handleSubmit() {
		if (!form.student_id || !form.password) {
			error = 'กรุณากรอกข้อมูลให้ครบถ้วน';
			return;
		}

		loading = true;
		error = '';

		try {
			const response = await apiClient.login(form);
			authStore.login(response.user);
			goto('/dashboard');
		} catch (err: any) {
			error = err.message || 'เกิดข้อผิดพลาดในการเข้าสู่ระบบ';
		} finally {
			loading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 p-4">
	<Card.Root class="w-full max-w-md">
		<Card.Header class="text-center">
			<Card.Title class="text-2xl font-bold">เข้าสู่ระบบ</Card.Title>
			<Card.Description>
				เข้าสู่ระบบ Trackivity ด้วยรหัสนิสิตและรหัสผ่าน
			</Card.Description>
		</Card.Header>
		
		<Card.Content class="space-y-4">
			<form on:submit|preventDefault={handleSubmit} class="space-y-4">
				<div class="space-y-2">
					<Label for="student_id">รหัสนิสิต</Label>
					<Input
						id="student_id"
						type="text"
						placeholder="กรอกรหัสนิสิต"
						bind:value={form.student_id}
						disabled={loading}
						required
					/>
				</div>
				
				<div class="space-y-2">
					<Label for="password">รหัสผ่าน</Label>
					<Input
						id="password"
						type="password"
						placeholder="กรอกรหัสผ่าน"
						bind:value={form.password}
						disabled={loading}
						required
					/>
				</div>

				{#if error}
					<div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
						{error}
					</div>
				{/if}

				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'กำลังเข้าสู่ระบบ...' : 'เข้าสู่ระบบ'}
				</Button>
			</form>
		</Card.Content>
		
		<Card.Footer class="text-center">
			<p class="text-sm text-muted-foreground">
				ยังไม่มีบัญชี? 
				<a href="/auth/register" class="text-primary hover:underline">
					สมัครสมาชิก
				</a>
			</p>
		</Card.Footer>
	</Card.Root>
</div>