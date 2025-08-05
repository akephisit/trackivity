<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { adminCreateSchema } from '$lib/schemas/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import * as Select from '$lib/components/ui/select';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Table from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import { IconLoader, IconPlus, IconEdit, IconTrash, IconShield, IconUsers, IconMail } from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { AdminLevel } from '$lib/types/admin';
	import { invalidateAll } from '$app/navigation';

	let { data } = $props();

	const form = superForm(data.form, {
		validators: zodClient(adminCreateSchema),
		onResult: ({ result }) => {
			if (result.type === 'success') {
				toast.success('สร้างแอดมินสำเร็จ');
				dialogOpen = false;
				invalidateAll();
			} else if (result.type === 'failure') {
				toast.error('เกิดข้อผิดพลาดในการสร้างแอดมิน');
			}
		}
	});

	const { form: formData, enhance, errors, submitting } = form;

	let dialogOpen = $state(false);
	let selectedAdminLevel = $state(undefined);
	let selectedFaculty = $state(undefined);

	// Admin Level options
	const adminLevelOptions = [
		{ value: AdminLevel.RegularAdmin, label: 'แอดมินทั่วไป' },
		{ value: AdminLevel.FacultyAdmin, label: 'แอดมินคณะ' },
		{ value: AdminLevel.SuperAdmin, label: 'ซุปเปอร์แอดมิน' }
	];

	// Faculty options
	let facultyOptions = $derived(data.faculties.map(faculty => ({
		value: faculty.id,
		label: faculty.name
	})));

	// Update form data when select values change
	$effect(() => {
		if (selectedAdminLevel) {
			$formData.admin_level = selectedAdminLevel;
		}
	});

	$effect(() => {
		if (selectedFaculty) {
			$formData.faculty_id = selectedFaculty;
		}
	});

	function getAdminLevelText(level: AdminLevel): string {
		switch (level) {
			case AdminLevel.SuperAdmin:
				return 'ซุปเปอร์แอดมิน';
			case AdminLevel.FacultyAdmin:
				return 'แอดมินคณะ';
			case AdminLevel.RegularAdmin:
				return 'แอดมินทั่วไป';
			default:
				return 'ไม่ระบุ';
		}
	}

	function getAdminLevelBadgeVariant(level: AdminLevel): 'default' | 'secondary' | 'destructive' | 'outline' {
		switch (level) {
			case AdminLevel.SuperAdmin:
				return 'destructive';
			case AdminLevel.FacultyAdmin:
				return 'default';
			case AdminLevel.RegularAdmin:
				return 'secondary';
			default:
				return 'outline';
		}
	}

	function getFacultyName(facultyId?: number): string {
		if (!facultyId) return '-';
		const faculty = data.faculties.find(f => f.id === facultyId);
		return faculty?.name || 'ไม่พบข้อมูล';
	}

	async function handleDelete(adminId: number, adminName: string) {
		if (!confirm(`คุณแน่ใจหรือไม่ที่จะลบแอดมิน "${adminName}"?`)) {
			return;
		}

		try {
			const formData = new FormData();
			formData.append('adminId', adminId.toString());

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('ลบแอดมินสำเร็จ');
				invalidateAll();
			} else {
				toast.error('เกิดข้อผิดพลาดในการลบแอดมิน');
			}
		} catch (error) {
			console.error('Delete error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	function resetForm() {
		selectedAdminLevel = undefined;
		selectedFaculty = undefined;
		$formData = {
			email: '',
			name: '',
			admin_level: AdminLevel.RegularAdmin,
			faculty_id: undefined,
			permissions: []
		};
	}

	function openDialog() {
		resetForm();
		dialogOpen = true;
	}
</script>

<svelte:head>
	<title>จัดการแอดมิน - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				จัดการแอดมิน
			</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				จัดการผู้ดูแลระบบและกำหนดสิทธิ์การเข้าถึง
			</p>
		</div>
		<Button onclick={openDialog}>
			<IconPlus class="h-4 w-4 mr-2" />
			เพิ่มแอดมิน
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-3 gap-6">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">แอดมินทั้งหมด</CardTitle>
				<IconUsers class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{data.admins.length}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ซุปเปอร์แอดมิน</CardTitle>
				<IconShield class="h-4 w-4 text-red-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">
					{data.admins.filter(a => a.admin_level === AdminLevel.SuperAdmin).length}
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">แอดมินคณะ</CardTitle>
				<IconShield class="h-4 w-4 text-blue-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">
					{data.admins.filter(a => a.admin_level === AdminLevel.FacultyAdmin).length}
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Admins Table -->
	<Card>
		<CardHeader>
			<CardTitle>รายการแอดมิน</CardTitle>
			<CardDescription>
				รายชื่อผู้ดูแลระบบทั้งหมดและสิทธิ์การเข้าถึง
			</CardDescription>
		</CardHeader>
		<CardContent>
			{#if data.admins.length > 0}
				<div class="rounded-md border">
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>ชื่อ</Table.Head>
								<Table.Head>อีเมล</Table.Head>
								<Table.Head>ระดับ</Table.Head>
								<Table.Head>คณะ</Table.Head>
								<Table.Head>สิทธิ์</Table.Head>
								<Table.Head class="text-right">การดำเนินการ</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each data.admins as admin}
								<Table.Row>
									<Table.Cell class="font-medium">
										{admin.user?.name || 'ไม่ระบุ'}
									</Table.Cell>
									<Table.Cell>
										<div class="flex items-center gap-2">
											<IconMail class="h-4 w-4 text-gray-400" />
											{admin.user?.email || 'ไม่ระบุ'}
										</div>
									</Table.Cell>
									<Table.Cell>
										<Badge variant={getAdminLevelBadgeVariant(admin.admin_level)}>
											{getAdminLevelText(admin.admin_level)}
										</Badge>
									</Table.Cell>
									<Table.Cell>
										{getFacultyName(admin.faculty_id)}
									</Table.Cell>
									<Table.Cell>
										<div class="text-sm text-gray-500">
											{admin.permissions.length} สิทธิ์
										</div>
									</Table.Cell>
									<Table.Cell class="text-right">
										<div class="flex items-center gap-2 justify-end">
											<Button variant="ghost" size="sm">
												<IconEdit class="h-4 w-4" />
											</Button>
											<Button
												variant="ghost"
												size="sm"
												onclick={() => handleDelete(admin.id, admin.user?.name || 'ไม่ระบุ')}
												class="text-red-600 hover:text-red-700"
											>
												<IconTrash class="h-4 w-4" />
											</Button>
										</div>
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>
			{:else}
				<div class="text-center py-8 text-gray-500 dark:text-gray-400">
					<IconShield class="h-12 w-12 mx-auto mb-4 opacity-50" />
					<p>ยังไม่มีแอดมินในระบบ</p>
					<Button onclick={openDialog} class="mt-4">
						<IconPlus class="h-4 w-4 mr-2" />
						เพิ่มแอดมินคนแรก
					</Button>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

<!-- Create Admin Dialog -->
<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>เพิ่มแอดมินใหม่</Dialog.Title>
			<Dialog.Description>
				กรอกข้อมูลเพื่อสร้างผู้ดูแลระบบใหม่
			</Dialog.Description>
		</Dialog.Header>

		<form method="POST" action="?/create" use:enhance class="space-y-4">
			{#if $errors._errors}
				<Alert variant="destructive">
					<AlertDescription>
						{$errors._errors[0]}
					</AlertDescription>
				</Alert>
			{/if}

			<Form.Field {form} name="name">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>ชื่อ-นามสกุล</Label>
						<Input
							{...props}
							bind:value={$formData.name}
							placeholder="กรอกชื่อ-นามสกุล"
							disabled={$submitting}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

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
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field {form} name="admin_level">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>ระดับแอดมิน</Label>
						<Select.Root type="single" bind:value={selectedAdminLevel} disabled={$submitting}>
							<Select.Trigger>
								{adminLevelOptions.find(opt => opt.value === selectedAdminLevel)?.label ?? "เลือกระดับแอดมิน"}
							</Select.Trigger>
							<Select.Content>
								{#each adminLevelOptions as option}
									<Select.Item value={option.value.toString()}>
										{option.label}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			{#if selectedAdminLevel === AdminLevel.FacultyAdmin}
				<Form.Field {form} name="faculty_id">
					<Form.Control>
						{#snippet children({ props })}
							<Label for={props.id}>คณะ</Label>
							<Select.Root type="single" bind:value={selectedFaculty} disabled={$submitting}>
								<Select.Trigger>
									{facultyOptions.find(opt => opt.value === selectedFaculty)?.label ?? "เลือกคณะ"}
								</Select.Trigger>
								<Select.Content>
									{#each facultyOptions as option}
										<Select.Item value={option.value.toString()}>
											{option.label}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			{/if}

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => dialogOpen = false}>
					ยกเลิก
				</Button>
				<Button type="submit" disabled={$submitting}>
					{#if $submitting}
						<IconLoader class="mr-2 h-4 w-4 animate-spin" />
						กำลังสร้าง...
					{:else}
						สร้างแอดมิน
					{/if}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>