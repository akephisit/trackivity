<script lang="ts">
	import { onMount } from 'svelte';
	import { currentUser, auth } from '$lib/stores/auth';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Separator } from '$lib/components/ui/separator';
	import {
		IconUser,
		IconMail,
		IconSchool,
		IconEdit,
		IconCheck,
		IconX,
		IconAlertCircle,
		IconShield,
		IconCalendar,
		IconPhone,
		IconMapPin
	} from '@tabler/icons-svelte';
	import { toast } from 'svelte-sonner';

	let editing = $state(false);
	let loading = $state(false);
	let error: string | null = $state(null);

	// Form data
	let formData = $state({
		first_name: '',
		last_name: '',
		email: '',
		phone: '',
		address: ''
	});

	onMount(() => {
		if ($currentUser) {
			// Initialize form with current user data
			formData = {
				first_name: $currentUser.first_name,
				last_name: $currentUser.last_name,
				email: $currentUser.email,
				phone: $currentUser.phone || '',
				address: $currentUser.address || ''
			};
		}
	});

	// Watch for user changes
	$effect(() => {
		if ($currentUser && !editing) {
			formData = {
				first_name: $currentUser.first_name,
				last_name: $currentUser.last_name,
				email: $currentUser.email,
				phone: $currentUser.phone || '',
				address: $currentUser.address || ''
			};
		}
	});

	function startEdit() {
		editing = true;
		error = null;
	}

	function cancelEdit() {
		editing = false;
		error = null;
		// Reset form to original values
		if ($currentUser) {
			formData = {
				first_name: $currentUser.first_name,
				last_name: $currentUser.last_name,
				email: $currentUser.email,
				phone: $currentUser.phone || '',
				address: $currentUser.address || ''
			};
		}
	}

	async function saveProfile() {
		if (!$currentUser) return;

		loading = true;
		error = null;

		try {
			// TODO: Implement API endpoint for updating profile
			// const response = await apiClient.updateProfile(formData);

			// Mock success for now
			toast.success('บันทึกข้อมูลส่วนตัวสำเร็จ');
			editing = false;

			// Refresh user data
			await auth.refreshUser();
		} catch (err) {
			console.error('Failed to update profile:', err);
			error = 'ไม่สามารถบันทึกข้อมูลได้ กรุณาลองอีกครั้ง';
		} finally {
			loading = false;
		}
	}

	function formatDate(dateString?: string): string {
		if (!dateString) return 'ไม่ระบุ';
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'long',
			day: 'numeric'
		});
	}

	function isFormValid(): boolean {
		return !!(formData.first_name.trim() && formData.last_name.trim() && formData.email.trim());
	}
</script>

<svelte:head>
	<title>โปรไฟล์ - Trackivity Student</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no" />
</svelte:head>

<div class="max-w-4xl space-y-6">
	<!-- Header -->
	<div class="flex flex-col justify-between gap-4 sm:flex-row sm:items-center">
		<div>
			<h1 class="text-2xl font-bold lg:text-3xl">โปรไฟล์ของฉัน</h1>
			<p class="text-muted-foreground">จัดการข้อมูลส่วนตัวและการตั้งค่าบัญชี</p>
		</div>

		{#if !editing}
			<Button onclick={startEdit} variant="outline" class="w-full sm:w-auto">
				<IconEdit class="mr-2 size-4" />
				แก้ไขข้อมูล
			</Button>
		{/if}
	</div>

	{#if $currentUser}
		<!-- Profile Info -->
		<div class="grid gap-6 lg:grid-cols-2">
			<!-- Basic Information -->
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<IconUser class="size-5" />
						ข้อมูลพื้นฐาน
					</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					{#if editing}
						<!-- Edit Mode -->
						<div class="space-y-4">
							<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
								<div class="space-y-2">
									<Label for="first_name">ชื่อ</Label>
									<Input id="first_name" bind:value={formData.first_name} placeholder="กรอกชื่อ" />
								</div>
								<div class="space-y-2">
									<Label for="last_name">นามสกุล</Label>
									<Input id="last_name" bind:value={formData.last_name} placeholder="กรอกนามสกุล" />
								</div>
							</div>

							<div class="space-y-2">
								<Label for="email">อีเมล</Label>
								<Input
									id="email"
									type="email"
									bind:value={formData.email}
									placeholder="กรอกอีเมล"
								/>
							</div>

							<div class="space-y-2">
								<Label for="phone">เบอร์โทรศัพท์</Label>
								<Input
									id="phone"
									type="tel"
									bind:value={formData.phone}
									placeholder="กรอกเบอร์โทรศัพท์ (ไม่บังคับ)"
								/>
							</div>

							<div class="space-y-2">
								<Label for="address">ที่อยู่</Label>
								<Input
									id="address"
									bind:value={formData.address}
									placeholder="กรอกที่อยู่ (ไม่บังคับ)"
								/>
							</div>

							{#if error}
								<Alert variant="destructive">
									<IconAlertCircle class="size-4" />
									<AlertDescription>{error}</AlertDescription>
								</Alert>
							{/if}

							<div class="flex gap-2">
								<Button onclick={saveProfile} disabled={loading || !isFormValid()} class="flex-1">
									{#if loading}
										กำลังบันทึก...
									{:else}
										<IconCheck class="mr-2 size-4" />
										บันทึก
									{/if}
								</Button>
								<Button variant="outline" onclick={cancelEdit} class="flex-1">
									<IconX class="mr-2 size-4" />
									ยกเลิก
								</Button>
							</div>
						</div>
					{:else}
						<!-- View Mode -->
						<div class="space-y-4">
							<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
								<div>
									<span class="text-muted-foreground text-sm">ชื่อ</span>
									<p class="font-medium">{$currentUser.first_name}</p>
								</div>
								<div>
									<span class="text-muted-foreground text-sm">นามสกุล</span>
									<p class="font-medium">{$currentUser.last_name}</p>
								</div>
							</div>

							<div>
								<span class="text-muted-foreground mb-1 flex items-center gap-1 text-sm">
									<IconMail class="size-3" />
									อีเมล
								</span>
								<p class="font-medium">{$currentUser.email}</p>
							</div>

							<div>
								<span class="text-muted-foreground mb-1 flex items-center gap-1 text-sm">
									<IconPhone class="size-3" />
									เบอร์โทรศัพท์
								</span>
								<p class="font-medium">{$currentUser.phone || 'ไม่ได้ระบุ'}</p>
							</div>

							<div>
								<span class="text-muted-foreground mb-1 flex items-center gap-1 text-sm">
									<IconMapPin class="size-3" />
									ที่อยู่
								</span>
								<p class="font-medium">{$currentUser.address || 'ไม่ได้ระบุ'}</p>
							</div>
						</div>
					{/if}
				</CardContent>
			</Card>

			<!-- Student Information -->
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<IconSchool class="size-5" />
						ข้อมูลนักศึกษา
					</CardTitle>
				</CardHeader>
				<CardContent class="space-y-4">
					<div>
						<span class="text-muted-foreground text-sm">รหัสนักศึกษา</span>
						<p class="text-lg font-medium">{$currentUser.student_id}</p>
					</div>

					{#if $currentUser.faculty_name}
						<div>
							<span class="text-muted-foreground text-sm">คณะ</span>
							<p class="font-medium">{$currentUser.faculty_name}</p>
						</div>
					{/if}

					{#if $currentUser.department_name}
						<div>
							<span class="text-muted-foreground text-sm">ภาควิชา</span>
							<p class="font-medium">{$currentUser.department_name}</p>
						</div>
					{/if}

					<Separator />

					<div>
						<span class="text-muted-foreground mb-1 flex items-center gap-1 text-sm">
							<IconCalendar class="size-3" />
							วันที่สมัครสมาชิก
						</span>
						<p class="font-medium">{formatDate($currentUser.created_at)}</p>
					</div>

					{#if $currentUser.updated_at}
						<div>
							<span class="text-muted-foreground text-sm">อัพเดตล่าสุด</span>
							<p class="text-sm">{formatDate($currentUser.updated_at)}</p>
						</div>
					{/if}
				</CardContent>
			</Card>
		</div>

		<!-- Account Status -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<IconShield class="size-5" />
					สถานะบัญชี
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
					<div class="rounded-lg border p-4 text-center">
						<Badge variant="default" class="mb-2">ใช้งานปกติ</Badge>
						<p class="text-muted-foreground text-sm">สถานะบัญชี</p>
					</div>

					<div class="rounded-lg border p-4 text-center">
						<Badge variant="secondary" class="mb-2">นักศึกษา</Badge>
						<p class="text-muted-foreground text-sm">ระดับผู้ใช้</p>
					</div>

					<div class="rounded-lg border p-4 text-center">
						<Badge variant="outline" class="mb-2">
							{$currentUser.permissions?.length || 0}
						</Badge>
						<p class="text-muted-foreground text-sm">สิทธิ์การใช้งาน</p>
					</div>

					<div class="rounded-lg border p-4 text-center">
						<Badge variant="outline" class="mb-2">เข้ารหัสแล้ว</Badge>
						<p class="text-muted-foreground text-sm">ความปลอดภัย</p>
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- Privacy Notice -->
		<Card class="border-muted bg-muted/30">
			<CardContent class="p-4">
				<div class="flex items-start gap-3">
					<IconShield class="text-muted-foreground mt-0.5 size-5 flex-shrink-0" />
					<div class="space-y-1">
						<h3 class="text-sm font-medium">ข้อมูลส่วนบุคคล</h3>
						<p class="text-muted-foreground text-xs">
							ข้อมูลส่วนบุคคลของคุณได้รับการป้องกันตามนีสยประปิสย์ของมหาวิทยาลัย
							เราจะไม่เปิดเผยข้อมูลของคุณให้บุคคลที่สามโดยไม่ได้รับความยินยอมจากคุณ
						</p>
					</div>
				</div>
			</CardContent>
		</Card>
	{:else}
		<!-- Loading State -->
		<Card>
			<CardHeader>
				<CardTitle>
					<Skeleton class="h-6 w-32" />
				</CardTitle>
			</CardHeader>
			<CardContent>
				<div class="space-y-4">
					{#each Array(4) as _}
						<div class="space-y-2">
							<Skeleton class="h-4 w-20" />
							<Skeleton class="h-10 w-full" />
						</div>
					{/each}
				</div>
			</CardContent>
		</Card>
	{/if}
</div>
