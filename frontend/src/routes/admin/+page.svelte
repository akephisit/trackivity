<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { 
		Users, 
		Shield, 
		Building, 
		Activity,
		TrendingUp,
		Clock,
		CheckCircle,
		AlertCircle,
		BarChart3
	} from 'lucide-svelte';
	import { AdminLevel } from '$lib/types/admin';

	let { data } = $props();

	// กำหนดสีและไอคอนสำหรับสถิติ
	const statCards = [
		{
			title: 'ผู้ใช้ทั้งหมด',
			value: data.stats.total_users,
			icon: Users,
			color: 'text-blue-600',
			bgColor: 'bg-blue-100',
			description: 'จำนวนผู้ใช้ในระบบ'
		},
		{
			title: 'แอดมิน',
			value: data.stats.total_admins,
			icon: Shield,
			color: 'text-green-600',
			bgColor: 'bg-green-100',
			description: 'จำนวนผู้ดูแลระบบ'
		},
		{
			title: 'คณะ',
			value: data.stats.total_faculties,
			icon: Building,
			color: 'text-purple-600',
			bgColor: 'bg-purple-100',
			description: 'จำนวนคณะในระบบ'
		},
		{
			title: 'กิจกรรมล่าสุด',
			value: data.stats.recent_activities,
			icon: Activity,
			color: 'text-orange-600',
			bgColor: 'bg-orange-100',
			description: 'กิจกรรมใน 24 ชั่วโมงที่ผ่านมา'
		}
	];

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
			</h1>
			<p class="mt-2 text-gray-600 dark:text-gray-400">
				ยินดีต้อนรับ, {data.user.name}
			</p>
		</div>
		<div class="mt-4 sm:mt-0">
			<Badge variant={getAdminLevelBadgeVariant(data.admin_role?.admin_level)}>
				{getAdminLevelText(data.admin_role?.admin_level)}
			</Badge>
		</div>
	</div>

	<!-- Stats Cards -->
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
		{#each statCards as stat}
			<Card>
				<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
					<CardTitle class="text-sm font-medium text-gray-600 dark:text-gray-400">
						{stat.title}
					</CardTitle>
					<div class="p-2 rounded-lg {stat.bgColor} dark:bg-opacity-20">
						<svelte:component this={stat.icon} class="h-4 w-4 {stat.color}" />
					</div>
				</CardHeader>
				<CardContent>
					<div class="text-2xl font-bold text-gray-900 dark:text-white">
						{stat.value.toLocaleString()}
					</div>
					<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
						{stat.description}
					</p>
				</CardContent>
			</Card>
		{/each}
	</div>

	<!-- Quick Actions & Recent Activities -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<!-- Quick Actions -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<BarChart3 class="h-5 w-5" />
					การดำเนินการด่วน
				</CardTitle>
				<CardDescription>
					ฟังก์ชันที่ใช้บ่อยสำหรับการจัดการระบบ
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				{#if data.admin_role?.admin_level === AdminLevel.SuperAdmin}
					<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
						<Button variant="outline" class="justify-start h-auto py-3">
							<Users class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการผู้ใช้</div>
								<div class="text-xs text-gray-500">เพิ่ม แก้ไข ลบผู้ใช้</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3">
							<Shield class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการแอดมิน</div>
								<div class="text-xs text-gray-500">กำหนดสิทธิ์ผู้ดูแล</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3">
							<Building class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการคณะ</div>
								<div class="text-xs text-gray-500">เพิ่ม แก้ไขข้อมูลคณะ</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3">
							<Activity class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">รายงานระบบ</div>
								<div class="text-xs text-gray-500">ดูสถิติและรายงาน</div>
							</div>
						</Button>
					</div>
				{:else if data.admin_role?.admin_level === AdminLevel.FacultyAdmin}
					<div class="grid grid-cols-1 gap-3">
						<Button variant="outline" class="justify-start h-auto py-3">
							<Users class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">จัดการผู้ใช้คณะ</div>
								<div class="text-xs text-gray-500">จัดการผู้ใช้ในคณะของคุณ</div>
							</div>
						</Button>
						<Button variant="outline" class="justify-start h-auto py-3">
							<Activity class="h-4 w-4 mr-2" />
							<div class="text-left">
								<div class="font-medium">รายงานคณะ</div>
								<div class="text-xs text-gray-500">สถิติและรายงานของคณะ</div>
							</div>
						</Button>
					</div>
				{:else}
					<div class="text-center py-4 text-gray-500 dark:text-gray-400">
						<Activity class="h-8 w-8 mx-auto mb-2 opacity-50" />
						<p>การดำเนินการจะแสดงตามสิทธิ์ของคุณ</p>
					</div>
				{/if}
			</CardContent>
		</Card>

		<!-- Recent Activities -->
		<Card>
			<CardHeader>
				<CardTitle class="flex items-center gap-2">
					<Clock class="h-5 w-5" />
					กิจกรรมล่าสุด
				</CardTitle>
				<CardDescription>
					กิจกรรมและการเปลี่ยนแปลงล่าสุดในระบบ
				</CardDescription>
			</CardHeader>
			<CardContent>
				{#if data.recentActivities && data.recentActivities.length > 0}
					<div class="space-y-4">
						{#each data.recentActivities.slice(0, 5) as activity}
							<div class="flex items-start space-x-3">
								<div class="flex-shrink-0">
									{#if activity.type === 'success'}
										<CheckCircle class="h-5 w-5 text-green-500" />
									{:else if activity.type === 'warning'}
										<AlertCircle class="h-5 w-5 text-yellow-500" />
									{:else}
										<Activity class="h-5 w-5 text-blue-500" />
									{/if}
								</div>
								<div class="flex-1 min-w-0">
									<p class="text-sm font-medium text-gray-900 dark:text-white">
										{activity.title || 'กิจกรรมในระบบ'}
									</p>
									<p class="text-xs text-gray-500 dark:text-gray-400">
										{activity.description || 'ไม่มีรายละเอียด'}
									</p>
									<p class="text-xs text-gray-400 dark:text-gray-500 mt-1">
										{formatDateTime(activity.created_at)}
									</p>
								</div>
							</div>
						{/each}
					</div>
					<Separator class="my-4" />
					<Button variant="link" size="sm" class="w-full">
						ดูกิจกรรมทั้งหมด
					</Button>
				{:else}
					<div class="text-center py-8 text-gray-500 dark:text-gray-400">
						<Clock class="h-8 w-8 mx-auto mb-2 opacity-50" />
						<p>ยังไม่มีกิจกรรมล่าสุด</p>
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
					<TrendingUp class="h-5 w-5" />
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
							{(data.stats.recent_activities / 24).toFixed(1)}
						</div>
						<div class="text-sm text-gray-500 dark:text-gray-400">
							กิจกรรมต่อชั่วโมง
						</div>
					</div>
					<div class="text-center">
						<div class="text-2xl font-bold text-purple-600 dark:text-purple-400">
							{Math.round(data.stats.total_users / data.stats.total_faculties || 0)}
						</div>
						<div class="text-sm text-gray-500 dark:text-gray-400">
							ผู้ใช้เฉลี่ยต่อคณะ
						</div>
					</div>
				</div>
			</CardContent>
		</Card>
	{/if}
</div>