<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { authStore } from '$lib/stores/auth';
</script>

<div class="flex-1 space-y-4 p-4 pt-6">
	<div class="flex items-center justify-between space-y-2">
		<h2 class="text-3xl font-bold tracking-tight">แดชบอร์ด</h2>
		<div class="flex items-center space-x-2">
			<Button href="/dashboard/activities/create">สร้างกิจกรรมใหม่</Button>
		</div>
	</div>

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">กิจกรรมทั้งหมด</Card.Title>
				<svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/>
				</svg>
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">12</div>
				<p class="text-xs text-muted-foreground">+2 จากเดือนที่แล้ว</p>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">ผู้เข้าร่วมทั้งหมด</Card.Title>
				<svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
				</svg>
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">2,345</div>
				<p class="text-xs text-muted-foreground">+15% จากเดือนที่แล้ว</p>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">กิจกรรมที่กำลังดำเนินการ</Card.Title>
				<svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
				</svg>
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">3</div>
				<p class="text-xs text-muted-foreground">อีก 2 กิจกรรมในสัปดาห์นี้</p>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
				<Card.Title class="text-sm font-medium">อัตราการเข้าร่วม</Card.Title>
				<svg class="h-4 w-4 text-muted-foreground" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
				</svg>
			</Card.Header>
			<Card.Content>
				<div class="text-2xl font-bold">87.3%</div>
				<p class="text-xs text-muted-foreground">+3.2% จากเดือนที่แล้ว</p>
			</Card.Content>
		</Card.Root>
	</div>

	<!-- Recent Activities -->
	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
		<Card.Root class="col-span-4">
			<Card.Header>
				<Card.Title>กิจกรรมล่าสุด</Card.Title>
			</Card.Header>
			<Card.Content class="pl-2">
				<div class="space-y-8">
					<div class="flex items-center">
						<div class="w-2 h-2 bg-primary rounded-full mr-4"></div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium leading-none">สัมมนาวิชาการ AI</p>
							<p class="text-sm text-muted-foreground">วันที่ 15 ม.ค. 2568 • 150 คน</p>
						</div>
						<div class="text-sm text-muted-foreground">เสร็จสิ้น</div>
					</div>
					<div class="flex items-center">
						<div class="w-2 h-2 bg-green-500 rounded-full mr-4"></div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium leading-none">ประชุมนิสิต</p>
							<p class="text-sm text-muted-foreground">วันที่ 18 ม.ค. 2568 • 89 คน</p>
						</div>
						<div class="text-sm text-green-600">กำลังดำเนินการ</div>
					</div>
					<div class="flex items-center">
						<div class="w-2 h-2 bg-yellow-500 rounded-full mr-4"></div>
						<div class="flex-1 space-y-1">
							<p class="text-sm font-medium leading-none">กิจกรรมจิตอาสา</p>
							<p class="text-sm text-muted-foreground">วันที่ 22 ม.ค. 2568 • รอลงทะเบียน</p>
						</div>
						<div class="text-sm text-yellow-600">รอดำเนินการ</div>
					</div>
				</div>
			</Card.Content>
		</Card.Root>

		<Card.Root class="col-span-3">
			<Card.Header>
				<Card.Title>การเข้าร่วมของฉัน</Card.Title>
				<Card.Description>
					สถิติการเข้าร่วมกิจกรรมของคุณ
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="space-y-4">
					<div class="flex items-center justify-between">
						<span class="text-sm">กิจกรรมที่เข้าร่วม</span>
						<span class="text-2xl font-bold">8</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-sm">ชั่วโมงที่เข้าร่วม</span>
						<span class="text-2xl font-bold">32</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-sm">กิจกรรมที่กำลังจะมา</span>
						<span class="text-2xl font-bold">2</span>
					</div>
					<Button class="w-full" href="/dashboard/participations">
						ดูรายละเอียดเพิ่มเติม
					</Button>
				</div>
			</Card.Content>
		</Card.Root>
	</div>
</div>