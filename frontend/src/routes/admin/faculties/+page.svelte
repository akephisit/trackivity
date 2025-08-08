<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { z } from 'zod';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
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
		IconSchool, 
		IconToggleLeft, 
		IconToggleRight,
		IconUsers,
		IconBuilding
	} from '@tabler/icons-svelte/icons';
	import { toast } from 'svelte-sonner';
	import { invalidateAll, invalidate } from '$app/navigation';

	let { data } = $props();
	let refreshing = $state(false);

	// Faculty schemas
	const facultyCreateSchema = z.object({
		name: z.string().min(1, 'กรุณากรอกชื่อคณะ'),
		code: z.string().min(1, 'กรุณากรอกรหัสคณะ').max(10, 'รหัสคณะต้องไม่เกิน 10 ตัวอักษร'),
		description: z.string().optional(),
		status: z.boolean().default(true)
	});

	// Forms
	const createForm = superForm(data.createForm, {
		validators: zodClient(facultyCreateSchema),
		onResult: async ({ result }) => {
			if (result.type === 'success') {
				toast.success('สร้างคณะสำเร็จ');
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
				toast.error('เกิดข้อผิดพลาดในการสร้างคณะ');
			}
		}
	});

	const { form: createFormData, enhance: createEnhance, errors: createErrors, submitting: createSubmitting } = createForm;

	// Dialog states
	let createDialogOpen = $state(false);
	let editDialogOpen = $state(false);
	let deleteDialogOpen = $state(false);

	// Edit form states
	let editingFaculty = $state<any>(null);
	let editFormData = $state({
		name: '',
		code: '',
		description: '',
		status: true
	});

	// Delete state
	let facultyToDelete = $state<{id: string, name: string} | null>(null);

	// Toggle loading
	let toggleLoading = $state<{[key: string]: boolean}>({});

	// Stats
	let stats = $derived({
		total: data.faculties.length,
		active: data.faculties.filter(f => f.status).length,
		inactive: data.faculties.filter(f => !f.status).length
	});

	function openCreateDialog() {
		$createFormData = {
			name: '',
			code: '',
			description: '',
			status: true
		};
		createDialogOpen = true;
	}

	function openEditDialog(faculty: any) {
		editingFaculty = faculty;
		editFormData = {
			name: faculty.name,
			code: faculty.code,
			description: faculty.description || '',
			status: faculty.status
		};
		editDialogOpen = true;
	}

	function openDeleteDialog(facultyId: string, facultyName: string) {
		facultyToDelete = { id: facultyId, name: facultyName };
		deleteDialogOpen = true;
	}

	async function handleUpdate() {
		if (!editingFaculty) return;

		try {
			const formData = new FormData();
			formData.append('facultyId', editingFaculty.id);
			formData.append('updateData', JSON.stringify(editFormData));

			const response = await fetch('?/update', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('แก้ไขคณะสำเร็จ');
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
				toast.error('เกิดข้อผิดพลาดในการแก้ไขคณะ');
			}
		} catch (error) {
			console.error('Update error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	async function handleDelete() {
		if (!facultyToDelete) return;

		try {
			const formData = new FormData();
			formData.append('facultyId', facultyToDelete.id);

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('ลบคณะสำเร็จ');
				deleteDialogOpen = false;
				facultyToDelete = null;
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
				toast.error('เกิดข้อผิดพลาดในการลบคณะ');
			}
		} catch (error) {
			console.error('Delete error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		}
	}

	async function handleToggleStatus(facultyId: string, currentStatus: boolean) {
		const newStatus = !currentStatus;
		const actionText = newStatus ? 'เปิดใช้งาน' : 'ปิดใช้งาน';

		toggleLoading = { ...toggleLoading, [facultyId]: true };

		try {
			const formData = new FormData();
			formData.append('facultyId', facultyId);

			const response = await fetch('?/toggleStatus', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success(`${actionText}คณะสำเร็จ`);
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
				toast.error(result.error || `เกิดข้อผิดพลาดในการ${actionText}คณะ`);
			}
		} catch (error) {
			console.error('Toggle status error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		} finally {
			toggleLoading = { ...toggleLoading, [facultyId]: false };
		}
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
</script>

<svelte:head>
	<title>จัดการคณะ - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 id="faculty-management-heading" class="text-4xl font-bold text-gray-900 dark:text-white">
				จัดการคณะ
			</h1>
			<p class="mt-3 text-lg text-gray-600 dark:text-gray-400">
				จัดการข้อมูลคณะในมหาวิทยาลัย รวมถึงการเปิด-ปิดการใช้งาน
			</p>
		</div>
		<Button onclick={openCreateDialog} class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 text-base font-medium">
			<IconPlus class="h-5 w-5 mr-2" />
			เพิ่มคณะใหม่
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-3 gap-6">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">คณะทั้งหมด</CardTitle>
				<IconSchool class="h-4 w-4 text-muted-foreground" />
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
				<IconUsers class="h-4 w-4 text-red-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">
					{stats.inactive}
				</div>
			</CardContent>
		</Card>
	</div>

	<!-- Faculties Table -->
	<div class="space-y-6" role="main" aria-labelledby="faculty-management-heading">
		{#if refreshing}
			<div class="flex items-center justify-center py-12" role="status" aria-live="polite">
				<IconLoader class="h-8 w-8 animate-spin mr-3 text-blue-500" />
				<span class="text-lg text-gray-600 dark:text-gray-300">กำลังรีเฟรชข้อมูล...</span>
			</div>
		{:else if data.faculties.length === 0}
			<div class="text-center py-16 text-gray-500 dark:text-gray-400">
				<IconSchool class="h-16 w-16 mx-auto mb-6 opacity-50" />
				<h3 class="text-xl font-semibold mb-2">ยังไม่มีข้อมูลคณะในระบบ</h3>
				<p class="text-gray-400 mb-6">เริ่มต้นด้วยการเพิ่มคณะแรก</p>
				<Button onclick={openCreateDialog} class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3">
					<IconPlus class="h-5 w-5 mr-2" />
					เพิ่มคณะแรก
				</Button>
			</div>
		{:else}
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-3">
						<IconSchool class="h-6 w-6 text-blue-600" />
						รายการคณะทั้งหมด
					</CardTitle>
					<CardDescription>
						จัดการข้อมูลคณะต่างๆ ในมหาวิทยาลัย
					</CardDescription>
				</CardHeader>
				<CardContent class="p-0">
					<div class="overflow-hidden">
						<Table.Root>
							<Table.Header>
								<Table.Row class="bg-gray-50 dark:bg-gray-800">
									<Table.Head class="font-semibold">ชื่อคณะ</Table.Head>
									<Table.Head class="font-semibold">รหัส</Table.Head>
									<Table.Head class="font-semibold">คำอธิบาย</Table.Head>
									<Table.Head class="font-semibold">สถานะ</Table.Head>
									<Table.Head class="font-semibold">วันที่สร้าง</Table.Head>
									<Table.Head class="text-right font-semibold">การดำเนินการ</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each data.faculties as faculty (faculty.id)}
									<Table.Row class="hover:bg-gray-50/50 dark:hover:bg-gray-800/50">
										<Table.Cell class="font-medium py-4">
											<div class="flex items-center gap-3">
												<div class="w-10 h-10 bg-blue-100 dark:bg-blue-900 rounded-lg flex items-center justify-center">
													<IconSchool class="h-5 w-5 text-blue-600 dark:text-blue-400" />
												</div>
												<div>
													<div class="font-semibold text-gray-900 dark:text-gray-100">
														{faculty.name}
													</div>
												</div>
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<Badge variant="outline" class="font-mono">
												{faculty.code}
											</Badge>
										</Table.Cell>
										<Table.Cell class="py-4 max-w-xs">
											<div class="text-sm text-gray-600 dark:text-gray-300 truncate">
												{faculty.description || '-'}
											</div>
										</Table.Cell>
										<Table.Cell class="py-4">
											<Badge 
												variant={faculty.status ? 'default' : 'secondary'}
												class={faculty.status ? 'bg-green-100 text-green-800 hover:bg-green-100' : 'bg-gray-100 text-gray-600'}
											>
												<span class="w-2 h-2 rounded-full mr-2" 
													class:bg-green-500={faculty.status} 
													class:bg-gray-400={!faculty.status}
												></span>
												{faculty.status ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}
											</Badge>
										</Table.Cell>
										<Table.Cell class="py-4 text-sm text-gray-500">
											{formatDateTime(faculty.created_at)}
										</Table.Cell>
										<Table.Cell class="text-right py-4">
											<div class="flex items-center gap-1 justify-end">
												<Button 
													variant="ghost" 
													size="sm" 
													onclick={() => handleToggleStatus(faculty.id, faculty.status)}
													disabled={toggleLoading[faculty.id] || false}
													class="{faculty.status ? 'text-orange-600 hover:text-orange-700 hover:bg-orange-50' : 'text-green-600 hover:text-green-700 hover:bg-green-50'} transition-colors"
													title="{faculty.status ? 'ปิดใช้งาน' : 'เปิดใช้งาน'}คณะ"
												>
													{#if toggleLoading[faculty.id]}
														<IconLoader class="h-4 w-4 animate-spin" />
													{:else if faculty.status}
														<IconToggleLeft class="h-4 w-4" />
													{:else}
														<IconToggleRight class="h-4 w-4" />
													{/if}
												</Button>
												<Button 
													variant="ghost" 
													size="sm" 
													onclick={() => openEditDialog(faculty)}
													class="text-blue-600 hover:text-blue-700 hover:bg-blue-50"
													title="แก้ไขคณะ"
												>
													<IconEdit class="h-4 w-4" />
												</Button>
												<Button
													variant="ghost"
													size="sm"
													onclick={() => openDeleteDialog(faculty.id, faculty.name)}
													class="text-red-600 hover:text-red-700 hover:bg-red-50"
													title="ลบคณะ"
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
				</CardContent>
			</Card>
		{/if}
	</div>
</div>

<!-- Create Faculty Dialog -->
<Dialog.Root bind:open={createDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>เพิ่มคณะใหม่</Dialog.Title>
			<Dialog.Description>
				กรอกข้อมูลเพื่อสร้างคณะใหม่ในระบบ
			</Dialog.Description>
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
						<Label for={props.id}>ชื่อคณะ</Label>
						<Input
							{...props}
							bind:value={$createFormData.name}
							placeholder="เช่น คณะวิทยาศาสตร์"
							disabled={$createSubmitting}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field form={createForm} name="code">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>รหัสคณะ</Label>
						<Input
							{...props}
							bind:value={$createFormData.code}
							placeholder="เช่น SCI"
							disabled={$createSubmitting}
							class="font-mono"
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
							placeholder="คำอธิบายเพิ่มเติมเกี่ยวกับคณะ"
							disabled={$createSubmitting}
							rows={3}
						/>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<div class="flex items-center space-x-2">
				<Switch bind:checked={$createFormData.status} disabled={$createSubmitting} />
				<Label>เปิดใช้งานทันทีหลังสร้าง</Label>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => createDialogOpen = false}>
					ยกเลิก
				</Button>
				<Button type="submit" disabled={$createSubmitting}>
					{#if $createSubmitting}
						<IconLoader class="mr-2 h-4 w-4 animate-spin" />
						กำลังสร้าง...
					{:else}
						สร้างคณะ
					{/if}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<!-- Edit Faculty Dialog -->
<Dialog.Root bind:open={editDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>แก้ไขคณะ</Dialog.Title>
			<Dialog.Description>
				แก้ไขข้อมูลของคณะ
			</Dialog.Description>
		</Dialog.Header>

		{#if editingFaculty}
			<div class="space-y-4">
				<div class="space-y-2">
					<Label>ชื่อคณะ</Label>
					<Input
						bind:value={editFormData.name}
						placeholder="เช่น คณะวิทยาศาสตร์"
					/>
				</div>

				<div class="space-y-2">
					<Label>รหัสคณะ</Label>
					<Input
						bind:value={editFormData.code}
						placeholder="เช่น SCI"
						class="font-mono"
					/>
				</div>

				<div class="space-y-2">
					<Label>คำอธิบาย</Label>
					<Textarea
						bind:value={editFormData.description}
						placeholder="คำอธิบายเพิ่มเติมเกี่ยวกับคณะ"
						rows={3}
					/>
				</div>

				<div class="flex items-center space-x-2">
					<Switch bind:checked={editFormData.status} />
					<Label>สถานะการใช้งาน</Label>
				</div>

				<Dialog.Footer>
					<Button type="button" variant="outline" onclick={() => editDialogOpen = false}>
						ยกเลิก
					</Button>
					<Button type="button" onclick={handleUpdate}>
						บันทึกการแก้ไข
					</Button>
				</Dialog.Footer>
			</div>
		{/if}
	</Dialog.Content>
</Dialog.Root>

<!-- Delete Faculty Confirmation Dialog -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>ยืนยันการลบคณะ</AlertDialog.Title>
			<AlertDialog.Description>
				{#if facultyToDelete}
					คุณแน่ใจหรือไม่ที่จะลบคณะ "{facultyToDelete.name}"?<br />
					<strong class="text-red-600">การดำเนินการนี้จะลบข้อมูลทั้งหมดที่เกี่ยวข้องกับคณะนี้ รวมถึงภาควิชาและแอดมินในคณะ</strong><br />
					การดำเนินการนี้ไม่สามารถยกเลิกได้
				{:else}
					กำลังโหลดข้อมูล...
				{/if}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => {
				deleteDialogOpen = false;
				facultyToDelete = null;
			}}>
				ยกเลิก
			</AlertDialog.Cancel>
			<AlertDialog.Action 
				onclick={handleDelete}
				class="bg-red-600 hover:bg-red-700 text-white"
			>
				ลบคณะ
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>