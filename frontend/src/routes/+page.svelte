<script lang="ts">
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { authStore } from '$lib/stores/auth';
</script>

<div class="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
	<!-- Hero Section -->
	<div class="container mx-auto px-4 py-16">
		<div class="text-center mb-16">
			<h1 class="text-4xl md:text-6xl font-bold text-gray-900 dark:text-white mb-6">
				<span class="text-primary">Trackivity</span>
			</h1>
			<p class="text-xl md:text-2xl text-gray-600 dark:text-gray-300 mb-8 max-w-3xl mx-auto">
				ระบบเก็บกิจกรรมมหาวิทยาลัยที่ทันสมัย<br />
				จัดการกิจกรรม ลงทะเบียน และติดตามผลได้อย่างง่ายดาย
			</p>
			
			{#if $authStore.user}
				<div class="space-x-4">
					<Button href="/dashboard" size="lg" class="text-lg px-8 py-4">
						เข้าสู่แดชบอร์ด
					</Button>
				</div>
			{:else}
				<div class="space-x-4">
					<Button href="/auth/register" size="lg" class="text-lg px-8 py-4">
						เริ่มต้นใช้งาน
					</Button>
					<Button href="/auth/login" variant="outline" size="lg" class="text-lg px-8 py-4">
						เข้าสู่ระบบ
					</Button>
				</div>
			{/if}
		</div>

		<!-- Features Section -->
		<div class="grid md:grid-cols-3 gap-8 mb-16">
			<Card.Root class="text-center">
				<Card.Header>
					<div class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4">
						<svg class="w-8 h-8 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v2m0 0v4a2 2 0 002 2h2m0-8V5a2 2 0 012-2h2m0 0h2a2 2 0 012 2v2m0 4v2a2 2 0 01-2 2h-2m-4 0H9a2 2 0 01-2-2v-2m0-4V7"/>
						</svg>
					</div>
					<Card.Title>จัดการกิจกรรมง่าย</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-gray-600 dark:text-gray-300">
						สร้าง แก้ไข และจัดการกิจกรรมได้อย่างสะดวก พร้อมระบบ QR Code สำหรับการเช็คอิน
					</p>
				</Card.Content>
			</Card.Root>

			<Card.Root class="text-center">
				<Card.Header>
					<div class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4">
						<svg class="w-8 h-8 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
						</svg>
					</div>
					<Card.Title>ระบบผู้ใช้แบบหลายระดับ</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-gray-600 dark:text-gray-300">
						รองรับผู้ใช้หลายระดับ จาก Super Admin, Faculty Admin ถึง Regular Admin
					</p>
				</Card.Content>
			</Card.Root>

			<Card.Root class="text-center">
				<Card.Header>
					<div class="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4">
						<svg class="w-8 h-8 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/>
						</svg>
					</div>
					<Card.Title>รายงานแบบ Real-time</Card.Title>
				</Card.Header>
				<Card.Content>
					<p class="text-gray-600 dark:text-gray-300">
						ติดตามสถิติการเข้าร่วมกิจกรรมแบบเรียลไทม์ พร้อมรายงานที่ครบถ้วน
					</p>
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</div>