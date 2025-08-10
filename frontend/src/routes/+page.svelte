<script lang="ts">
	import ActivitiesTable from "$lib/components/activities-table.svelte";
	import { mockActivities } from "$lib/data/activities.js";
	import { Button } from "$lib/components/ui/button";
	import { IconUser } from '@tabler/icons-svelte/icons';
</script>

<div class="min-h-screen bg-background">
	<!-- Header -->
	<header class="border-b bg-card">
		<div class="container mx-auto px-4 py-6">
			<div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
				<div>
					<h1 class="text-3xl font-bold text-foreground">ระบบติดตามกิจกรรม</h1>
					<p class="text-muted-foreground mt-1">
						จัดการและติดตามกิจกรรมทั้งหมดของมหาวิทยาลัยในที่เดียว
					</p>
				</div>
				<div class="flex flex-col sm:flex-row items-end sm:items-center gap-3">
					<div class="text-sm text-muted-foreground">
						{new Date().toLocaleDateString('th-TH', {
							year: 'numeric',
							month: 'long',
							day: 'numeric',
							weekday: 'long'
						})}
					</div>
					<Button href="/login" variant="outline" size="sm" class="flex items-center gap-2">
						<IconUser class="h-4 w-4" />
						เข้าสู่ระบบ
					</Button>
				</div>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<main class="container mx-auto px-4 py-8">
		<ActivitiesTable activities={mockActivities} />
	</main>

	<!-- Footer -->
	<footer class="border-t bg-muted/30 mt-16">
		<div class="container mx-auto px-4 py-6">
			<div class="text-center text-sm text-muted-foreground">
				<p>© 2024 ระบบติดตามกิจกรรม - มหาวิทยาลัยเทคโนโลยีพระจอมเกล้าธนบุรี</p>
			</div>
		</div>
	</footer>
</div>
