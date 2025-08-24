<script lang="ts">
	import { superForm } from 'sveltekit-superforms';
	import { zodClient } from 'sveltekit-superforms/adapters';
	import { adminCreateSchema, PrefixOptions } from '$lib/schemas/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import * as Form from '$lib/components/ui/form';
	import * as Select from '$lib/components/ui/select';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Table from '$lib/components/ui/table';
	import * as Collapsible from '$lib/components/ui/collapsible';
	import { Badge } from '$lib/components/ui/badge';
	// import { Separator } from '$lib/components/ui/separator';
	import { IconLoader, IconPlus, IconEdit, IconTrash, IconShield, IconUsers, IconMail, IconToggleLeft, IconToggleRight, IconChevronDown } from '@tabler/icons-svelte/icons';
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
				toast.error('เกิดข้อผิดพลาดในการสร้างแอดมิน');
			}
		}
	});

	const { form: formData, enhance, errors, submitting } = form;

	let dialogOpen = $state(false);
	let editDialogOpen = $state(false);
	let selectedAdminLevel = $state<AdminLevel | undefined>(undefined);
	let selectedFaculty = $state<string | undefined>(undefined);
	let selectedPrefix = $state<string | undefined>(undefined);
	let editingAdmin = $state<AdminRole | null>(null);
	let editSelectedAdminLevel = $state<AdminLevel | undefined>(undefined);
	let editSelectedFaculty = $state<string | undefined>(undefined);

	// State for delete confirmation
	let deleteDialogOpen = $state(false);
	let adminToDelete = $state<{id: string, userId: string, name: string} | null>(null);

	// State for toggle loading
	let toggleLoading = $state<{[key: string]: boolean}>({});

	// Admin Level options
	const adminLevelOptions = [
		{ value: AdminLevel.RegularAdmin, label: 'แอดมินทั่วไป' },
		{ value: AdminLevel.FacultyAdmin, label: 'แอดมินคณะ' },
		{ value: AdminLevel.SuperAdmin, label: 'ซุปเปอร์แอดมิน' }
	];

	// Faculty options
	let facultyOptions = $derived(data.faculties.map(faculty => ({
		value: faculty.id, // Keep as string (UUID)
		label: faculty.name
	})));

	// Update form data when select values change - using separate effects to prevent loops
	$effect(() => {
		if (selectedAdminLevel !== undefined) {
			// Always update admin_level immediately when selectedAdminLevel changes
			$formData.admin_level = selectedAdminLevel;
			console.log('Updated formData.admin_level to:', selectedAdminLevel);
			
			// Clear faculty selection when changing to SuperAdmin or RegularAdmin
			if (selectedAdminLevel === AdminLevel.SuperAdmin || selectedAdminLevel === AdminLevel.RegularAdmin) {
				selectedFaculty = undefined;
				$formData.faculty_id = undefined;
				console.log('Cleared faculty_id because admin_level is:', selectedAdminLevel);
			}
		}
	});
	
	// Separate effect for faculty changes
	$effect(() => {
		if (selectedAdminLevel === AdminLevel.FacultyAdmin && selectedFaculty !== undefined) {
			// Update faculty_id when we have both FacultyAdmin level and selected faculty
			$formData.faculty_id = selectedFaculty;
			console.log('Updated formData.faculty_id to:', selectedFaculty, 'for FacultyAdmin');
		}
	});

	// Effect for prefix changes
	$effect(() => {
		if (selectedPrefix !== undefined) {
			$formData.prefix = selectedPrefix;
			console.log('Updated formData.prefix to:', selectedPrefix);
		}
	});


	function getFacultyName(admin: AdminRole): string {
		if (!admin.faculty_id) return '-';
		if (admin.faculty?.name) return admin.faculty.name;
		const faculty = data.faculties.find(f => f.id === admin.faculty_id);
		return faculty?.name || 'ไม่พบข้อมูล';
	}

	function getRoleDisplayName(adminLevel: AdminLevel): string {
		switch (adminLevel) {
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

	function openDeleteDialog(adminId: string, userId: string, adminName: string) {
		adminToDelete = { id: adminId, userId: userId, name: adminName };
		deleteDialogOpen = true;
	}

	async function handleDelete() {
		if (!adminToDelete) return;

		try {
			const formData = new FormData();
			formData.append('adminId', adminToDelete.userId); // ใช้ userId เพื่อส่งไปยัง /api/users endpoint
			formData.append('userId', adminToDelete.userId); // ส่ง userId เพิ่มเติมเพื่อความชัดเจน

			const response = await fetch('?/delete', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success('ลบแอดมินสำเร็จ');
				deleteDialogOpen = false;
				adminToDelete = null;
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
		selectedPrefix = undefined;
		$formData = {
			prefix: '',
			first_name: '',
			last_name: '',
			email: '',
			password: '',
			admin_level: AdminLevel.RegularAdmin, // This will be overridden when user selects
			faculty_id: undefined,
			permissions: []
		};
		console.log('Form reset with default admin_level:', AdminLevel.RegularAdmin);
	}

	function openDialog() {
		resetForm();
		dialogOpen = true;
	}

	function openEditDialog(admin: AdminRole) {
		editingAdmin = admin;
		editSelectedAdminLevel = admin.admin_level;
		editSelectedFaculty = admin.faculty_id; // already string (UUID)
		editDialogOpen = true;
	}

	async function handleUpdate(adminId: string, userId: string, updateData: {
		first_name: string;
		last_name: string;
		email: string;
		admin_level: AdminLevel;
		faculty_id?: string;
		permissions: string[];
	}) {
		try {
			const formData = new FormData();
			formData.append('adminId', adminId); // เก็บ admin role id 
			formData.append('userId', userId); // ส่ง user id เพื่อใช้กับ /api/users endpoint
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

	async function handleToggleStatus(adminId: string, _userId: string, currentEnabledStatus: boolean) {
		const newStatus = !currentEnabledStatus;
		const actionText = newStatus ? 'เปิดใช้งาน' : 'ปิดใช้งาน';

		// Set loading state for this specific admin
		toggleLoading = { ...toggleLoading, [adminId]: true };

		try {
			const formData = new FormData();
			formData.append('adminId', adminId); // admin role id for the new endpoint
			formData.append('isActive', newStatus.toString()); // This gets converted to is_enabled in server

			const response = await fetch('?/toggleStatus', {
				method: 'POST',
				body: formData
			});

			const result = await response.json();

			if (result.type === 'success') {
				toast.success(`${actionText}บัญชีแอดมินสำเร็จ`);
				// รีเฟรชข้อมูลทันทีเพื่อให้ UI อัพเดต
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
				toast.error(result.error || `เกิดข้อผิดพลาดในการ${actionText}บัญชีแอดมิน`);
			}
		} catch (error) {
			console.error('Toggle status error:', error);
			toast.error('เกิดข้อผิดพลาดในการเชื่อมต่อ');
		} finally {
			// Clear loading state
			toggleLoading = { ...toggleLoading, [adminId]: false };
		}
	}

	// Function to get combined admin status for display
	function getAdminCombinedStatus(admin: AdminRole): {
		isEnabled: boolean;
		isActive: boolean;
		statusText: string;
		badgeClass: string;
		dotClass: string;
	} {
		// Check if admin account is enabled (can login)
		const isEnabled = 'is_enabled' in admin && admin.is_enabled !== undefined 
			? Boolean(admin.is_enabled)
			: Boolean(admin.permissions && admin.permissions.length > 0);

		// Check if admin has active login session
		const isActive = 'is_active' in admin && admin.is_active !== undefined && admin.is_active !== null
			? Boolean(admin.is_active)
			: false;

		// Determine combined status
		let statusText: string;
		let badgeClass: string;
		let dotClass: string;

		if (!isEnabled) {
			statusText = 'ปิดใช้งาน';
			badgeClass = 'bg-red-100 text-red-800 hover:bg-red-100';
			dotClass = 'bg-red-500';
		} else if (isActive) {
			statusText = 'ใช้งานอยู่';
			badgeClass = 'bg-green-100 text-green-800 hover:bg-green-100';
			dotClass = 'bg-green-500';
		} else {
			statusText = 'ไม่ออนไลน์';
			badgeClass = 'bg-yellow-100 text-yellow-800 hover:bg-yellow-100';
			dotClass = 'bg-yellow-500';
		}

		return {
			isEnabled,
			isActive,
			statusText,
			badgeClass,
			dotClass
		};
	}

	// Function to get account enabled status (for toggle button)
	function getAdminEnabledStatus(admin: AdminRole): boolean {
		// Check if admin account is enabled (can login)
		if ('is_enabled' in admin && admin.is_enabled !== undefined) {
			return Boolean(admin.is_enabled);
		}
		// Fallback to permissions check for backward compatibility
		return Boolean(admin.permissions && admin.permissions.length > 0);
	}

	// Group admins by level and faculty
	let groupedAdmins = $derived.by(() => {
		const admins = data.admins || [];
		
		// Create a map to track unique admin IDs and filter duplicates silently
		const adminMap = new Map();
		const uniqueAdmins = admins.filter((admin) => {
			// Create a unique key using admin role ID and user ID
			const adminKey = `${admin.id}-${admin.user_id}`;
			
			// If already seen this admin, skip it
			if (adminMap.has(adminKey)) {
				return false;
			}
			
			adminMap.set(adminKey, admin);
			return true;
		});
		
		// Separate super admins
		const superAdmins = uniqueAdmins.filter(admin => admin.admin_level === AdminLevel.SuperAdmin);
		
		// Group faculty admins AND regular admins by faculty
		const facultyAndRegularAdmins = uniqueAdmins.filter(admin => 
			admin.admin_level === AdminLevel.FacultyAdmin || admin.admin_level === AdminLevel.RegularAdmin
		);
		const facultyGroups: { [key: string]: { faculty: { id: string; name: string } | null; admins: AdminRole[] } } = {};
		
		facultyAndRegularAdmins.forEach(admin => {
			const facultyId = admin.faculty_id || 'unassigned';
			const facultyName = admin.faculty?.name || getFacultyName(admin) || 'ไม่ได้มอบหมายคณะ';
			
			if (!facultyGroups[facultyId]) {
				facultyGroups[facultyId] = {
					faculty: admin.faculty_id ? 
						{ id: admin.faculty_id, name: facultyName } : 
						null,
					admins: []
				};
			}
			facultyGroups[facultyId].admins.push(admin);
		});

		// Sort admins within each faculty group: FacultyAdmin first, then RegularAdmin
		Object.values(facultyGroups).forEach(group => {
			group.admins.sort((a, b) => {
				// FacultyAdmin (0) comes before RegularAdmin (1)
				const orderA = a.admin_level === AdminLevel.FacultyAdmin ? 0 : 1;
				const orderB = b.admin_level === AdminLevel.FacultyAdmin ? 0 : 1;
				
				if (orderA !== orderB) {
					return orderA - orderB;
				}
				
				// If same admin level, sort by name
				const nameA = (a.user?.first_name || '') + ' ' + (a.user?.last_name || '');
				const nameB = (b.user?.first_name || '') + ' ' + (b.user?.last_name || '');
				return nameA.localeCompare(nameB, 'th');
			});
		});
		
		// No separate regular admins since they're now merged with faculty admins
		const regularAdmins: AdminRole[] = [];
		
		return {
			superAdmins,
			facultyGroups: Object.entries(facultyGroups).sort(([aKey, aGroup], [bKey, bGroup]) => {
				// Sort unassigned last, then by faculty name
				if (aKey === 'unassigned') return 1;
				if (bKey === 'unassigned') return -1;
				return (aGroup.faculty?.name || '').localeCompare(bGroup.faculty?.name || '');
			}),
			regularAdmins
		};
	});
</script>

<svelte:head>
	<title>จัดการแอดมิน - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 id="admin-management-heading" class="text-4xl font-bold text-gray-900 dark:text-white">
				จัดการแอดมิน
			</h1>
			<p class="mt-3 text-lg text-gray-600 dark:text-gray-400">
				จัดการผู้ดูแลระบบและกำหนดสิทธิ์การเข้าถึง แยกตามระดับและคณะ
			</p>
		</div>
		<Button onclick={openDialog} class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 text-base font-medium">
			<IconPlus class="h-5 w-5 mr-2" />
			เพิ่มแอดมิน
		</Button>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">แอดมินทั้งหมด</CardTitle>
				<IconUsers class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{groupedAdmins.superAdmins.length + groupedAdmins.facultyGroups.reduce((acc, [, group]) => acc + group.admins.length, 0)}</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ซุปเปอร์แอดมิน</CardTitle>
				<IconShield class="h-4 w-4 text-red-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">
					{groupedAdmins.superAdmins.length}
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
					{groupedAdmins.facultyGroups.reduce((acc, [, group]) => acc + group.admins.length, 0)}
				</div>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">แอดมินคณะและทั่วไป</CardTitle>
				<IconUsers class="h-4 w-4 text-gray-500" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-gray-600">
					{groupedAdmins.facultyGroups.reduce((acc, [, group]) => acc + group.admins.length, 0)}
				</div>
				<p class="text-xs text-muted-foreground">
					รวมแอดมินในคณะทั้งหมด
				</p>
			</CardContent>
		</Card>
	</div>

	<!-- Admin Groups -->
	<div class="space-y-8" role="main" aria-labelledby="admin-management-heading">
		{#if refreshing}
			<div class="flex items-center justify-center py-12" role="status" aria-live="polite">
				<IconLoader class="h-8 w-8 animate-spin mr-3 text-blue-500" />
				<span class="text-lg text-gray-600 dark:text-gray-300">กำลังรีเฟรชข้อมูล...</span>
			</div>
		{:else if (data.admins || []).length === 0}
			<div class="text-center py-16 text-gray-500 dark:text-gray-400" aria-labelledby="no-admins-heading">
				<IconShield class="h-16 w-16 mx-auto mb-6 opacity-50" />
				<h3 id="no-admins-heading" class="text-xl font-semibold mb-2">ยังไม่มีแอดมินในระบบ</h3>
				<p class="text-gray-400 mb-6">เริ่มต้นด้วยการเพิ่มผู้ดูแลระบบคนแรก</p>
				<Button onclick={openDialog} class="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3">
					<IconPlus class="h-5 w-5 mr-2" />
					เพิ่มแอดมินคนแรก
				</Button>
			</div>
		{:else}
			<!-- Super Admins Section -->
			{#if groupedAdmins.superAdmins.length > 0}
				<section aria-labelledby="super-admin-heading">
					<Card class="border-red-200 shadow-sm">
						<CardHeader class="bg-red-50/50 dark:bg-red-950/20">
							<CardTitle id="super-admin-heading" class="flex items-center gap-3">
								<IconShield class="h-6 w-6 text-red-600" aria-hidden="true" />
								<span class="text-xl font-bold text-red-700 dark:text-red-400">ซุปเปอร์แอดมิน</span>
								<Badge variant="destructive" class="ml-2 px-3 py-1 text-sm font-semibold">
									{groupedAdmins.superAdmins.length} คน
								</Badge>
							</CardTitle>
							<CardDescription class="text-red-600 dark:text-red-300 mt-2">
								ผู้ดูแลระบบระดับสูงสุดที่มีสิทธิ์เข้าถึงและจัดการทุกส่วนของระบบ
							</CardDescription>
						</CardHeader>
						<CardContent class="p-0">
							<div class="overflow-hidden">
								<Table.Root>
									<Table.Header>
										<Table.Row class="bg-gray-50 dark:bg-gray-800 hover:bg-gray-50 dark:hover:bg-gray-800">
											<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">ชื่อ-นามสกุล</Table.Head>
											<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">อีเมล</Table.Head>
											<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">บทบาท</Table.Head>
											<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">สถานะ</Table.Head>
											<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">สิทธิ์</Table.Head>
											<Table.Head class="text-right font-semibold text-gray-900 dark:text-gray-100">การดำเนินการ</Table.Head>
										</Table.Row>
									</Table.Header>
									<Table.Body>
										{#each groupedAdmins.superAdmins as admin (`super-${admin.id}-${admin.user_id}`)}
											<Table.Row class="hover:bg-red-50/30 dark:hover:bg-red-950/10 transition-colors">
												<Table.Cell class="font-medium py-4">
													<div class="flex items-center gap-3">
														<div class="w-8 h-8 bg-red-100 dark:bg-red-900 rounded-full flex items-center justify-center">
															<IconShield class="h-4 w-4 text-red-600 dark:text-red-400" aria-hidden="true" />
														</div>
														<span class="text-gray-900 dark:text-gray-100">
															{admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}`.trim() : 'ไม่ระบุชื่อ'}
														</span>
													</div>
												</Table.Cell>
												<Table.Cell class="py-4">
													<div class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
														<IconMail class="h-4 w-4 text-gray-400" aria-hidden="true" />
														<span class="text-sm">{admin.user?.email || 'ไม่ระบุอีเมล'}</span>
													</div>
												</Table.Cell>
												<Table.Cell class="py-4">
													<Badge variant="destructive" class="bg-red-100 text-red-800 hover:bg-red-100">
														{getRoleDisplayName(admin.admin_level)}
													</Badge>
												</Table.Cell>
												<Table.Cell class="py-4">
													{@const status = getAdminCombinedStatus(admin)}
													<Badge 
														variant="default"
														class={status.badgeClass}
													>
														<span class="w-2 h-2 rounded-full mr-2 {status.dotClass}" aria-hidden="true"></span>
														{status.statusText}
													</Badge>
												</Table.Cell>
												<Table.Cell class="py-4">
													<div class="text-sm font-medium text-gray-700 dark:text-gray-300">
														{admin.permissions?.length || 0} สิทธิ์
													</div>
												</Table.Cell>
												<Table.Cell class="text-right py-4">
													<div class="flex items-center gap-1 justify-end" role="group" aria-label="การดำเนินการสำหรับ {admin.user?.first_name || 'ไม่ระบุชื่อ'}">
														<Button 
															variant="ghost" 
															size="sm" 
															onclick={() => handleToggleStatus(admin.id, admin.user_id || admin.user?.id || '', getAdminEnabledStatus(admin))}
															disabled={toggleLoading[admin.id] || false}
															class="{getAdminEnabledStatus(admin) ? 'text-orange-600 hover:text-orange-700 hover:bg-orange-50' : 'text-green-600 hover:text-green-700 hover:bg-green-50'} transition-colors"
															aria-label="{getAdminEnabledStatus(admin) ? 'ปิดการใช้งานบัญชี' : 'เปิดการใช้งานบัญชี'} {admin.user?.first_name || 'แอดมิน'}"
															title="{getAdminEnabledStatus(admin) ? 'ปิดการใช้งานบัญชี' : 'เปิดการใช้งานบัญชี'}แอดมิน"
														>
															{#if toggleLoading[admin.id]}
																<IconLoader class="h-4 w-4 animate-spin" aria-hidden="true" />
															{:else if getAdminEnabledStatus(admin)}
																<IconToggleLeft class="h-4 w-4" aria-hidden="true" />
															{:else}
																<IconToggleRight class="h-4 w-4" aria-hidden="true" />
															{/if}
														</Button>
														<Button 
															variant="ghost" 
															size="sm" 
															onclick={() => openEditDialog(admin)}
															class="text-blue-600 hover:text-blue-700 hover:bg-blue-50 transition-colors"
															aria-label="แก้ไข {admin.user?.first_name || 'แอดมิน'}"
															title="แก้ไขแอดมิน"
														>
															<IconEdit class="h-4 w-4" aria-hidden="true" />
														</Button>
														<Button
															variant="ghost"
															size="sm"
															onclick={() => openDeleteDialog(admin.id, admin.user_id || admin.user?.id || '', admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}`.trim() : 'ไม่ระบุชื่อ')}
															class="text-red-600 hover:text-red-700 hover:bg-red-50 transition-colors"
															aria-label="ลบ {admin.user?.first_name || 'แอดมิน'}"
															title="ลบแอดมิน"
														>
															<IconTrash class="h-4 w-4" aria-hidden="true" />
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
				</section>
			{/if}

			<!-- Faculty Admins Section -->
			{#if groupedAdmins.facultyGroups.length > 0}
				{#if groupedAdmins.superAdmins.length > 0}
					<div class="relative my-12">
						<div class="absolute inset-0 flex items-center">
							<div class="w-full border-t border-gray-200 dark:border-gray-700"></div>
						</div>
						<div class="relative flex justify-center text-sm">
							<span class="px-4 bg-white dark:bg-gray-900 text-gray-500 dark:text-gray-400 font-medium">แอดมินคณะและทั่วไป</span>
						</div>
					</div>
				{/if}
				
				<div class="space-y-6" role="region" aria-labelledby="faculty-admins-heading">
					<h2 id="faculty-admins-heading" class="sr-only">แอดมินคณะและทั่วไป จัดกลุ่มตามคณะ</h2>
					{#each groupedAdmins.facultyGroups as [facultyId, facultyGroup] (facultyId)}
						<Collapsible.Root open class="group">
							<Card class="border-blue-200 shadow-sm hover:shadow-md transition-shadow">
								<CardHeader class="bg-blue-50/50 dark:bg-blue-950/20 pb-4">
									<div class="flex items-center justify-between">
										<CardTitle class="flex items-center gap-3">
											<div class="flex items-center gap-3">
												<div class="w-10 h-10 bg-blue-100 dark:bg-blue-900 rounded-lg flex items-center justify-center">
													<IconShield class="h-5 w-5 text-blue-600 dark:text-blue-400" aria-hidden="true" />
												</div>
												<div>
													<h3 class="text-lg font-bold text-blue-700 dark:text-blue-300">
														{facultyGroup.faculty?.name || 'ไม่ได้มอบหมายคณะ'}
													</h3>
													<div class="flex items-center gap-2 mt-1">
														<Badge variant="default" class="bg-blue-100 text-blue-800 hover:bg-blue-100 px-2 py-1">
															{facultyGroup.admins.length} คน
														</Badge>
														{#if facultyGroup.faculty?.id}
															<span class="text-xs text-blue-600 dark:text-blue-400 font-mono bg-blue-50 dark:bg-blue-900/30 px-2 py-1 rounded">
																ID: {facultyGroup.faculty.id.slice(-8)}
															</span>
														{/if}
													</div>
												</div>
											</div>
										</CardTitle>
										<Collapsible.Trigger 
											class="flex items-center justify-center w-8 h-8 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors [&[data-state=open]>svg]:rotate-180"
											aria-label={facultyGroup.faculty?.name ? `ขยาย/หดแอดมินคณะ${facultyGroup.faculty.name}` : 'ขยาย/หดแอดมินที่ไม่ได้มอบหมายคณะ'}
											title="คลิกเพื่อขยาย/หดรายการแอดมิน"
										>
											<IconChevronDown class="h-4 w-4 text-blue-600 dark:text-blue-400 transition-transform duration-200" aria-hidden="true" />
										</Collapsible.Trigger>
									</div>
									<CardDescription class="text-blue-600 dark:text-blue-300 mt-3 ml-13">
										แอดมินคณะและแอดมินทั่วไป {facultyGroup.faculty?.name ? `ที่มีสิทธิ์จัดการข้อมูลในคณะ${facultyGroup.faculty.name}` : 'ที่ยังไม่ได้รับมอบหมายคณะที่รับผิดชอบ'}
									</CardDescription>
								</CardHeader>
								<Collapsible.Content class="data-[state=closed]:animate-collapsible-up data-[state=open]:animate-collapsible-down">
									<CardContent class="p-0">
										<div class="overflow-hidden border-t border-blue-100 dark:border-blue-800">
											<Table.Root>
												<Table.Header>
													<Table.Row class="bg-blue-25 dark:bg-blue-950/10 hover:bg-blue-25 dark:hover:bg-blue-950/10">
														<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">ชื่อ-นามสกุล</Table.Head>
														<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">อีเมล</Table.Head>
														<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">บทบาท</Table.Head>
														<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">สถานะ</Table.Head>
														<Table.Head class="font-semibold text-gray-900 dark:text-gray-100">สิทธิ์</Table.Head>
														<Table.Head class="text-right font-semibold text-gray-900 dark:text-gray-100">การดำเนินการ</Table.Head>
													</Table.Row>
												</Table.Header>
												<Table.Body>
													{#each facultyGroup.admins as admin (`faculty-${facultyId}-${admin.id}-${admin.user_id}`)}
														<Table.Row class="hover:bg-blue-50/30 dark:hover:bg-blue-950/10 transition-colors">
															<Table.Cell class="font-medium py-4">
																<div class="flex items-center gap-3">
																	<div class="w-8 h-8 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
																		<IconUsers class="h-4 w-4 text-blue-600 dark:text-blue-400" aria-hidden="true" />
																	</div>
																	<span class="text-gray-900 dark:text-gray-100">
																		{admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}`.trim() : 'ไม่ระบุชื่อ'}
																	</span>
																</div>
															</Table.Cell>
															<Table.Cell class="py-4">
																<div class="flex items-center gap-2 text-gray-600 dark:text-gray-300">
																	<IconMail class="h-4 w-4 text-gray-400" aria-hidden="true" />
																	<span class="text-sm">{admin.user?.email || 'ไม่ระบุอีเมล'}</span>
																</div>
															</Table.Cell>
															<Table.Cell class="py-4">
																<Badge variant="default" class="bg-blue-100 text-blue-800 hover:bg-blue-100">
																	{getRoleDisplayName(admin.admin_level)}
																</Badge>
															</Table.Cell>
															<Table.Cell class="py-4">
																{@const status = getAdminCombinedStatus(admin)}
																<Badge 
																	variant="default"
																	class={status.badgeClass}
																>
																	<span class="w-2 h-2 rounded-full mr-2 {status.dotClass}" aria-hidden="true"></span>
																	{status.statusText}
																</Badge>
															</Table.Cell>
															<Table.Cell class="py-4">
																<div class="text-sm font-medium text-gray-700 dark:text-gray-300">
																	{admin.permissions?.length || 0} สิทธิ์
																</div>
															</Table.Cell>
															<Table.Cell class="text-right py-4">
																<div class="flex items-center gap-1 justify-end" role="group" aria-label="การดำเนินการสำหรับ {admin.user?.first_name || 'ไม่ระบุชื่อ'}">
																	<Button 
																		variant="ghost" 
																		size="sm" 
																		onclick={() => handleToggleStatus(admin.id, admin.user_id || admin.user?.id || '', getAdminEnabledStatus(admin))}
																		disabled={toggleLoading[admin.id] || false}
																		class="{getAdminEnabledStatus(admin) ? 'text-orange-600 hover:text-orange-700 hover:bg-orange-50' : 'text-green-600 hover:text-green-700 hover:bg-green-50'} transition-colors"
																		aria-label="{getAdminEnabledStatus(admin) ? 'ปิดการใช้งานบัญชี' : 'เปิดการใช้งานบัญชี'} {admin.user?.first_name || 'แอดมิน'}"
																		title="{getAdminEnabledStatus(admin) ? 'ปิดการใช้งานบัญชี' : 'เปิดการใช้งานบัญชี'}แอดมิน"
																	>
																		{#if toggleLoading[admin.id]}
																			<IconLoader class="h-4 w-4 animate-spin" aria-hidden="true" />
																		{:else if getAdminEnabledStatus(admin)}
																			<IconToggleLeft class="h-4 w-4" aria-hidden="true" />
																		{:else}
																			<IconToggleRight class="h-4 w-4" aria-hidden="true" />
																		{/if}
																	</Button>
																	<Button 
																		variant="ghost" 
																		size="sm" 
																		onclick={() => openEditDialog(admin)}
																		class="text-blue-600 hover:text-blue-700 hover:bg-blue-50 transition-colors"
																		aria-label="แก้ไข {admin.user?.first_name || 'แอดมิน'}"
																		title="แก้ไขแอดมิน"
																	>
																		<IconEdit class="h-4 w-4" aria-hidden="true" />
																	</Button>
																	<Button
																		variant="ghost"
																		size="sm"
																		onclick={() => openDeleteDialog(admin.id, admin.user_id || admin.user?.id || '', admin.user?.first_name ? `${admin.user.first_name} ${admin.user.last_name || ''}`.trim() : 'ไม่ระบุชื่อ')}
																		class="text-red-600 hover:text-red-700 hover:bg-red-50 transition-colors"
																		aria-label="ลบ {admin.user?.first_name || 'แอดมิน'}"
																		title="ลบแอดมิน"
																	>
																		<IconTrash class="h-4 w-4" aria-hidden="true" />
																	</Button>
																</div>
															</Table.Cell>
														</Table.Row>
													{/each}
												</Table.Body>
											</Table.Root>
										</div>
									</CardContent>
								</Collapsible.Content>
							</Card>
						</Collapsible.Root>
					{/each}
				</div>
			{/if}

		{/if}
	</div>
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

			<Form.Field {form} name="prefix">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>คำนำหน้า</Label>
						<Select.Root 
							type="single" 
							bind:value={selectedPrefix} 
							disabled={$submitting}
							onValueChange={(value) => {
								console.log('Prefix changed to:', value);
								selectedPrefix = value as string;
							}}
						>
							<Select.Trigger>
								{PrefixOptions.find(opt => opt.value === selectedPrefix)?.label ?? "เลือกคำนำหน้า"}
							</Select.Trigger>
							<Select.Content>
								{#each PrefixOptions as option}
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

			<div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
				<Form.Field {form} name="first_name">
					<Form.Control>
						{#snippet children({ props })}
							<Label for={props.id}>ชื่อจริง</Label>
							<Input
								{...props}
								bind:value={$formData.first_name}
								placeholder="กรอกชื่อจริง"
								disabled={$submitting}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>

				<Form.Field {form} name="last_name">
					<Form.Control>
						{#snippet children({ props })}
							<Label for={props.id}>นามสกุล</Label>
							<Input
								{...props}
								bind:value={$formData.last_name}
								placeholder="กรอกนามสกุล"
								disabled={$submitting}
							/>
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			</div>

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

			<Form.Field {form} name="password">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>รหัสผ่าน</Label>
						<Input
							{...props}
							type="password"
							bind:value={$formData.password}
							placeholder="กรุณาใส่รหัสผ่าน"
							disabled={$submitting}
						/>
						<div class="text-xs text-gray-600 dark:text-gray-400 mt-1">
							รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร ประกอบด้วยตัวพิมพ์เล็ก พิมพ์ใหญ่ และตัวเลข
						</div>
					{/snippet}
				</Form.Control>
				<Form.FieldErrors />
			</Form.Field>

			<Form.Field {form} name="admin_level">
				<Form.Control>
					{#snippet children({ props })}
						<Label for={props.id}>ระดับแอดมิน</Label>
						<Select.Root 
							type="single" 
							bind:value={selectedAdminLevel} 
							disabled={$submitting}
							onValueChange={(value) => {
								const newLevel = value as AdminLevel;
								console.log('Admin level changed to:', newLevel);
								selectedAdminLevel = newLevel;
							}}
						>
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
							<Label for={props.id}>คณะ <span class="text-red-500">*</span></Label>
							<Select.Root 
								type="single" 
								bind:value={selectedFaculty} 
								disabled={$submitting} 
								required
								onValueChange={(value) => {
									const newFaculty = value as string;
									console.log('Faculty changed to:', newFaculty);
									selectedFaculty = newFaculty;
								}}
							>
								<Select.Trigger class={!selectedFaculty ? "border-red-300" : ""}>
									{facultyOptions.find(opt => opt.value === selectedFaculty)?.label ?? "เลือกคณะที่รับผิดชอบ"}
								</Select.Trigger>
								<Select.Content>
									{#each facultyOptions as option}
										<Select.Item value={option.value}>
											{option.label}
										</Select.Item>
									{/each}
								</Select.Content>
							</Select.Root>
							{#if selectedAdminLevel === AdminLevel.FacultyAdmin && !selectedFaculty}
								<p class="text-sm text-red-600 mt-1">กรุณาเลือกคณะสำหรับแอดมินระดับคณะ</p>
							{/if}
							<!-- Debug info -->
							{#if selectedAdminLevel === AdminLevel.FacultyAdmin}
								<div class="text-xs text-gray-500 mt-1 p-2 bg-gray-50 rounded border">
									Debug: selectedFaculty = {selectedFaculty}, formData.faculty_id = {$formData.faculty_id}
								</div>
							{/if}
						{/snippet}
					</Form.Control>
					<Form.FieldErrors />
				</Form.Field>
			{:else if selectedAdminLevel === AdminLevel.SuperAdmin}
				<div class="rounded-md bg-blue-50 dark:bg-blue-950/20 p-3">
					<div class="flex items-center">
						<IconShield class="h-5 w-5 text-blue-500 mr-2" />
						<p class="text-sm text-blue-700 dark:text-blue-300">
							ซุปเปอร์แอดมินมีสิทธิ์เข้าถึงทุกคณะ ไม่จำเป็นต้องระบุคณะเฉพาะ
						</p>
					</div>
				</div>
			{:else if selectedAdminLevel === AdminLevel.RegularAdmin}
				<div class="rounded-md bg-gray-50 dark:bg-gray-800/50 p-3">
					<div class="flex items-center">
						<IconUsers class="h-5 w-5 text-gray-500 mr-2" />
						<p class="text-sm text-gray-600 dark:text-gray-300">
							แอดมินทั่วไปจะได้รับสิทธิ์พื้นฐานในการจัดการระบบ
						</p>
					</div>
				</div>
			{/if}

			<!-- Hidden inputs to ensure form data is sent correctly -->
			<input type="hidden" name="admin_level" bind:value={$formData.admin_level} />
			<input type="hidden" name="prefix" bind:value={$formData.prefix} />
			<input type="hidden" name="first_name" bind:value={$formData.first_name} />
			<input type="hidden" name="last_name" bind:value={$formData.last_name} />
			{#if $formData.faculty_id}
				<input type="hidden" name="faculty_id" bind:value={$formData.faculty_id} />
			{/if}

			<!-- Debug section for development -->
			<div class="text-xs text-gray-500 p-2 bg-gray-50 rounded border mt-4">
				<div><strong>Debug Information:</strong></div>
				<div>Selected Admin Level: {selectedAdminLevel}</div>
				<div>Selected Faculty: {selectedFaculty}</div>
				<div>Selected Prefix: {selectedPrefix}</div>
				<div>Form Data Admin Level: {$formData.admin_level}</div>
				<div>Form Data Faculty ID: {$formData.faculty_id}</div>
				<div>Form Data Prefix: {$formData.prefix}</div>
				<div>Form Data First Name: {$formData.first_name}</div>
				<div>Form Data Last Name: {$formData.last_name}</div>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => dialogOpen = false}>
					ยกเลิก
				</Button>
				<Button 
					type="submit" 
					disabled={$submitting || (selectedAdminLevel === AdminLevel.FacultyAdmin && !selectedFaculty)}
					onclick={() => {
						console.log('=== FORM SUBMISSION DEBUG ===');
						console.log('selectedAdminLevel:', selectedAdminLevel);
						console.log('selectedFaculty:', selectedFaculty);
						console.log('selectedPrefix:', selectedPrefix);
						console.log('$formData:', $formData);
						console.log('admin_level in formData:', $formData.admin_level);
						console.log('faculty_id in formData:', $formData.faculty_id);
						console.log('prefix in formData:', $formData.prefix);
						console.log('first_name in formData:', $formData.first_name);
						console.log('last_name in formData:', $formData.last_name);
						console.log('===============================');
					}}
				>
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
								{facultyOptions.find(opt => opt.value === editSelectedFaculty)?.label ?? "เลือกคณะ"}
							</Select.Trigger>
							<Select.Content>
								{#each facultyOptions as option}
									<Select.Item value={option.value}>
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
								handleUpdate(editingAdmin.id, editingAdmin.user_id || editingAdmin.user.id, {
									first_name: editingAdmin.user.first_name,
									last_name: editingAdmin.user.last_name,
									email: editingAdmin.user.email,
									admin_level: editSelectedAdminLevel,
									faculty_id: editSelectedFaculty,
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

<!-- Delete Admin Confirmation Dialog -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>ยืนยันการลบแอดมิน</AlertDialog.Title>
			<AlertDialog.Description>
				{#if adminToDelete}
					คุณแน่ใจหรือไม่ที่จะลบแอดมิน "{adminToDelete.name}"?<br />
					การดำเนินการนี้ไม่สามารถยกเลิกได้ และจะลบข้อมูลของแอดมินออกจากระบบถาวร
				{:else}
					กำลังโหลดข้อมูล...
				{/if}
			</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={() => {
				deleteDialogOpen = false;
				adminToDelete = null;
			}}>
				ยกเลิก
			</AlertDialog.Cancel>
			<AlertDialog.Action 
				onclick={handleDelete}
				class="bg-red-600 hover:bg-red-700 text-white"
			>
				ลบแอดมิน
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>