<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { apiClient, ApiError, type LoginRequest } from '$lib/api/client';
	import { authStore } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	let studentId = '';
	let password = '';
	let isLoading = false;

	async function handleSubmit() {
		if (!studentId || !password) {
			toast.error('กรุณากรอกข้อมูลให้ครบถ้วน');
			return;
		}

		isLoading = true;

		try {
			const credentials: LoginRequest = {
				student_id: studentId,
				password: password
			};

			const response = await apiClient.login(credentials);
			authStore.setUser(response.user);
			toast.success('เข้าสู่ระบบสำเร็จ');
			goto('/dashboard');
		} catch (error) {
			if (error instanceof ApiError) {
				if (error.status === 401) {
					toast.error('รหัสนักศึกษาหรือรหัสผ่านไม่ถูกต้อง');
				} else {
					toast.error('เกิดข้อผิดพลาดในการเข้าสู่ระบบ');
				}
			} else {
				toast.error('เกิดข้อผิดพลาดที่ไม่คาดคิด');
			}
			console.error('Login error:', error);
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>เข้าสู่ระบบ - Trackivity</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 flex items-center justify-center min-h-screen">
	<Card class="w-full max-w-md">
		<CardHeader class="space-y-1">
			<CardTitle class="text-2xl text-center">เข้าสู่ระบบ</CardTitle>
			<CardDescription class="text-center">
				กรอกรหัสนักศึกษาและรหัสผ่านเพื่อเข้าสู่ระบบ
			</CardDescription>
		</CardHeader>
		<CardContent>
			<form on:submit|preventDefault={handleSubmit} class="space-y-4">
				<div class="space-y-2">
					<label for="student-id" class="text-sm font-medium">
						รหัสนักศึกษา
					</label>
					<input
						id="student-id"
						type="text"
						bind:value={studentId}
						placeholder="กรอกรหัสนักศึกษา"
						required
						class="w-full px-3 py-2 border border-input rounded-md bg-background"
						disabled={isLoading}
					/>
				</div>
				
				<div class="space-y-2">
					<label for="password" class="text-sm font-medium">
						รหัสผ่าน
					</label>
					<input
						id="password"
						type="password"
						bind:value={password}
						placeholder="กรอกรหัสผ่าน"
						required
						class="w-full px-3 py-2 border border-input rounded-md bg-background"
						disabled={isLoading}
					/>
				</div>

				<Button type="submit" class="w-full" disabled={isLoading}>
					{isLoading ? 'กำลังเข้าสู่ระบบ...' : 'เข้าสู่ระบบ'}
				</Button>
			</form>

			<div class="mt-4 text-center">
				<p class="text-sm text-muted-foreground">
					ยังไม่มีบัญชี? 
					<a href="/auth/register" class="text-primary hover:underline">
						สมัครสมาชิก
					</a>
				</p>
			</div>
		</CardContent>
	</Card>
</div>