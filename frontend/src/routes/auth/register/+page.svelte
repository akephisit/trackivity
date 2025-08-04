<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Label } from "$lib/components/ui/label/index.js";
	import { apiClient, type RegisterRequest } from '$lib/api/client';
	import { authStore } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	let form: RegisterRequest = {
		student_id: '',
		email: '',
		password: '',
		first_name: '',
		last_name: ''
	};
	let confirmPassword = '';
	let loading = false;
	let error = '';
	let success = '';

	onMount(() => {
		// If already authenticated, redirect to dashboard
		if ($authStore.isAuthenticated) {
			goto('/dashboard');
		}
	});

	async function handleSubmit() {
		// Validation
		if (!form.student_id || !form.email || !form.password || !form.first_name || !form.last_name) {
			error = 'กรุณากรอกข้อมูลให้ครบถ้วน';
			return;
		}

		if (form.password !== confirmPassword) {
			error = 'รหัสผ่านและยืนยันรหัสผ่านไม่ตรงกัน';
			return;
		}

		if (form.password.length < 6) {
			error = 'รหัสผ่านต้องมีความยาวอย่างน้อย 6 ตัวอักษร';
			return;
		}

		loading = true;
		error = '';
		success = '';

		try {
			const response = await apiClient.register(form);
			success = response.message || 'สมัครสมาชิกสำเร็จ! กำลังเข้าสู่ระบบ...';
			
			// Auto login after successful registration
			setTimeout(async () => {
				try {
					const loginResponse = await apiClient.login({
						student_id: form.student_id,
						password: form.password
					});
					authStore.login(loginResponse.user);
					goto('/dashboard');
				} catch (loginError) {
					goto('/auth/login');
				}
			}, 2000);
		} catch (err: any) {
			error = err.message || 'เกิดข้อผิดพลาดในการสมัครสมาชิก';
		} finally {
			loading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 p-4">
	<Card.Root class="w-full max-w-md">
		<Card.Header class="text-center">
			<Card.Title class="text-2xl font-bold">สมัครสมาชิก</Card.Title>
			<Card.Description>
				สร้างบัญชี Trackivity เพื่อเข้าร่วมกิจกรรมมหาวิทยาลัย
			</Card.Description>
		</Card.Header>
		
		<Card.Content class="space-y-4">
			<form on:submit|preventDefault={handleSubmit} class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<Label for="first_name">ชื่อ</Label>
						<Input
							id="first_name"
							type="text"
							placeholder="ชื่อ"
							bind:value={form.first_name}
							disabled={loading}
							required
						/>
					</div>
					<div class="space-y-2">
						<Label for="last_name">นามสกุล</Label>
						<Input
							id="last_name"
							type="text"
							placeholder="นามสกุล"
							bind:value={form.last_name}
							disabled={loading}
							required
						/>
					</div>
				</div>

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
					<Label for="email">อีเมล</Label>
					<Input
						id="email"
						type="email"
						placeholder="example@university.ac.th"
						bind:value={form.email}
						disabled={loading}
						required
					/>
				</div>
				
				<div class="space-y-2">
					<Label for="password">รหัสผ่าน</Label>
					<Input
						id="password"
						type="password"
						placeholder="กรอกรหัสผ่าน (อย่างน้อย 6 ตัวอักษร)"
						bind:value={form.password}
						disabled={loading}
						required
					/>
				</div>

				<div class="space-y-2">
					<Label for="confirm_password">ยืนยันรหัสผ่าน</Label>
					<Input
						id="confirm_password"
						type="password"
						placeholder="ยืนยันรหัสผ่าน"
						bind:value={confirmPassword}
						disabled={loading}
						required
					/>
				</div>

				{#if error}
					<div class="text-sm text-destructive bg-destructive/10 p-3 rounded-md">
						{error}
					</div>
				{/if}

				{#if success}
					<div class="text-sm text-green-600 bg-green-50 p-3 rounded-md">
						{success}
					</div>
				{/if}

				<Button type="submit" class="w-full" disabled={loading}>
					{loading ? 'กำลังสมัครสมาชิก...' : 'สมัครสมาชิก'}
				</Button>
			</form>
		</Card.Content>
		
		<Card.Footer class="text-center">
			<p class="text-sm text-muted-foreground">
				มีบัญชีอยู่แล้ว? 
				<a href="/auth/login" class="text-primary hover:underline">
					เข้าสู่ระบบ
				</a>
			</p>
		</Card.Footer>
	</Card.Root>
</div>