<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Select from '$lib/components/ui/select';
	import * as Popover from '$lib/components/ui/popover';
	import { Calendar } from '$lib/components/ui/calendar';
	import { Separator } from '$lib/components/ui/separator';
	import { 
		IconSearch, 
		IconFilter, 
		IconX, 
		IconCalendar,
		IconRefresh,
		IconDownload
	} from '@tabler/icons-svelte/icons';
	import type { UserFilter, Faculty, Department } from '$lib/types/admin';
	import { createEventDispatcher } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	// Props
	export let filters: UserFilter = {};
	export let faculties: Faculty[] = [];
	export let departments: Department[] = [];
	export let showFacultyFilter = true;
	export let loading = false;

	// Internal state
	let searchValue = filters.search || '';
	let selectedFaculty = filters.faculty_id || 'all';
	let selectedDepartment = filters.department_id || 'all';
	let selectedStatus = filters.status || 'all';
	let selectedRole = filters.role || 'all';
	let createdAfter = filters.created_after || '';
	let createdBefore = filters.created_before || '';
	let showAdvancedFilters = false;

	// Reactive filtered departments based on selected faculty
	$: filteredDepartments = selectedFaculty === 'all' 
		? (departments || []) 
		: (departments || []).filter(dept => dept.faculty_id === selectedFaculty);

	// Event dispatcher
	const dispatch = createEventDispatcher<{
		filtersChanged: UserFilter;
		export: UserFilter;
		clearFilters: void;
	}>();

	// Role options
	const roleOptions = [
		{ value: 'all', label: 'ทุกบทบาท' },
		{ value: 'student', label: 'นักศึกษา' },
		{ value: 'faculty', label: 'อาจารย์' },
		{ value: 'staff', label: 'เจ้าหน้าที่' },
		{ value: 'super_admin', label: 'ซุปเปอร์แอดมิน' },
		{ value: 'faculty_admin', label: 'แอดมินคณะ' },
		{ value: 'regular_admin', label: 'แอดมินทั่วไป' },
		{ value: 'admin', label: 'แอดมินอื่นๆ' }
	];

	// Status options
	const statusOptions = [
		{ value: 'all', label: 'ทุกสถานะ' },
		{ value: 'online', label: 'ใช้งานอยู่' },
		{ value: 'offline', label: 'ไม่ออนไลน์' },
		{ value: 'disabled', label: 'ปิดใช้งาน' },
		{ value: 'active', label: 'เปิดใช้งาน' },
		{ value: 'inactive', label: 'ปิดใช้งาน' },
		{ value: 'suspended', label: 'ถูกระงับ' }
	];

	// Apply filters
	function applyFilters() {
		const newFilters: UserFilter = {
			search: searchValue || undefined,
			faculty_id: selectedFaculty === 'all' ? undefined : selectedFaculty,
			department_id: selectedDepartment === 'all' ? undefined : selectedDepartment,
			status: selectedStatus === 'all' ? undefined : selectedStatus as any,
			role: selectedRole === 'all' ? undefined : selectedRole as any,
			created_after: createdAfter || undefined,
			created_before: createdBefore || undefined
		};

		// Update URL with filters
		const params = new URLSearchParams($page.url.searchParams);
		
		// Clear existing filter params
		params.delete('search');
		params.delete('faculty_id');
		params.delete('department_id');
		params.delete('status');
		params.delete('role');
		params.delete('created_after');
		params.delete('created_before');
		params.delete('page'); // Reset to first page when filters change

		// Add new filter params
		Object.entries(newFilters).forEach(([key, value]) => {
			if (value !== undefined && value !== '') {
				params.set(key, value);
			}
		});

		// Navigate with new filters
		goto(`?${params.toString()}`, { replaceState: true, noScroll: true });

		dispatch('filtersChanged', newFilters);
	}

	// Clear all filters
	function clearFilters() {
		searchValue = '';
		selectedFaculty = 'all';
		selectedDepartment = 'all';
		selectedStatus = 'all';
		selectedRole = 'all';
		createdAfter = '';
		createdBefore = '';
		showAdvancedFilters = false;

		// Clear URL params
		const params = new URLSearchParams();
		goto(`?${params.toString()}`, { replaceState: true, noScroll: true });

		dispatch('clearFilters');
	}

	// Export with current filters
	function handleExport() {
		const currentFilters: UserFilter = {
			search: searchValue || undefined,
			faculty_id: selectedFaculty === 'all' ? undefined : selectedFaculty,
			department_id: selectedDepartment === 'all' ? undefined : selectedDepartment,
			status: selectedStatus === 'all' ? undefined : selectedStatus as any,
			role: selectedRole === 'all' ? undefined : selectedRole as any,
			created_after: createdAfter || undefined,
			created_before: createdBefore || undefined
		};

		dispatch('export', currentFilters);
	}

	// Count active filters
	$: activeFiltersCount = [
		searchValue,
		selectedFaculty !== 'all' ? selectedFaculty : null,
		selectedDepartment !== 'all' ? selectedDepartment : null,
		selectedStatus !== 'all' ? selectedStatus : null,
		selectedRole !== 'all' ? selectedRole : null,
		createdAfter,
		createdBefore
	].filter(Boolean).length;

	// Handle Enter key in search
	function handleSearchKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			applyFilters();
		}
	}

	// Auto-apply filters when department changes (since it depends on faculty)
	$: if (selectedFaculty === 'all') {
		selectedDepartment = 'all';
	}

	// Initialize from URL params
	function initializeFromUrl() {
		const params = $page.url.searchParams;
		searchValue = params.get('search') || '';
		selectedFaculty = params.get('faculty_id') || 'all';
		selectedDepartment = params.get('department_id') || 'all';
		selectedStatus = params.get('status') || 'all';
		selectedRole = params.get('role') || 'all';
		createdAfter = params.get('created_after') || '';
		createdBefore = params.get('created_before') || '';
	}

	// Initialize on component mount
	initializeFromUrl();
</script>

<div class="space-y-4">
	<!-- Search and Quick Filters Row -->
	<div class="flex flex-col sm:flex-row gap-3">
		<!-- Search Input -->
		<div class="relative flex-1">
			<IconSearch class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
			<Input
				bind:value={searchValue}
				placeholder="ค้นหาชื่อ, อีเมล, รหัสนักศึกษา..."
				class="pl-10 pr-4"
				on:keydown={handleSearchKeydown}
				disabled={loading}
			/>
		</div>

		<!-- Quick Filter Buttons -->
		<div class="flex gap-2">
			<!-- Status Filter -->
			<Select.Root bind:value={selectedStatus}>
				<Select.Trigger class="w-32">
					{statusOptions.find(o => o.value === selectedStatus)?.label || 'สถานะ'}
				</Select.Trigger>
				<Select.Content>
					{#each statusOptions as option}
						<Select.Item value={option.value}>{option.label}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>

			<!-- Role Filter -->
			<Select.Root bind:value={selectedRole}>
				<Select.Trigger class="w-32">
					{roleOptions.find(o => o.value === selectedRole)?.label || 'บทบาท'}
				</Select.Trigger>
				<Select.Content>
					{#each roleOptions as option}
						<Select.Item value={option.value}>{option.label}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>

			<!-- Faculty Filter (if enabled) -->
			{#if showFacultyFilter && faculties.length > 0}
				<Select.Root bind:value={selectedFaculty}>
					<Select.Trigger class="w-40">
						{selectedFaculty === 'all' ? 'ทุกคณะ' : faculties.find(f => f.id === selectedFaculty)?.name || 'คณะ'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="all">ทุกคณะ</Select.Item>
						{#each faculties as faculty}
							<Select.Item value={faculty.id}>{faculty.name}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			{/if}

			<!-- Advanced Filters Toggle -->
			<Button
				variant="outline"
				size="sm"
				on:click={() => showAdvancedFilters = !showAdvancedFilters}
				class={showAdvancedFilters ? 'bg-muted' : ''}
			>
				<IconFilter class="h-4 w-4 mr-2" />
				ตัวกรองเพิ่มเติม
				{#if activeFiltersCount > 0}
					<Badge variant="secondary" class="ml-2 h-5 w-5 p-0 text-xs">
						{activeFiltersCount}
					</Badge>
				{/if}
			</Button>
		</div>
	</div>

	<!-- Advanced Filters Panel -->
	{#if showAdvancedFilters}
		<div class="border rounded-lg p-4 space-y-4 bg-muted/20">
			<div class="flex items-center justify-between">
				<h3 class="text-sm font-medium">ตัวกรองขั้นสูง</h3>
				<Button
					variant="ghost"
					size="sm"
					on:click={() => showAdvancedFilters = false}
				>
					<IconX class="h-4 w-4" />
				</Button>
			</div>

			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
				<!-- Department Filter -->
				{#if filteredDepartments.length > 0}
					<div>
						<label for="department-select" class="text-sm font-medium mb-2 block">สาขา/ภาควิชา</label>
						<Select.Root bind:value={selectedDepartment}>
							<Select.Trigger id="department-select">
								{selectedDepartment === 'all' ? 'ทุกสาขา' : filteredDepartments.find(d => d.id === selectedDepartment)?.name || 'เลือกสาขา'}
							</Select.Trigger>
							<Select.Content>
								<Select.Item value="all">ทุกสาขา</Select.Item>
								{#each filteredDepartments as department}
									<Select.Item value={department.id}>{department.name}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>
				{/if}

				<!-- Date Range Filters -->
				<div>
					<label for="created-after-select" class="text-sm font-medium mb-2 block">สมัครหลังจากวันที่</label>
					<Popover.Root>
						<Popover.Trigger asChild let:builder>
							<Button
								id="created-after-select"
								variant="outline"
								class="w-full justify-start text-left font-normal {!createdAfter ? 'text-muted-foreground' : ''}"
								builders={[builder]}
							>
								<IconCalendar class="mr-2 h-4 w-4" />
								{createdAfter ? new Date(createdAfter).toLocaleDateString('th-TH') : 'เลือกวันที่'}
							</Button>
						</Popover.Trigger>
						<Popover.Content class="w-auto p-0" align="start">
							<Calendar
								bind:value={createdAfter}
								initialFocus
							/>
						</Popover.Content>
					</Popover.Root>
				</div>

				<div>
					<label for="created-before-select" class="text-sm font-medium mb-2 block">สมัครก่อนวันที่</label>
					<Popover.Root>
						<Popover.Trigger asChild let:builder>
							<Button
								id="created-before-select"
								variant="outline"
								class="w-full justify-start text-left font-normal {!createdBefore ? 'text-muted-foreground' : ''}"
								builders={[builder]}
							>
								<IconCalendar class="mr-2 h-4 w-4" />
								{createdBefore ? new Date(createdBefore).toLocaleDateString('th-TH') : 'เลือกวันที่'}
							</Button>
						</Popover.Trigger>
						<Popover.Content class="w-auto p-0" align="start">
							<Calendar
								bind:value={createdBefore}
								initialFocus
							/>
						</Popover.Content>
					</Popover.Root>
				</div>
			</div>
		</div>
	{/if}

	<!-- Active Filters Display -->
	{#if activeFiltersCount > 0}
		<div class="flex items-center gap-2 flex-wrap">
			<span class="text-sm text-muted-foreground">ตัวกรองที่เปิดใช้งาน:</span>
			
			{#if searchValue}
				<Badge variant="secondary" class="gap-1">
					ค้นหา: {searchValue}
					<button on:click={() => { searchValue = ''; applyFilters(); }}>
						<IconX class="h-3 w-3" />
					</button>
				</Badge>
			{/if}

			{#if selectedStatus !== 'all'}
				<Badge variant="secondary" class="gap-1">
					สถานะ: {statusOptions.find(o => o.value === selectedStatus)?.label}
					<button on:click={() => { selectedStatus = 'all'; applyFilters(); }}>
						<IconX class="h-3 w-3" />
					</button>
				</Badge>
			{/if}

			{#if selectedRole !== 'all'}
				<Badge variant="secondary" class="gap-1">
					บทบาท: {roleOptions.find(o => o.value === selectedRole)?.label}
					<button on:click={() => { selectedRole = 'all'; applyFilters(); }}>
						<IconX class="h-3 w-3" />
					</button>
				</Badge>
			{/if}

			{#if selectedFaculty !== 'all'}
				<Badge variant="secondary" class="gap-1">
					คณะ: {faculties.find(f => f.id === selectedFaculty)?.name}
					<button on:click={() => { selectedFaculty = 'all'; applyFilters(); }}>
						<IconX class="h-3 w-3" />
					</button>
				</Badge>
			{/if}

			{#if selectedDepartment !== 'all'}
				<Badge variant="secondary" class="gap-1">
					สาขา: {departments.find(d => d.id === selectedDepartment)?.name}
					<button on:click={() => { selectedDepartment = 'all'; applyFilters(); }}>
						<IconX class="h-3 w-3" />
					</button>
				</Badge>
			{/if}
		</div>
	{/if}

	<!-- Action Buttons -->
	<div class="flex items-center justify-between">
		<div class="flex gap-2">
			<Button on:click={applyFilters} disabled={loading}>
				<IconRefresh class="h-4 w-4 mr-2" />
				ปรับปรุงผลการค้นหา
			</Button>

			{#if activeFiltersCount > 0}
				<Button variant="outline" on:click={clearFilters}>
					<IconX class="h-4 w-4 mr-2" />
					ล้างตัวกรอง
				</Button>
			{/if}
		</div>

		<Button variant="outline" on:click={handleExport}>
			<IconDownload class="h-4 w-4 mr-2" />
			ส่งออกข้อมูล
		</Button>
	</div>

	<Separator />
</div>