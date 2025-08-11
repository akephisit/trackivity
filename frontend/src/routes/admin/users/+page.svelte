<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import * as Table from '$lib/components/ui/table';
	import { 
		IconUsers, 
		IconUserCheck, 
		IconUserX, 
		IconPlus,
		IconDownload,
		IconRefresh,
		IconSettings
	} from '@tabler/icons-svelte/icons';
	
	// Import filtering component
	import UserFilters from '$lib/components/user-management/UserFilters.svelte';

	// Get data from server load function
	let { data } = $props();

	// Extract data from server load with safe defaults
	const { 
		users = [], 
		stats = { total_users: 0, active_users: 0, inactive_users: 0, students: 0, faculty: 0, staff: 0, recent_registrations: 0 }, 
		faculties = [],
		filters = {},
		pagination = { page: 1, limit: 20, total_count: 0, total_pages: 1 }, 
		canManageAllUsers = false,
		adminLevel
	} = $derived(data || {});

	// Status badge colors and display text (similar to admin/admins)
	function getStatusBadgeVariant(status: string) {
		switch (status) {
			case 'online': return 'default';
			case 'offline': return 'secondary';
			case 'disabled': return 'destructive';
			case 'active': return 'default';
			case 'inactive': return 'secondary';
			case 'suspended': return 'destructive';
			default: return 'secondary';
		}
	}
	
	function getStatusText(status: string): string {
		switch (status) {
			case 'online': return 'ใช้งานอยู่';
			case 'offline': return 'ไม่ออนไลน์';
			case 'disabled': return 'ปิดใช้งาน';
			case 'active': return 'เปิดใช้งาน';
			case 'inactive': return 'ไม่ใช้งาน';
			case 'suspended': return 'ระงับ';
			default: return status;
		}
	}

	// Role badge colors and display text
	function getRoleBadgeVariant(role: string) {
		switch (role) {
			case 'super_admin': return 'destructive';
			case 'faculty_admin': return 'default';
			case 'regular_admin': return 'secondary';
			case 'admin': return 'destructive';
			case 'faculty': return 'default';
			case 'staff': return 'secondary';
			case 'student': return 'outline';
			default: return 'secondary';
		}
	}
	
	function getRoleText(role: string): string {
		switch (role) {
			case 'super_admin': return 'ซุปเปอร์แอดมิน';
			case 'faculty_admin': return 'แอดมินคณะ';
			case 'regular_admin': return 'แอดมินทั่วไป';
			case 'admin': return 'แอดมิน';
			case 'faculty': return 'อาจารย์';
			case 'staff': return 'เจ้าหน้าที่';
			case 'student': return 'นักศึกษา';
			default: return role;
		}
	}

	// Format date
	function formatDate(dateString: string) {
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}
</script>

<svelte:head>
	<title>จัดการผู้ใช้ - Admin Dashboard</title>
</svelte:head>

<div class="flex-1 space-y-4 p-4 md:p-8 pt-6">
	<div class="flex items-center justify-between space-y-2">
		<h2 class="text-3xl font-bold tracking-tight">จัดการผู้ใช้</h2>
		<div class="flex items-center space-x-2">
			<Button variant="outline" size="sm">
				<IconRefresh class="h-4 w-4 mr-2" />
				รีเฟรช
			</Button>
			<Button variant="outline" size="sm">
				<IconDownload class="h-4 w-4 mr-2" />
				ส่งออก
			</Button>
			{#if canManageAllUsers}
			<Button size="sm">
				<IconPlus class="h-4 w-4 mr-2" />
				เพิ่มผู้ใช้
			</Button>
			{/if}
		</div>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ผู้ใช้ทั้งหมด</CardTitle>
				<IconUsers class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats.total_users}</div>
			</CardContent>
		</Card>
		
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ใช้งานอยู่</CardTitle>
				<IconUserCheck class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">{stats.active_users}</div>
			</CardContent>
		</Card>
		
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">ไม่ใช้งาน</CardTitle>
				<IconUserX class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-gray-600">{stats.inactive_users}</div>
			</CardContent>
		</Card>
		
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">นักเรียน</CardTitle>
				<IconSettings class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-blue-600">{stats.students}</div>
			</CardContent>
		</Card>
	</div>

	<!-- Advanced Filters -->
	<UserFilters
		filters={filters || {}}
		faculties={Array.isArray(faculties) ? faculties : []}
		departments={[]}
		showFacultyFilter={canManageAllUsers}
		on:filtersChanged={(e) => {
			// Handle filter changes by navigating with new filters
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

	<!-- Users Table -->
	<Card>
		<CardHeader>
			<CardTitle>รายการผู้ใช้</CardTitle>
			<CardDescription>
				จัดการและติดตามผู้ใช้ในระบบ
				{#if !canManageAllUsers}
				(เฉพาะคณะของคุณ)
				{/if}
			</CardDescription>
		</CardHeader>
		<CardContent>
			<div class="rounded-md border">
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>ผู้ใช้</Table.Head>
							<Table.Head>อีเมล</Table.Head>
							<Table.Head>บทบาท</Table.Head>
							<Table.Head>สถานะ</Table.Head>
							<Table.Head>คณะ/ภาควิชา</Table.Head>
							<Table.Head>เข้าสู่ระบบล่าสุด</Table.Head>
							<Table.Head>สร้างเมื่อ</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each users as user}
						<Table.Row>
							<Table.Cell class="font-medium">
								<div class="flex flex-col">
									<span>{user.first_name} {user.last_name}</span>
									{#if user.student_id}
									<span class="text-sm text-muted-foreground">{user.student_id}</span>
									{/if}
								</div>
							</Table.Cell>
							<Table.Cell>{user.email}</Table.Cell>
							<Table.Cell>
								<Badge variant={getRoleBadgeVariant(user.role)}>{getRoleText(user.role)}</Badge>
							</Table.Cell>
							<Table.Cell>
								<Badge variant={getStatusBadgeVariant(user.status)}>
									{#if user.status === 'online' || user.status === 'offline' || user.status === 'disabled'}
										<span class="w-2 h-2 rounded-full mr-2 {user.status === 'online' ? 'bg-green-500' : user.status === 'offline' ? 'bg-yellow-500' : 'bg-red-500'}"></span>
									{/if}
									{getStatusText(user.status)}
								</Badge>
							</Table.Cell>
							<Table.Cell>
								<div class="flex flex-col">
									{#if user.role === 'super_admin'}
										<!-- ซุปเปอร์แอดมินไม่ต้องแสดงคณะ -->
										<span class="text-sm text-muted-foreground">-</span>
									{:else if user.role === 'faculty_admin' || user.role === 'regular_admin'}
										<!-- แอดมินคณะและแอดมินทั่วไป แสดงเฉพาะคณะ -->
										{#if user.faculty}
											<span class="text-sm">{user.faculty.name}</span>
										{:else}
											<span class="text-sm text-muted-foreground">ไม่ระบุคณะ</span>
										{/if}
									{:else}
										<!-- นักศึกษาและอื่นๆ แสดงทั้งคณะและสาขาวิชา -->
										{#if user.faculty}
											<span class="text-sm">{user.faculty.name}</span>
										{:else}
											<span class="text-sm text-muted-foreground">ไม่ระบุคณะ</span>
										{/if}
										{#if user.department}
											<span class="text-xs text-muted-foreground">{user.department.name}</span>
										{/if}
									{/if}
								</div>
							</Table.Cell>
							<Table.Cell>
								{#if user.last_login}
								{formatDate(user.last_login)}
								{:else}
								<span class="text-muted-foreground">ไม่เคย</span>
								{/if}
							</Table.Cell>
							<Table.Cell>{formatDate(user.created_at)}</Table.Cell>
						</Table.Row>
						{:else}
						<Table.Row>
							<Table.Cell colspan={7} class="h-24 text-center">
								ไม่พบข้อมูลผู้ใช้
							</Table.Cell>
						</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</div>

			<!-- Pagination -->
			{#if pagination.total_pages > 1}
			<div class="flex items-center justify-end space-x-2 py-4">
				<div class="flex items-center space-x-6 lg:space-x-8">
					<div class="flex items-center space-x-2">
						<p class="text-sm font-medium">หน้า {pagination.page} จาก {pagination.total_pages}</p>
					</div>
					<div class="flex items-center space-x-2">
						<Button variant="outline" size="sm" disabled={pagination.page === 1}>
							ก่อนหน้า
						</Button>
						<Button variant="outline" size="sm" disabled={pagination.page === pagination.total_pages}>
							ถัดไป
						</Button>
					</div>
				</div>
			</div>
			{/if}
		</CardContent>
	</Card>
</div>