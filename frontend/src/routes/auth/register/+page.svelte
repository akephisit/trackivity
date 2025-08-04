<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { apiClient, ApiError, type RegisterRequest } from '$lib/api/client';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';

	let studentId = '';
	let email = '';
	let password = '';
	let confirmPassword = '';
	let firstName = '';
	let lastName = '';
	let isLoading = false;

	async function handleSubmit() {
		if (!studentId || !email || !password || !firstName || !lastName) {
			toast.error('กรุณากรอกข้อมูลให้ครบถ้วน');
			return;
		}

		if (password !== confirmPassword) {
			toast.error('รหัสผ่านไม่ตรงกัน');
			return;
		}

		if (password.length < 8) {
			toast.error('รหัสผ่านต้องมีอย่างน้อย 8 ตัวอักษร');
			return;
		}

		isLoading = true;

		try {
			const userData: RegisterRequest = {
				student_id: studentId,
				email: email,
				password: password,
				first_name: firstName,
				last_name: lastName
			};

			await apiClient.register(userData);
			toast.success('สมัครสมาชิกสำเร็จ กรุณาเข้าสู่ระบบ');
			goto('/auth/login');
		} catch (error) {
			if (error instanceof ApiError) {
				if (error.status === 400) {
					toast.error('ข้อมูลไม่ถูกต้องหรือรหัสนักศึกษา/อีเมลถูกใช้แล้ว');
				} else {
					toast.error('เกิดข้อผิดพลาดในการสมัครสมาชิก');
				}
			} else {
				toast.error('เกิดข้อผิดพลาดที่ไม่คาดคิด');
			}
			console.error('Register error:', error);
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>สมัครสมาชิก - Trackivity</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 flex items-center justify-center min-h-screen">
	<Card class="w-full max-w-md">
		<CardHeader class="space-y-1">
			<CardTitle class="text-2xl text-center">สมัครสมาชิก</CardTitle>
			<CardDescription class="text-center">
				สร้างบัญชีผู้ใช้ใหม่สำหรับระบบ Trackivity
			</CardDescription>
		</CardHeader>
		<CardContent>
			<form on:submit|preventDefault={handleSubmit} class="space-y-4">
				<div class="grid grid-cols-2 gap-4">
					<div class="space-y-2">
						<label for="first-name" class="text-sm font-medium">
							ชื่อ
						</label>
						<input
							id="first-name"
							type="text"
							bind:value={firstName}
							placeholder="ชื่อจริง"
							required
							class="w-full px-3 py-2 border border-input rounded-md bg-background"
							disabled={isLoading}
						/>
					</div>
					
					<div class="space-y-2">
						<label for="last-name" class="text-sm font-medium">
							นามสกุล
						</label>
						<input
							id="last-name"
							type="text"
							bind:value={lastName}
							placeholder="นามสกุล"
							required
							class="w-full px-3 py-2 border border-input rounded-md bg-background"
							disabled={isLoading}
						/>
					</div>
				</div>

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
					<label for="email" class="text-sm font-medium">
						อีเมล
					</label>
					<input
						id="email"
						type="email"
						bind:value={email}
						placeholder="example@university.ac.th"
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
						placeholder="กรอกรหัสผ่าน (อย่างน้อย 8 ตัวอักษร)"
						required
						class="w-full px-3 py-2 border border-input rounded-md bg-background"
						disabled={isLoading}
					/>
				</div>

				<div class="space-y-2">
					<label for="confirm-password" class="text-sm font-medium">
						ยืนยันรหัสผ่าน
					</label>
					<input
						id="confirm-password"
						type="password"
						bind:value={confirmPassword}
						placeholder="กรอกรหัสผ่านอีกครั้ง"
						required
						class="w-full px-3 py-2 border border-input rounded-md bg-background"
						disabled={isLoading}
					/>
				</div>

				<Button type="submit" class="w-full" disabled={isLoading}>
					{isLoading ? 'กำลังสมัครสมาชิก...' : 'สมัครสมาชิก'}
				</Button>
			</form>

			<div class="mt-4 text-center">
				<p class="text-sm text-muted-foreground">
					มีบัญชีแล้ว? 
					<a href="/auth/login" class="text-primary hover:underline">
						เข้าสู่ระบบ
					</a>
				</p>
			</div>
		</CardContent>
	</Card>
</div>