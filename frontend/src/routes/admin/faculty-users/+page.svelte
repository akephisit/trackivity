<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Table from '$lib/components/ui/table';
	import {
		getCoreRowModel,
		getFilteredRowModel,
		getPaginationRowModel,
		getSortedRowModel,
		type ColumnFiltersState,
		type PaginationState,
		type SortingState,
		type VisibilityState,
		type RowSelectionState
	} from '@tanstack/table-core';
	import { createSvelteTable } from '$lib/components/ui/data-table/data-table.svelte.js';
	import { FlexRender } from '$lib/components/ui/data-table';
	import { 
		IconUsers, 
		IconUserCheck, 
		IconUserX, 
		IconPlus,
		IconDownload,
		IconRefresh,
		IconSettings,
		IconTrash
	} from '@tabler/icons-svelte/icons';

	// Import our components
	import UserFilters from '$lib/components/user-management/UserFilters.svelte';
	import { getUserTableColumns, columnVisibilityPresets } from '$lib/components/user-table-columns';

	// Get data from server load function
	let { data } = $props();

	// Extract data from server load with safe defaults
	const { 
		users = { users: [], pagination: { page: 1, limit: 20, total: 0, pages: 1 } }, 
		stats, 
		faculties = [], 
		departments = [], 
		filters = {}, 
		adminLevel, 
		pagination = { page: 1, limit: 20, total: 0, pages: 1 }, 
		meta = { title: 'จัดการผู้ใช้', description: 'จัดการผู้ใช้ในระบบ' }
	} = $derived(data || {});

	// Component state
	let loading = false;

	// Table state
	let sorting = $state<SortingState>([]);
	let columnFilters = $state<ColumnFiltersState>([]);
	let columnVisibility = $state<VisibilityState>(columnVisibilityPresets.detailed);
	let rowSelection = $state<RowSelectionState>({});
	const tablePagination = $derived<PaginationState>({
		pageIndex: (pagination?.page || 1) - 1,
		pageSize: pagination?.limit || 20
	});

	// Get appropriate columns for admin level
	const columns = $derived(getUserTableColumns(adminLevel, adminLevel === 'FacultyAdmin'));

	// Create table instance
	const table = $derived(createSvelteTable({
		get data() {
			return users?.users || [];
		},
		columns,
		state: {
			get sorting() {
				return sorting;
			},
			get columnFilters() {
				return columnFilters;
			},
			get columnVisibility() {
				return columnVisibility;
			},
			get rowSelection() {
				return rowSelection;
			},
			get pagination() {
				return tablePagination;
			}
		},
		enableRowSelection: true,
		getCoreRowModel: getCoreRowModel(),
		getFilteredRowModel: getFilteredRowModel(),
		getSortedRowModel: getSortedRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		onSortingChange: (updater) => {
			if (typeof updater === 'function') {
				sorting = updater(sorting);
			} else {
				sorting = updater;
			}
		},
		onColumnFiltersChange: (updater) => {
			if (typeof updater === 'function') {
				columnFilters = updater(columnFilters);
			} else {
				columnFilters = updater;
			}
		},
		onColumnVisibilityChange: (updater) => {
			if (typeof updater === 'function') {
				columnVisibility = updater(columnVisibility);
			} else {
				columnVisibility = updater;
			}
		},
		onRowSelectionChange: (updater) => {
			if (typeof updater === 'function') {
				rowSelection = updater(rowSelection);
			} else {
				rowSelection = updater;
			}
		}
	}));

	// Track selected users
	const selectedUsers = $derived(Object.keys(rowSelection).filter(key => rowSelection[key]));

	// Permission checks
	const canManageUsers = $derived(adminLevel === 'SuperAdmin' || adminLevel === 'FacultyAdmin');
	const canViewAllFaculties = $derived(adminLevel === 'SuperAdmin');

	// Statistics calculations
	const totalUsers = $derived(stats?.total_users || 0);
	const activeUsers = $derived(stats?.active_users || 0);
	const students = $derived(stats?.students || 0);
	const faculty = $derived(stats?.faculty || 0);
</script>

<svelte:head>
	<title>{meta.title} | Trackivity</title>
	<meta name="description" content={meta.description} />
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">
				{adminLevel === 'SuperAdmin' ? 'ระบบจัดการผู้ใช้ทั้งหมด' : 'จัดการผู้ใช้คณะ'}
			</h1>
			<p class="text-muted-foreground">
				{adminLevel === 'SuperAdmin' 
					? 'จัดการผู้ใช้ทั้งระบบพร้อมการกรองตามคณะ'
					: 'จัดการข้อมูลผู้ใช้ในคณะของคุณ'}
			</p>
		</div>
		
		<div class="flex gap-2">
			{#if canManageUsers}
				<Button>
					<IconPlus class="mr-2 h-4 w-4" />
					เพิ่มผู้ใช้
				</Button>
			{/if}
			
			<Button variant="outline">
				<IconRefresh class="mr-2 h-4 w-4" />
				รีเฟรช
			</Button>
		</div>
	</div>

	<!-- Statistics Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ผู้ใช้ทั้งหมด</CardTitle>
				<IconUsers class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{totalUsers.toLocaleString()}</div>
				<p class="text-xs text-muted-foreground">
					{users?.pagination?.total || (users as any)?.total_count || 0} รายการในตาราง
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ผู้ใช้ที่เปิดใช้งาน</CardTitle>
				<IconUserCheck class="h-4 w-4 text-green-600" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{activeUsers.toLocaleString()}</div>
				<p class="text-xs text-muted-foreground">
					{totalUsers > 0 ? ((activeUsers / totalUsers) * 100).toFixed(1) : 0}% ของผู้ใช้ทั้งหมด
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">นักศึกษา</CardTitle>
				<IconUsers class="h-4 w-4 text-blue-600" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{students.toLocaleString()}</div>
				<p class="text-xs text-muted-foreground">
					{totalUsers > 0 ? ((students / totalUsers) * 100).toFixed(1) : 0}% ของผู้ใช้ทั้งหมด
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">อาจารย์</CardTitle>
				<IconUsers class="h-4 w-4 text-purple-600" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{faculty.toLocaleString()}</div>
				<p class="text-xs text-muted-foreground">
					{totalUsers > 0 ? ((faculty / totalUsers) * 100).toFixed(1) : 0}% ของผู้ใช้ทั้งหมด
				</p>
			</CardContent>
		</Card>
	</div>

	<!-- Advanced Filters -->
	<UserFilters
		filters={filters || {}}
		faculties={Array.isArray(faculties) ? faculties : []}
		departments={Array.isArray(departments) ? departments : []}
		showFacultyFilter={canViewAllFaculties}
		{loading}
		on:filtersChanged={(e) => {
			// Handle filter changes
			console.log('Filters changed:', e.detail);
		}}
		on:export={(e) => {
			// Handle export
			console.log('Export requested with filters:', e.detail);
		}}
		on:clearFilters={() => {
			// Handle clear filters
			console.log('Clear filters');
		}}
	/>

	<!-- Bulk Actions Bar -->
	{#if selectedUsers.length > 0}
		<Card class="border-primary/20 bg-primary/5">
			<CardContent class="p-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-2">
						<Badge variant="secondary">
							{selectedUsers.length} รายการที่เลือก
						</Badge>
						<span class="text-sm text-muted-foreground">
							จาก {users?.users?.length || 0} รายการ
						</span>
					</div>

					<div class="flex gap-2">
						<Button variant="outline" size="sm">
							<IconUserCheck class="mr-2 h-4 w-4" />
							เปิดใช้งาน
						</Button>
						<Button variant="outline" size="sm">
							<IconUserX class="mr-2 h-4 w-4" />
							ปิดใช้งาน
						</Button>
						<Button variant="outline" size="sm">
							<IconDownload class="mr-2 h-4 w-4" />
							ส่งออก
						</Button>
						<Button variant="destructive" size="sm">
							<IconTrash class="mr-2 h-4 w-4" />
							ลบ
						</Button>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Users Table -->
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle>รายการผู้ใช้</CardTitle>
					<CardDescription>
						แสดงผลจำนวน {users?.users?.length || 0} จาก {users?.pagination?.total || (users as any)?.total_count || 0} รายการ
					</CardDescription>
				</div>
				
				<div class="flex gap-2">
					<Button variant="outline" size="sm">
						<IconSettings class="mr-2 h-4 w-4" />
						จัดการคอลัมน์
					</Button>
				</div>
			</div>
		</CardHeader>
		
		<CardContent class="p-0">
			<div class="overflow-hidden">
				<Table.Root>
					<Table.Header>
						{#each table.getHeaderGroups() as headerGroup (headerGroup.id)}
							<Table.Row>
								{#each headerGroup.headers as header (header.id)}
									<Table.Head 
										colspan={header.colSpan}
										class={header.column.getCanSort() ? 'cursor-pointer select-none' : ''}
										onclick={header.column.getToggleSortingHandler()}
									>
										{#if !header.isPlaceholder}
											<FlexRender
												content={header.column.columnDef.header}
												context={header.getContext()}
											/>
										{/if}
									</Table.Head>
								{/each}
							</Table.Row>
						{/each}
					</Table.Header>
					
					<Table.Body>
						{#if table.getRowModel().rows?.length}
							{#each table.getRowModel().rows as row (row.id)}
								<Table.Row data-state={row.getIsSelected() && 'selected'}>
									{#each row.getVisibleCells() as cell (cell.id)}
										<Table.Cell>
											<FlexRender
												content={cell.column.columnDef.cell}
												context={cell.getContext()}
											/>
										</Table.Cell>
									{/each}
								</Table.Row>
							{/each}
						{:else}
							<Table.Row>
								<Table.Cell colspan={columns.length} class="h-24 text-center">
									<div class="flex flex-col items-center gap-2">
										<IconUsers class="h-8 w-8 text-muted-foreground" />
										<span class="text-muted-foreground">
											ไม่พบผู้ใช้ที่ตรงกับเงื่อนไขการค้นหา
										</span>
									</div>
								</Table.Cell>
							</Table.Row>
						{/if}
					</Table.Body>
				</Table.Root>
			</div>
		</CardContent>
	</Card>

	<!-- Table Pagination -->
	{#if users?.pagination?.pages && users.pagination.pages > 1}
		<Card>
			<CardContent class="p-4">
				<div class="flex items-center justify-between">
					<div class="text-sm text-muted-foreground">
						แสดงผล {((users?.pagination?.page || 1) - 1) * (users?.pagination?.limit || 20) + 1} - 
						{Math.min((users?.pagination?.page || 1) * (users?.pagination?.limit || 20), users?.pagination?.total || (users as any)?.total_count || 0)} 
						จาก {users?.pagination?.total || (users as any)?.total_count || 0} รายการ
					</div>
					
					<div class="flex items-center gap-2">
						<Button 
							variant="outline" 
							size="sm"
							disabled={(users?.pagination?.page || 1) <= 1}
						>
							หน้าก่อนหน้า
						</Button>
						
						<span class="text-sm">
							หน้า {users?.pagination?.page || 1} จาก {users?.pagination?.pages || 1}
						</span>
						
						<Button 
							variant="outline" 
							size="sm"
							disabled={(users?.pagination?.page || 1) >= (users?.pagination?.pages || 1)}
						>
							หน้าถัดไป
						</Button>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}
</div>