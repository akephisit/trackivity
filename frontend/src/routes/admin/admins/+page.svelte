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
	import { IconLoader, IconPlus, IconEdit, IconTrash, IconShield, IconUsers, IconMail, IconToggleLeft, IconToggleRight } from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { AdminLevel, type AdminRole } from '$lib/types/admin';
	import { invalidateAll, invalidate } from '$app/navigation';

	let { data } = $props();
	let refreshing = $state(false);

	const form = superForm(data.form, {
		validators: zodClient(adminCreateSchema),
		onResult: async ({ result }) => {
			if (result.type === 'success') {
				toast.success('สร้างแอดมินสำเร็จ');
				dialogOpen = false;
				
				// รอสักครู่แล้วค่อย invalidate เพื่อให้เซิร์ฟเวอร์ commit ข้อมูล
				// เพิ่ม delay เพื่อให้ database transaction commit เสร็จก่อน
				setTimeout(async () => {
					try {
						refreshing = true;
						console.log('Refreshing admin list after create...');
						
						// ใช้ invalidate แทน invalidateAll เพื่อ performance ดีกว่า
						await invalidate('app:page-data');
						// fallback ไป invalidateAll 
						await invalidateAll();
						
						console.log('Admin list refreshed successfully');
						refreshing = false;
					} catch (error) {
						console.error('Failed to refresh data:', error);
						refreshing = false;
						// force page reload ถ้า invalidation ล้มเหลว
						window.location.reload();
					}
				}, 500);
			} else if (result.type === 'failure') {
				toast.error('เกิดข้อผิดพลาดในการสร้างแอดมิน');
			}
		}
	});

	const { form: formData, enhance, errors, submitting } = form;

	let dialogOpen = $state(false);
	let editDialogOpen = $state(false);
	let selectedAdminLevel = $state(undefined);
	let selectedFaculty = $state(undefined);
	let editingAdmin = $state<AdminRole | null>(null);
	let editSelectedAdminLevel = $state<AdminLevel | undefined>(undefined);
	let editSelectedFaculty = $state<string | undefined>(undefined);

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
			// Ensure we set the enum value directly, not string
			const enumValue = selectedAdminLevel as AdminLevel;
			$formData.admin_level = enumValue;
		}
	});

	$effect(() => {
		if (selectedFaculty) {
			// Use faculty UUID string directly
			$formData.faculty_id = selectedFaculty;
		} else {
			// Clear faculty_id when no faculty is selected
			$formData.faculty_id = '';
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

	function getFacultyName(admin: AdminRole): string {
		if (!admin.faculty_id) return '-';
		if (admin.faculty?.name) return admin.faculty.name;
		const faculty = data.faculties.find(f => f.id === admin.faculty_id);
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
				setTimeout(async () => {
					try {
						await invalidate('app:page-data');
						await invalidateAll();
					} catch (error) {
						console.error('Failed to refresh data after delete:', error);
						window.location.reload();
					}
				}, 500);
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

	function openEditDialog(admin: AdminRole) {
		editingAdmin = admin;
		editSelectedAdminLevel = admin.admin_level;
		editSelectedFaculty = admin.faculty_id ? admin.faculty_id.toString() : undefined;
		editDialogOpen = true;
	}

	async function handleUpdate(adminId: number, updateData: {
		first_name: string;
		last_name: string;
		email: string;
		admin_level: AdminLevel;
		faculty_id?: number;
		permissions: string[];
	}) {
		try {
			const formData = new FormData();
			formData.append('adminId', adminId.toString());
			formData.append('updateData', JSON.stringify(updateData));

			const response = await fetch('?/update', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('แก้ไขแอดมินสำเร็จ');
				editDialogOpen = false;
				setTimeout(async () => {
					try {
						await invalidate('app:page-data');
						await invalidateAll();
					} catch (error) {
						console.error('Failed to refresh data after update:', error);
						window.location.reload();
					}
				}, 500);
			} else {
				toast.error('เกิดข้อผิดพลาดในการแก้ไขแอดมิน');
			}
		} catch (error) {
			console.error('Update error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	async function handleToggleStatus(adminId: number, currentStatus: boolean, adminName: string) {
		const newStatus = !currentStatus;
		const actionText = newStatus ? 'เปิดใช้งาน' : 'ปิดใช้งาน';
		
		if (!confirm(`คุณต้องการ${actionText}แอดมิน "${adminName}" หรือไม่?`)) {
			return;
		}

		try {
			const formData = new FormData();
			formData.append('adminId', adminId.toString());
			formData.append('isActive', newStatus.toString());

			const response = await fetch('?/toggleStatus', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success(`${actionText}แอดมินสำเร็จ`);
				setTimeout(async () => {
					try {
						await invalidate('app:page-data');
						await invalidateAll();
					} catch (error) {
						console.error('Failed to refresh data after toggle status:', error);
						window.location.reload();
					}
				}, 500);
			} else {
				toast.error(`เกิดข้อผิดพลาดในการ${actionText}แอดมิน`);
			}
		} catch (error) {
			console.error('Toggle status error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	function getAdminActiveStatus(admin: AdminRole): boolean {
		// Check if admin has any permissions (empty permissions means deactivated)
		return admin.permissions && admin.permissions.length > 0;
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
				<div class="text-2xl font-bold">{(data.admins || []).length}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ซุปเปอร์แอดมิน</CardTitle>
				<IconShield class="h-4 w-4 text-red-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">
					{(data.admins || []).filter(a => a.admin_level === AdminLevel.SuperAdmin).length}
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
					{(data.admins || []).filter(a => a.admin_level === AdminLevel.FacultyAdmin).length}
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
			{#if refreshing}
				<div class="flex items-center justify-center py-8">
					<IconLoader class="h-6 w-6 animate-spin mr-2" />
					<span class="text-gray-600">กำลังรีเฟรชข้อมูล...</span>
				</div>
			{:else if (data.admins || []).length > 0}
				<div class="rounded-md border">
					<Table.Root>
						<Table.Header>
							<Table.Row>
								<Table.Head>ชื่อ</Table.Head>
								<Table.Head>อีเมล</Table.Head>
								<Table.Head>ระดับ</Table.Head>
								<Table.Head>คณะ</Table.Head>
								<Table.Head>สถานะ</Table.Head>
								<Table.Head>สิทธิ์</Table.Head>
								<Table.Head class="text-right">การดำเนินการ</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each (data.admins || []) as admin}
								<Table.Row>
									<Table.Cell class="font-medium">
										{admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}` : 'ไม่ระบุ'}
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
										{getFacultyName(admin)}
									</Table.Cell>
									<Table.Cell>
										<Badge variant={getAdminActiveStatus(admin) ? 'default' : 'secondary'}>
											{getAdminActiveStatus(admin) ? 'ใช้งาน' : 'ไม่ใช้งาน'}
										</Badge>
									</Table.Cell>
									<Table.Cell>
										<div class="text-sm text-gray-500">
											{admin.permissions?.length || 0} สิทธิ์
										</div>
									</Table.Cell>
									<Table.Cell class="text-right">
										<div class="flex items-center gap-2 justify-end">
											<Button 
												variant="ghost" 
												size="sm" 
												onclick={() => handleToggleStatus(
													admin.id, 
													getAdminActiveStatus(admin), 
													admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}` : 'ไม่ระบุ'
												)}
												class={getAdminActiveStatus(admin) ? 'text-orange-600 hover:text-orange-700' : 'text-green-600 hover:text-green-700'}
											>
												{#if getAdminActiveStatus(admin)}
													<IconToggleLeft class="h-4 w-4" />
												{:else}
													<IconToggleRight class="h-4 w-4" />
												{/if}
											</Button>
											<Button variant="ghost" size="sm" onclick={() => openEditDialog(admin)}>
												<IconEdit class="h-4 w-4" />
											</Button>
											<Button
												variant="ghost"
												size="sm"
												onclick={() => handleDelete(admin.id, admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}` : 'ไม่ระบุ')}
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
									<Select.Item value={option.value}>
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
										<Select.Item value={option.value}>
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

<!-- Edit Admin Dialog -->
<Dialog.Root bind:open={editDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>แก้ไขแอดมิน</Dialog.Title>
			<Dialog.Description>
				แก้ไขข้อมูลและสิทธิ์ของแอดมิน
			</Dialog.Description>
		</Dialog.Header>

		{#if editingAdmin && editingAdmin.user}
			<div class="space-y-4">
				<div class="space-y-2">
					<Label>ชื่อ-นามสกุล</Label>
					<Input
						bind:value={editingAdmin.user.first_name}
						placeholder="ชื่อ"
						class="mb-2"
					/>
					<Input
						bind:value={editingAdmin.user.last_name}
						placeholder="นามสกุล"
					/>
				</div>

				<div class="space-y-2">
					<Label>อีเมล</Label>
					<Input
						type="email"
						bind:value={editingAdmin.user.email}
						placeholder="admin@example.com"
					/>
				</div>

				<div class="space-y-2">
					<Label>ระดับแอดมิน</Label>
					<Select.Root type="single" bind:value={editSelectedAdminLevel}>
						<Select.Trigger>
							{adminLevelOptions.find(opt => opt.value === editSelectedAdminLevel)?.label ?? "เลือกระดับแอดมิน"}
						</Select.Trigger>
						<Select.Content>
							{#each adminLevelOptions as option}
								<Select.Item value={option.value}>
									{option.label}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				{#if editSelectedAdminLevel === AdminLevel.FacultyAdmin}
					<div class="space-y-2">
						<Label>คณะ</Label>
						<Select.Root type="single" bind:value={editSelectedFaculty}>
							<Select.Trigger>
								{facultyOptions.find(opt => opt.value === (editSelectedFaculty ? parseInt(editSelectedFaculty) : undefined))?.label ?? "เลือกคณะ"}
							</Select.Trigger>
							<Select.Content>
								{#each facultyOptions as option}
									<Select.Item value={option.value.toString()}>
										{option.label}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>
				{/if}

				<Dialog.Footer>
					<Button type="button" variant="outline" onclick={() => editDialogOpen = false}>
						ยกเลิก
					</Button>
					<Button 
						type="button" 
						onclick={() => {
							if (editingAdmin && editingAdmin.user && editSelectedAdminLevel) {
								handleUpdate(editingAdmin.id, {
									first_name: editingAdmin.user.first_name,
									last_name: editingAdmin.user.last_name,
									email: editingAdmin.user.email,
									admin_level: editSelectedAdminLevel,
									faculty_id: editSelectedFaculty ? parseInt(editSelectedFaculty) : undefined,
									permissions: editingAdmin.permissions || []
								});
							}
						}}
					>
						บันทึกการแก้ไข
					</Button>
				</Dialog.Footer>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>