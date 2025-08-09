<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { Progress } from '$lib/components/ui/progress';
	import * as Chart from '$lib/components/ui/chart';
	import { 
		IconUsers, 
		IconShield, 
		IconBuilding, 
		IconActivity,
		IconTrendingUp,
		IconClock,
		IconCircleCheck,
		IconAlertCircle,
		IconChartBar,
		IconSchool,
		IconUsersGroup,
		IconBook,
		IconAward,
		IconShieldCheck
	} from '@tabler/icons-svelte/icons';
	import { AdminLevel } from '$lib/types/admin';
	import { BarChart } from 'layerchart';
	import { scaleBand } from 'd3-scale';

	let { data } = $props();

	// กำหนดสีและไอคอนสำหรับสถิติ (แบ่งตาม Admin Level)
	function getStatCards() {
		const isFacultyAdmin = data.admin_role?.admin_level === AdminLevel.FacultyAdmin;
		const stats = data.stats;

		if (isFacultyAdmin) {
			// สถิติเฉพาะคณะสำหรับ Faculty Admin
			return [
				{
					title: 'ผู้ใช้ในคณะ',
					value: stats?.faculty_users || 0,
					icon: IconUsers,
					color: 'text-blue-600',
					bgColor: 'bg-blue-100',
					description: 'จำนวนผู้ใช้ในคณะของคุณ'
				},
				{
					title: 'ภาควิชา',
					value: stats?.departments_count || 0,
					icon: IconBuilding,
					color: 'text-green-600',
					bgColor: 'bg-green-100',
					description: 'จำนวนภาควิชาในคณะ'
				},
				{
					title: 'ผู้ใช้ใหม่เดือนนี้',
					value: stats?.new_users_this_month || 0,
					icon: IconUsersGroup,
					color: 'text-purple-600',
					bgColor: 'bg-purple-100',
					description: 'ผู้ใช้ลงทะเบียนใหม่'
				},
				{
					title: 'ผู้ใช้ที่ใช้งาน',
					value: stats?.active_users || 0,
					icon: IconActivity,
					color: 'text-orange-600',
					bgColor: 'bg-orange-100',
					description: 'ผู้ใช้ที่เข้าใช้ใน 7 วันที่ผ่านมา'
				}
			];
		} else {
			// สถิติระบบทั้งหมดสำหรับ Super Admin
			return [
				{
					title: 'ผู้ใช้ทั้งหมด',
					value: stats?.total_users || 0,
					icon: IconUsers,
					color: 'text-blue-600',
					bgColor: 'bg-blue-100',
					description: 'จำนวนผู้ใช้ในระบบ'
				},
				{
					title: 'กิจกรรมทั้งหมด',
					value: stats?.total_activities || 0,
					icon: IconShield,
					color: 'text-green-600',
					bgColor: 'bg-green-100',
					description: 'จำนวนกิจกรรมในระบบ'
				},
				{
					title: 'การเข้าร่วม',
					value: stats?.total_participations || 0,
					icon: IconBuilding,
					color: 'text-purple-600',
					bgColor: 'bg-purple-100',
					description: 'จำนวนการเข้าร่วมกิจกรรม'
				},
				{
					title: 'เซสชันที่ใช้งาน',
					value: stats?.active_sessions || 0,
					icon: IconActivity,
					color: 'text-orange-600',
					bgColor: 'bg-orange-100',
					description: 'กิจกรรมใน 24 ชั่วโมงที่ผ่านมา'
				}
			];
		}
	}

	// Use $derived for reactive statements in runes mode
	const statCards = $derived(getStatCards());

	// Chart data for department breakdown (Faculty Admin)
	const departmentChartData = $derived(data.stats?.department_breakdown?.map((dept: any) => ({
		department: dept.name?.substring(0, 20) + (dept.name?.length > 20 ? '...' : '') || 'ไม่ระบุ',
		users: dept.user_count || 0,
		color: 'var(--chart-1)'
	})) || []);

	// Chart configuration
	const chartConfig = {
		users: {
			label: "จำนวนผู้ใช้",
			color: "var(--chart-1)",
		},
	} satisfies Chart.ChartConfig;

	function getAdminLevelText(level: AdminLevel | undefined): string {
		if (!level) return 'ไม่ระบุ';
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

	function getAdminLevelBadgeVariant(level: AdminLevel | undefined): 'default' | 'secondary' | 'destructive' | 'outline' {
		if (!level) return 'outline';
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

	function formatDateTime(dateString: string): string {
		return new Date(dateString).toLocaleString('th-TH');
	}
</script>

<svelte:head>
	<title>แดชบอร์ด - Admin Panel</title>
</svelte:head>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex flex-col sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 dark:text-white">
				แดชบอร์ด
				{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin && data.admin_role?.faculty}
					- {data.admin_role.faculty.name}
				{/if}
			</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				ยินดีต้อนรับ, {data.user.first_name} {data.user.last_name}
				{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin && data.admin_role?.faculty}
					<span class="ml-2 text-sm bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-2 py-1 rounded-full">
						<IconSchool class="inline h-4 w-4 mr-1" />
						{data.admin_role.faculty.code || data.admin_role.faculty.name}
					</span>
				{/if}
			</p>
		</div>
		<div class="mt-4 sm:mt-0">
			<Badge variant={getAdminLevelBadgeVariant(data.admin_role?.admin_level)}>
				{getAdminLevelText(data.admin_role?.admin_level)}
			</Badge>
		</div>
	</div>

	<!-- Faculty Info Card for Faculty Admin -->
	{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin && data.admin_role?.faculty}
		<Card class="border-blue-200 dark:border-blue-800 bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-950 dark:to-indigo-950">
			<CardHeader>
				<CardTitle class="flex items-center gap-2 text-blue-800 dark:text-blue-200">
					<IconSchool class="h-5 w-5" />
					ข้อมูลคณะ
				</CardTitle>
				<CardDescription>
					ข้อมูลสรุปของคณะที่คุณดูแล
				</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4">
					<div class="text-center">
						<div class="text-2xl font-bold text-blue-700 dark:text-blue-300">
							{data.admin_role.faculty.name}
						</div>
						<div class="text-sm text-gray-600 dark:text-gray-400">
							ชื่อคณะ
						</div>
					</div>
					<div class="text-center">
						<div class="text-2xl font-bold text-green-700 dark:text-green-300">
							{data.stats?.departments_count || 0}
						</div>
						<div class="text-sm text-gray-600 dark:text-gray-400">
							ภาควิชา
						</div>
					</div>
					<div class="text-center">
						<div class="text-2xl font-bold text-purple-700 dark:text-purple-300">
							{data.stats?.faculty_users || 0}
						</div>
						<div class="text-sm text-gray-600 dark:text-gray-400">
							ผู้ใช้ทั้งหมด
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
		{#each statCards as stat}
			<Card class="hover:shadow-lg transition-shadow duration-200">
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium text-gray-600 dark:text-gray-400">
						{stat.title}
					</CardTitle>
					<div class="p-2 rounded-lg {stat.bgColor} dark:bg-opacity-20">
						<stat.icon class="h-4 w-4 {stat.color}" />
					</div>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-gray-900 dark:text-white">
						{(stat.value || 0).toLocaleString()}
					</div>
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
						{stat.description}
					</p>
				</CardContent>
			</Card>
		{/each}
	</div>

	<!-- Department Analytics for Faculty Admin -->
	{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin && departmentChartData?.length > 0}
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<!-- Department Distribution Chart -->
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<IconChartBar class="h-5 w-5" />
						การกระจายผู้ใช้ตามภาควิชา
					</CardTitle>
					<CardDescription>
						จำนวนผู้ใช้ในแต่ละภาควิชาของคณะ
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Chart.Container config={chartConfig} class="min-h-[200px] w-full">
						<BarChart
							data={departmentChartData}
							xScale={scaleBand().padding(0.25)}
							x="department"
							y="users"
							axis="x"
							props={{
								xAxis: {
									format: (d) => d.length > 15 ? d.substring(0, 15) + '...' : d,
								},
							}}
						>
							{#snippet tooltip()}
								<Chart.Tooltip />
							{/snippet}
						</BarChart>
					</Chart.Container>
				</CardContent>
			</Card>

			<!-- Faculty Performance Metrics -->
			<Card>
				<CardHeader>
					<CardTitle class="flex items-center gap-2">
						<IconAward class="h-5 w-5" />
						ประสิทธิภาพคณะ
					</CardTitle>
					<CardDescription>
						ตัวชี้วัดและเป้าหมายของคณะ
					</CardDescription>
				</CardHeader>
				<CardContent class="space-y-4">
					<div>
						<div class="flex justify-between items-center mb-2">
							<span class="text-sm font-medium">อัตราผู้ใช้งานใหม่</span>
							<span class="text-sm text-gray-500">
								{Math.round(((data.stats?.new_users_this_month || 0) / Math.max(data.stats?.faculty_users || 1, 1)) * 100)}%
							</span>
						</div>
						<Progress value={Math.round(((data.stats?.new_users_this_month || 0) / Math.max(data.stats?.faculty_users || 1, 1)) * 100)} class="h-2" />
					</div>
					<div>
						<div class="flex justify-between items-center mb-2">
							<span class="text-sm font-medium">อัตราผู้ใช้งานใช้งาน</span>
							<span class="text-sm text-gray-500">
								{Math.round(((data.stats?.active_users || 0) / Math.max(data.stats?.faculty_users || 1, 1)) * 100)}%
							</span>
						</div>
						<Progress value={Math.round(((data.stats?.active_users || 0) / Math.max(data.stats?.faculty_users || 1, 1)) * 100)} class="h-2" />
					</div>
					<div>
						<div class="flex justify-between items-center mb-2">
							<span class="text-sm font-medium">ความครบถ้วนข้อมูลภาควิชา</span>
							<span class="text-sm text-gray-500">85%</span>
						</div>
						<Progress value={85} class="h-2" />
					</div>
					<div class="grid grid-cols-2 gap-4 mt-4">
						<div class="text-center p-3 bg-green-50 dark:bg-green-950 rounded-lg">
							<IconShieldCheck class="h-6 w-6 text-green-600 dark:text-green-400 mx-auto mb-1" />
							<div class="text-sm font-medium text-green-800 dark:text-green-200">เป้าหมายสำเร็จ</div>
							<div class="text-xs text-green-600 dark:text-green-400">3 จาก 4 เป้าหมาย</div>
						</div>
						<div class="text-center p-3 bg-blue-50 dark:bg-blue-950 rounded-lg">
							<IconTrendingUp class="h-6 w-6 text-blue-600 dark:text-blue-400 mx-auto mb-1" />
							<div class="text-sm font-medium text-blue-800 dark:text-blue-200">แนวโน้มดี</div>
							<div class="text-xs text-blue-600 dark:text-blue-400">+12% จากเดือนที่แล้ว</div>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	{/if}

	<!-- Quick Actions & Recent Activities -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<!-- Quick Actions -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<IconChartBar class="h-5 w-5" />
					การดำเนินการด่วน
				</CardTitle>
				<CardDescription>
					ฟังก์ชันที่ใช้บ่อยสำหรับการจัดการระบบ
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				{#if data.admin_role?.admin_level === AdminLevel.SuperAdmin}
					<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-blue-50 dark:hover:bg-blue-950" onclick={() => window.location.href = '/admin/faculty-users'}>
							<IconUsers class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการผู้ใช้</div>
								<div class="text-xs text-gray-500">เพิ่ม แก้ไข ลบผู้ใช้</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-green-50 dark:hover:bg-green-950" onclick={() => window.location.href = '/admin/admins'}>
							<IconShield class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการแอดมิน</div>
								<div class="text-xs text-gray-500">กำหนดสิทธิ์ผู้ดูแล</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-purple-50 dark:hover:bg-purple-950" onclick={() => window.location.href = '/admin/faculties'}>
							<IconBuilding class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการคณะ</div>
								<div class="text-xs text-gray-500">เพิ่ม แก้ไขข้อมูลคณะ</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-orange-50 dark:hover:bg-orange-950">
							<IconActivity class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">รายงานระบบ</div>
								<div class="text-xs text-gray-500">ดูสถิติและรายงาน</div>
							</div>
						</Button>
					</div>
				{:else if data.admin_role?.admin_level === AdminLevel.FacultyAdmin}
					<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-blue-50 dark:hover:bg-blue-950" onclick={() => window.location.href = '/admin/faculty-users'}>
							<IconUsers class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการผู้ใช้คณะ</div>
								<div class="text-xs text-gray-500">จัดการผู้ใช้ในคณะของคุณ</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-green-50 dark:hover:bg-green-950" onclick={() => window.location.href = '/admin/departments'}>
							<IconBuilding class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการภาควิชา</div>
								<div class="text-xs text-gray-500">เพิ่ม แก้ไข ภาควิชาในคณะ</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-purple-50 dark:hover:bg-purple-950">
							<IconChartBar class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">รายงานคณะ</div>
								<div class="text-xs text-gray-500">สถิติและรายงานของคณะ</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3 hover:bg-indigo-50 dark:hover:bg-indigo-950">
							<IconBook class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการหลักสูตร</div>
								<div class="text-xs text-gray-500">ดูและจัดการหลักสูตร</div>
							</div>
						</Button>
					</div>
				{:else}
					<div class="text-center py-4 text-gray-500 dark:text-gray-400">
						<IconActivity class="h-8 w-8 mx-auto mb-2 opacity-50" />
						<p>การดำเนินการจะแสดงตามสิทธิ์ของคุณ</p>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Recent Activities -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<IconClock class="h-5 w-5" />
					กิจกรรมล่าสุด
					{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin}
						<span class="text-sm font-normal text-gray-500">(คณะ{data.admin_role?.faculty?.name})</span>
					{/if}
				</CardTitle>
				<CardDescription>
					{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin}
						กิจกรรมและการเปลี่ยนแปลงล่าสุดในคณะของคุณ
					{:else}
						กิจกรรมและการเปลี่ยนแปลงล่าสุดในระบบ
					{/if}
				</CardDescription>
			</CardHeader>
			<CardContent>
				{#if data.recentActivities && data.recentActivities.length > 0}
					<div class="space-y-4">
						{#each data.recentActivities.slice(0, 5) as activity}
							<div class="flex items-start space-x-3 p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors duration-200">
								<div class="flex-shrink-0">
									{#if activity.type === 'success' || activity.action === 'login' || activity.action === 'profile_update'}
										<IconCircleCheck class="h-5 w-5 text-green-500" />
									{:else if activity.type === 'warning' || activity.action === 'status_change'}
										<IconAlertCircle class="h-5 w-5 text-yellow-500" />
									{:else if activity.action === 'logout'}
										<IconUsers class="h-5 w-5 text-gray-500" />
									{:else}
										<IconActivity class="h-5 w-5 text-blue-500" />
									{/if}
								</div>
								<div class="flex-1 min-w-0">
									<p class="text-sm font-medium text-gray-900 dark:text-white">
										{activity.title || activity.description || 'กิจกรรมในระบบ'}
									</p>
									{#if activity.user_name}
										<p class="text-xs text-gray-500 dark:text-gray-400">
											โดย: {activity.user_name}
											{#if activity.department_name}
												- {activity.department_name}
											{/if}
										</p>
									{/if}
									<div class="flex items-center justify-between mt-1">
										<p class="text-xs text-gray-400 dark:text-gray-500">
											{formatDateTime(activity.created_at)}
										</p>
										{#if activity.faculty_name && data.admin_role?.admin_level === AdminLevel.SuperAdmin}
											<span class="text-xs bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300 px-2 py-1 rounded-full">
												{activity.faculty_name}
											</span>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
					<Separator class="my-4" />
					<Button variant="link" size="sm" class="w-full">
						ดูกิจกรรมทั้งหมด
						<IconChartBar class="h-4 w-4 ml-2" />
					</Button>
				{:else}
					<div class="text-center py-8 text-gray-500 dark:text-gray-400">
						<IconClock class="h-8 w-8 mx-auto mb-2 opacity-50" />
						<p>ยังไม่มีกิจกรรมล่าสุด</p>
						{#if data.admin_role?.admin_level === AdminLevel.FacultyAdmin}
							<p class="text-xs mt-2">กิจกรรมจะแสดงเฉพาะในคณะของคุณ</p>
						{/if}
					</div>
				{/if}
			</CardContent>
		</Card>
	</div>

	<!-- System Info -->
	{#if data.admin_role?.admin_level === AdminLevel.SuperAdmin}
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<IconTrendingUp class="h-5 w-5" />
					ข้อมูลระบบ
				</CardTitle>
				<CardDescription>
					สถานะและข้อมูลทั่วไปของระบบ
				</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
					<div class="text-center">
						<div class="text-2xl font-bold text-green-600 dark:text-green-400">
							99.9%
						</div>
						<div class="text-sm text-gray-500 dark:text-gray-400">
							Uptime
						</div>
					</div>
					<div class="text-center">
						<div class="text-2xl font-bold text-blue-600 dark:text-blue-400">
							{((data.stats?.recent_activities?.length || 0) / 24).toFixed(1)}
						</div>
						<div class="text-sm text-gray-500 dark:text-gray-400">
							กิจกรรมต่อชั่วโมง
						</div>
					</div>
					<div class="text-center">
						<div class="text-2xl font-bold text-purple-600 dark:text-purple-400">
							{data.stats?.user_registrations_today || 0}
						</div>
						<div class="text-sm text-gray-500 dark:text-gray-400">
							ผู้ใช้ลงทะเบียนวันนี้
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}
</div>