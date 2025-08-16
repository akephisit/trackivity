<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Table from '$lib/components/ui/table';
	import { Badge } from '$lib/components/ui/badge';
	import { Switch } from '$lib/components/ui/switch';
	import {
		IconLoader,
		IconPlus,
		IconEdit,
		IconTrash,
		IconBuilding,
		IconToggleLeft,
		IconToggleRight,
		IconUsers,
		IconSearch,
		IconFilter,
		IconUserCheck,
		IconUserPlus,
		IconShield
	} from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { invalidateAll, invalidate } from '$app/navigation';
	import type { Department, ExtendedAdminRole } from '$lib/types/admin';

	let { data } = $props();
	let refreshing = $state(false);


	// Department schemas
	const departmentCreateSchema = z.object({
		name: z.string().min(1, 'กรุณากรอกชื่อภาควิชา'),
		code: z.string().min(1, 'กรุณากรอกรหัสภาควิชา'),
		description: z.string().optional(),
		head_name: z.string().optional(),
		head_email: z.string().email('รูปแบบอีเมลไม่ถูกต้อง').optional().or(z.literal('')),
		status: z.boolean().default(true),
		faculty_id: z.string().uuid('กรุณาเลือกคณะที่ถูกต้อง').optional()
	});

	// Forms
	const createForm = superForm(data.createForm, {
		validators: zodClient(departmentCreateSchema),
		onResult: async ({ result }) => {
			if (result.type === 'success') {
				toast.success('สร้างภาควิชาสำเร็จ');
				createDialogOpen = false;

				setTimeout(async () => {
					try {
						refreshing = true;
						await invalidate('app:page-data');
						await invalidateAll();
						refreshing = false;
					} catch (error) {
						console.error('Failed to refresh data:', error);
						refreshing = false;
						window.location.reload();
					}
				}, 500);
			} else if (result.type === 'failure') {
				// Display specific error message if available
				const errorMessage = result.data?.error || 'เกิดข้อผิดพลาดในการสร้างภาควิชา';
				toast.error(errorMessage);
			}
		}
	});

	const {
		form: createFormData,
		enhance: createEnhance,
		errors: createErrors,
		submitting: createSubmitting
	} = createForm;

	// Dialog states
	let createDialogOpen = $state(false);
	let editDialogOpen = $state(false);
	let deleteDialogOpen = $state(false);

	// Edit form states
	let editingDepartment = $state<Department | null>(null);
	let editFormData = $state({
		name: '',
		code: '',
		description: '',
		head_name: '',
		head_email: '',
		status: true
	});

	// Delete state
	let departmentToDelete = $state<{ id: string; name: string } | null>(null);


	// Toggle loading
	let toggleLoading = $state<{ [key: string]: boolean }>({});

	// Search and filter states
	let searchQuery = $state('');
	let statusFilter = $state<'all' | 'active' | 'inactive'>('all');

	// Filtered departments
	let filteredDepartments = $derived.by(() => {
		let filtered = data.departments;

		// Apply search filter
		if (searchQuery.trim()) {
			const query = searchQuery.toLowerCase();
			filtered = filtered.filter(
				(dept) =>
					dept.name.toLowerCase().includes(query) ||
					(dept.description && dept.description.toLowerCase().includes(query)) ||
					(dept.head_name && dept.head_name.toLowerCase().includes(query))
			);
		}

		// Apply status filter
		if (statusFilter !== 'all') {
			filtered = filtered.filter((dept) =>
				statusFilter === 'active' ? dept.status : !dept.status
			);
		}

		return filtered;
	});

	// Stats
	let stats = $derived.by(() => ({
		total: data.departments.length,
		active: data.departments.filter((d) => d.status).length,
		inactive: data.departments.filter((d) => !d.status).length,
		totalStudents: data.departments.reduce((sum, d) => sum + (d.students_count || 0), 0),
		totalAdmins: data.departments.reduce((sum, d) => sum + (d.admins_count || 0), 0)
	}));

	function openCreateDialog() {
		$createFormData = {
			name: '',
			code: '',
			description: '',
			head_name: '',
			head_email: '',
			status: true,
			faculty_id: undefined
		};
		createDialogOpen = true;
	}

	function openEditDialog(department: Department) {
		editingDepartment = department;
		editFormData = {
			name: department.name,
			code: department.code || '',
			description: department.description || '',
			head_name: department.head_name || '',
			head_email: department.head_email || '',
			status: department.status
		};
		editDialogOpen = true;
	}

	function openDeleteDialog(departmentId: string, departmentName: string) {
		departmentToDelete = { id: departmentId, name: departmentName };
		deleteDialogOpen = true;
	}

	async function handleUpdate() {
		if (!editingDepartment) return;

		try {
			const formData = new FormData();
			formData.append('departmentId', editingDepartment.id);
			formData.append('updateData', JSON.stringify(editFormData));

			const response = await fetch('?/update', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('แก้ไขภาควิชาสำเร็จ');
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
				toast.error('เกิดข้อผิดพลาดในการแก้ไขภาควิชา');
			}
		} catch (error) {
			console.error('Update error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	async function handleDelete() {
		if (!departmentToDelete) return;

		try {
			const formData = new FormData();
			formData.append('departmentId', departmentToDelete.id);

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('ลบภาควิชาสำเร็จ');
				deleteDialogOpen = false;
				departmentToDelete = null;
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
				// Display specific error message if available
				const errorMessage = result.error || 'เกิดข้อผิดพลาดในการลบภาควิชา';
				toast.error(errorMessage);
			}
		} catch (error) {
			console.error('Delete error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	async function handleToggleStatus(departmentId: string, currentStatus: boolean) {
		const newStatus = !currentStatus;
		const actionText = newStatus ? 'เปิดใช้งาน' : 'ปิดใช้งาน';

		toggleLoading = { ...toggleLoading, [departmentId]: true };

		try {
			const formData = new FormData();
			formData.append('departmentId', departmentId);

			const response = await fetch('?/toggleStatus', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success(`${actionText}ภาควิชาสำเร็จ`);
				setTimeout(async () => {
					try {
						await invalidate('app:page-data');
						await invalidateAll();
					} catch (error) {
						console.error('Failed to refresh data after toggle status:', error);
						window.location.reload();
					}
				}, 300);
			} else {
				toast.error(result.error || `เกิดข้อผิดพลาดในการ${actionText}ภาควิชา`);
			}
		} catch (error) {
			console.error('Toggle status error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		} finally {
			toggleLoading = { ...toggleLoading, [departmentId]: false };
		}
	}

	function clearSearch() {
		searchQuery = '';
	}





	function formatDateTime(dateString: string) {
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	// Get page title based on user role
	let pageTitle = $derived(
		data.userRole === 'FacultyAdmin' && data.currentFaculty
			? `จัดการภาควิชา - ${data.currentFaculty.name}`
			: 'จัดการภาควิชา'
	);
</script>

<svelte:head>
	<title>{pageTitle} - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1
				id="department-management-heading"
				class="text-4xl font-bold text-gray-900 dark:text-white"
			>
				{#if data.userRole === 'FacultyAdmin' && data.currentFaculty}
					จัดการภาควิชา - {data.currentFaculty.name}
				{:else}
					จัดการภาควิชา
				{/if}
			</h1>
			<p class="mt-3 text-lg text-gray-600 dark:text-gray-400">
				{#if data.userRole === 'FacultyAdmin'}
					จัดการภาควิชาในคณะของคุณ รวมถึงการเปิด-ปิดการใช้งาน
				{:else}
					จัดการภาควิชาทั้งหมดในระบบ รวมถึงการเปิด-ปิดการใช้งาน
				{/if}
			</p>
		</div>
		<Button
			onclick={openCreateDialog}
			class="bg-blue-600 px-6 py-3 text-base font-medium text-white hover:bg-blue-700"
		>
			<IconPlus class="mr-2 h-5 w-5" />
			เพิ่มภาควิชาใหม่
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-5">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ภาควิชาทั้งหมด</CardTitle>
				<IconBuilding class="text-muted-foreground h-4 w-4" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats.total}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">เปิดใช้งาน</CardTitle>
				<IconBuilding class="h-4 w-4 text-green-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">
					{stats.active}
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ปิดใช้งาน</CardTitle>
				<IconBuilding class="h-4 w-4 text-red-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">
					{stats.inactive}
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">นักศึกษาทั้งหมด</CardTitle>
				<IconUsers class="h-4 w-4 text-blue-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">
					{stats.totalStudents}
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">แอดมินทั้งหมด</CardTitle>
				<IconUserCheck class="h-4 w-4 text-purple-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-purple-600">
					{stats.totalAdmins}
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Search and Filter -->
	<Card>
		<CardHeader>
			<CardTitle class="flex items-center gap-3">
				<IconFilter class="h-6 w-6 text-blue-600" />
				ค้นหาและกรอง
			</CardTitle>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex flex-col gap-4 sm:flex-row">
				<div class="flex-1">
					<div class="relative">
						<IconSearch
							class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-gray-400"
						/>
						<Input
							bind:value={searchQuery}
							placeholder="ค้นหาภาควิชา, หัวหน้าภาค, หรือคำอธิบาย..."
							class="pl-10 pr-10"
						/>
						{#if searchQuery}
							<Button
								variant="ghost"
								size="sm"
								onclick={clearSearch}
								class="absolute right-1 top-1/2 h-8 w-8 -translate-y-1/2 transform p-0"
							>
								<span class="sr-only">ล้างการค้นหา</span>
								×
							</Button>
						{/if}
					</div>
				</div>
				<div class="flex gap-2">
					<Button
						variant={statusFilter === 'all' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (statusFilter = 'all')}
					>
						ทั้งหมด ({stats.total})
					</Button>
					<Button
						variant={statusFilter === 'active' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (statusFilter = 'active')}
						class="border-green-600 text-green-600 hover:bg-green-50"
					>
						เปิดใช้งาน ({stats.active})
					</Button>
					<Button
						variant={statusFilter === 'inactive' ? 'default' : 'outline'}
						size="sm"
						onclick={() => (statusFilter = 'inactive')}
						class="border-red-600 text-red-600 hover:bg-red-50"
					>
						ปิดใช้งาน ({stats.inactive})
					</Button>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Departments Table -->
	<div class="space-y-6" role="main" aria-labelledby="department-management-heading">
		{#if refreshing}
			<div class="flex items-center justify-center py-12" role="status" aria-live="polite">
				<IconLoader class="mr-3 h-8 w-8 animate-spin text-blue-500" />
				<span class="text-lg text-gray-600 dark:text-gray-300">กำลังรีเฟรชข้อมูล...</span>
			</div>
		{:else if filteredDepartments.length === 0}
			<div class="py-16 text-center text-gray-500 dark:text-gray-400">
				{#if searchQuery || statusFilter !== 'all'}
					<IconSearch class="mx-auto mb-6 h-16 w-16 opacity-50" />
					<h3 class="mb-2 text-xl font-semibold">ไม่พบข้อมูลที่ตรงกับการค้นหา</h3>
					<p class="mb-6 text-gray-400">ลองเปลี่ยนคำค้นหาหรือตัวกรองใหม่</p>
					<Button onclick={clearSearch} variant="outline">ล้างการค้นหา</Button>
				{:else}
					<IconBuilding class="mx-auto mb-6 h-16 w-16 opacity-50" />
					<h3 class="mb-2 text-xl font-semibold">ยังไม่มีข้อมูลภาควิชาในระบบ</h3>
					<p class="mb-6 text-gray-400">เริ่มต้นด้วยการเพิ่มภาควิชาแรก</p>
					<Button
						onclick={openCreateDialog}
						class="bg-blue-600 px-6 py-3 text-white hover:bg-blue-700"
					>
						<IconPlus class="mr-2 h-5 w-5" />
						เพิ่มภาควิชาแรก
					</Button>
				{/if}
			</div>
		{:else}
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-3">
						<IconBuilding class="h-6 w-6 text-blue-600" />
						รายการภาควิชา
						<Badge variant="secondary" class="ml-2">
							{filteredDepartments.length} รายการ
						</Badge>
					</CardTitle>
					<CardDescription>จัดการข้อมูลภาควิชาต่างๆ ในระบบ</CardDescription>
				</CardHeader>
				<CardContent class="p-0">
					<div class="overflow-hidden">
						<Table.Root>
							<Table.Header>
								<Table.Row class="bg-gray-50 dark:bg-gray-800">
									<Table.Head class="font-semibold">ชื่อภาควิชา</Table.Head>
									<Table.Head class="font-semibold">รหัส</Table.Head>
									{#if data.userRole === 'SuperAdmin'}
										<Table.Head class="font-semibold">คณะ</Table.Head>
									{/if}
									<Table.Head class="font-semibold">หัวหน้าภาค</Table.Head>
									<Table.Head class="text-center font-semibold">จำนวนนักศึกษา</Table.Head>
									<Table.Head class="font-semibold">สถานะ</Table.Head>
									<Table.Head class="font-semibold">วันที่สร้าง</Table.Head>
									<Table.Head class="text-right font-semibold">การดำเนินการ</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each filteredDepartments as department (department.id)}
									<Table.Row class="hover:bg-gray-50/50 dark:hover:bg-gray-800/50">
										<Table.Cell class="py-4 font-medium">
											<div class="flex items-center gap-3">
												<div
													class="flex h-10 w-10 items-center justify-center rounded-lg bg-blue-100 dark:bg-blue-900"
												>
													<IconBuilding class="h-5 w-5 text-blue-600 dark:text-blue-400" />
												</div>
												<div>
													<div class="font-semibold text-gray-900 dark:text-gray-100">
														{department.name}
													</div>
													{#if department.description}
														<div class="max-w-xs truncate text-sm text-gray-500 dark:text-gray-400">
															{department.description}
														</div>
													{/if}
												</div>
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<Badge variant="outline">
												{department.code}
											</Badge>
										</Table.Cell>
										{#if data.userRole === 'SuperAdmin'}
											<Table.Cell class="py-4">
												{#if department.faculty}
													<Badge variant="outline">
														{department.faculty.name}
													</Badge>
												{:else}
													<span class="text-gray-400">-</span>
												{/if}
											</Table.Cell>
										{/if}
										<Table.Cell class="py-4">
											{#if department.head_name}
												<div>
													<div class="font-medium text-gray-900 dark:text-gray-100">
														{department.head_name}
													</div>
													{#if department.head_email}
														<div class="text-sm text-gray-500 dark:text-gray-400">
															{department.head_email}
														</div>
													{/if}
												</div>
											{:else}
												<span class="text-gray-400">ยังไม่ได้กำหนด</span>
											{/if}
										</Table.Cell>
										<Table.Cell class="py-4 text-center">
											<Badge variant="secondary">
												{department.students_count || 0}
											</Badge>
										</Table.Cell>
										<Table.Cell class="py-4">
											<Badge
												variant={department.status ? 'default' : 'secondary'}
												class={department.status
													? 'bg-green-100 text-green-800 hover:bg-green-100'
													: 'bg-gray-100 text-gray-600'}
											>
												<span
													class="mr-2 h-2 w-2 rounded-full"
													class:bg-green-500={department.status}
													class:bg-gray-400={!department.status}
												></span>
												{department.status ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}
											</Badge>
										</Table.Cell>
										<Table.Cell class="py-4 text-sm text-gray-500">
											{formatDateTime(department.created_at)}
										</Table.Cell>
										<Table.Cell class="py-4 text-right">
											{#if data.userRole === 'SuperAdmin' || data.userRole === 'FacultyAdmin'}
												<div class="flex items-center justify-end gap-1">
													<Button
														variant="ghost"
														size="sm"
														onclick={() => handleToggleStatus(department.id, department.status)}
														disabled={toggleLoading[department.id] || false}
														class="{department.status
															? 'text-orange-600 hover:bg-orange-50 hover:text-orange-700'
															: 'text-green-600 hover:bg-green-50 hover:text-green-700'} transition-colors"
														title="{department.status ? 'ปิดใช้งาน' : 'เปิดใช้งาน'}ภาควิชา"
													>
														{#if toggleLoading[department.id]}
															<IconLoader class="h-4 w-4 animate-spin" />
														{:else if department.status}
															<IconToggleLeft class="h-4 w-4" />
														{:else}
															<IconToggleRight class="h-4 w-4" />
														{/if}
													</Button>
													<Button
														variant="ghost"
														size="sm"
														onclick={() => openEditDialog(department)}
														class="text-blue-600 hover:bg-blue-50 hover:text-blue-700"
														title="แก้ไขภาควิชา"
													>
														<IconEdit class="h-4 w-4" />
													</Button>
													<Button
														variant="ghost"
														size="sm"
														onclick={() => openDeleteDialog(department.id, department.name)}
														class="text-red-600 hover:bg-red-50 hover:text-red-700"
														title="ลบภาควิชา"
													>
														<IconTrash class="h-4 w-4" />
													</Button>
												</div>
											{:else}
												<div class="text-center text-sm text-gray-500">
													-
												</div>
											{/if}
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</div>
				</CardContent>
			</Card>
		{/if}
	</div>
</div>

<!-- Create Department Dialog -->
<Dialog.Root bind:open={createDialogOpen}>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>เพิ่มภาควิชาใหม่</Dialog.Title>
			<Dialog.Description>กรอกข้อมูลเพื่อสร้างภาควิชาใหม่ในระบบ</Dialog.Description>
		</Dialog.Header>

		<form method="POST" action="?/create" use:createEnhance class="space-y-4">
			{#if $createErrors._errors}
				<Alert variant="destructive">
					<AlertDescription>
						{$createErrors._errors[0]}
					</AlertDescription>
				</Alert>
			{/if}

			<Form.Field form={createForm} name="name">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>ชื่อภาควิชา</Label>
						<Input
							{...props}
							bind:value={$createFormData.name}
							placeholder="เช่น ภาควิชาวิทยาการคอมพิวเตอร์"
							disabled={$createSubmitting}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createForm} name="code">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>รหัสภาควิชา</Label>
						<Input
							{...props}
							bind:value={$createFormData.code}
							placeholder="เช่น CS หรือ COMP"
							disabled={$createSubmitting}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createForm} name="description">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>คำอธิบาย (ไม่บังคับ)</Label>
						<Textarea
							{...props}
							bind:value={$createFormData.description}
							placeholder="คำอธิบายเพิ่มเติมเกี่ยวกับภาควิชา"
							disabled={$createSubmitting}
							rows={3}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createForm} name="head_name">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>ชื่อหัวหน้าภาค (ไม่บังคับ)</Label>
						<Input
							{...props}
							bind:value={$createFormData.head_name}
							placeholder="เช่น รศ.ดร. สมชาย ใจดี"
							disabled={$createSubmitting}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

				<Form.Field form={createForm} name="head_email">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>อีเมลหัวหน้าภาค (ไม่บังคับ)</Label>
						<Input
							{...props}
							type="email"
							bind:value={$createFormData.head_email}
							placeholder="เช่น head@university.ac.th"
							disabled={$createSubmitting}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			{#if data.userRole !== 'FacultyAdmin'}
				<Form.Field form={createForm} name="faculty_id">
					<Form.Control>
						{#snippet children({ props })}
							<Label for={props.id}>คณะ</Label>
							<select
								{...props}
								bind:value={$createFormData.faculty_id}
								class="w-full border rounded-md p-2 bg-background"
								disabled={$createSubmitting}
							>
								<option value="" disabled selected>กรุณาเลือกคณะ</option>
								{#if data.faculties}
									{#each data.faculties as fac}
										<option value={fac.id}>{fac.name}</option>
									{/each}
								{:else}
									<option disabled>ไม่พบรายการคณะ</option>
								{/if}
							</select>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			{/if}

			<div class="flex items-center space-x-2">
				<Switch bind:checked={$createFormData.status} disabled={$createSubmitting} />
				<Label>เปิดใช้งานทันทีหลังสร้าง</Label>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (createDialogOpen = false)}>
					ยกเลิก
				</Button>
				<Button type="submit" disabled={$createSubmitting}>
					{#if $createSubmitting}
						<IconLoader class="mr-2 h-4 w-4 animate-spin" />
						กำลังสร้าง...
					{:else}
						สร้างภาควิชา
					{/if}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit Department Dialog -->
<Dialog.Root bind:open={editDialogOpen}>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>แก้ไขภาควิชา</Dialog.Title>
			<Dialog.Description>แก้ไขข้อมูลของภาควิชา</Dialog.Description>
		</Dialog.Header>

		{#if editingDepartment}
			<div class="space-y-4">
				<div class="space-y-2">
					<Label>ชื่อภาควิชา</Label>
					<Input bind:value={editFormData.name} placeholder="เช่น ภาควิชาวิทยาการคอมพิวเตอร์" />
				</div>

				<div class="space-y-2">
					<Label>รหัสภาควิชา</Label>
					<Input bind:value={editFormData.code} placeholder="เช่น CS หรือ COMP" />
				</div>

				<div class="space-y-2">
					<Label>คำอธิบาย</Label>
					<Textarea
						bind:value={editFormData.description}
						placeholder="คำอธิบายเพิ่มเติมเกี่ยวกับภาควิชา"
						rows={3}
					/>
				</div>

				<div class="space-y-2">
					<Label>ชื่อหัวหน้าภาค</Label>
					<Input bind:value={editFormData.head_name} placeholder="เช่น รศ.ดร. สมชาย ใจดี" />
				</div>

				<div class="space-y-2">
					<Label>อีเมลหัวหน้าภาค</Label>
					<Input
						type="email"
						bind:value={editFormData.head_email}
						placeholder="เช่น head@university.ac.th"
					/>
				</div>

				<div class="flex items-center space-x-2">
					<Switch bind:checked={editFormData.status} />
					<Label>สถานะการใช้งาน</Label>
				</div>

				<Dialog.Footer>
					<Button type="button" variant="outline" onclick={() => (editDialogOpen = false)}>
						ยกเลิก
					</Button>
					<Button type="button" onclick={handleUpdate}>บันทึกการแก้ไข</Button>
				</Dialog.Footer>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Department Confirmation Dialog -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>ยืนยันการลบภาควิชา</AlertDialog.Title>
			<AlertDialog.Description>
				{#if departmentToDelete}
					คุณแน่ใจหรือไม่ที่จะลบภาควิชา "{departmentToDelete.name}"?<br />
					<strong class="text-red-600"
						>หมายเหตุ: ไม่สามารถลบภาควิชาได้หากยังมีผู้ใช้หรือนักศึกษาที่สังกัดอยู่</strong
					><br />
					<span class="text-sm text-gray-600">กรุณาย้ายหรือลบผู้ใช้ทั้งหมดออกจากภาควิชานี้ก่อน</span><br />
					การดำเนินการนี้ไม่สามารถยกเลิกได้
				{:else}
					กำลังโหลดข้อมูล...
				{/if}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel
				onclick={() => {
					deleteDialogOpen = false;
					departmentToDelete = null;
				}}
			>
				ยกเลิก
			</AlertDialog.Cancel>
			<AlertDialog.Action onclick={handleDelete} class="bg-red-600 text-white hover:bg-red-700">
				ลบภาควิชา
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
