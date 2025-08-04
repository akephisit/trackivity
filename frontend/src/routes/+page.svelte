<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
</script>

<svelte:head>
	<title>Trackivity - University Activity Tracking System</title>
	<meta name="description" content="Modern university activity tracking system built with SvelteKit and Rust" />
</svelte:head>

<div class="container mx-auto px-4 py-8">
	<header class="text-center mb-12">
		<h1 class="text-4xl font-bold mb-4">Trackivity</h1>
		<p class="text-lg text-muted-foreground mb-8">
			University Activity Tracking System
		</p>
		<div class="flex gap-4 justify-center">
			<Button href="/auth/login">เข้าสู่ระบบ</Button>
			<Button variant="outline" href="/auth/register">สมัครสมาชิก</Button>
		</div>
	</header>

	<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
		<Card>
			<CardHeader>
				<CardTitle>กิจกรรมมหาวิทยาลัย</CardTitle>
				<CardDescription>
					ติดตามและเข้าร่วมกิจกรรมต่างๆ ภายในมหาวิทยาลัย
				</CardDescription>
			</CardHeader>
			<CardContent>
				<p class="text-sm text-muted-foreground">
					ระบบจัดการกิจกรรมที่ทันสมัยและใช้งานง่าย
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>QR Code Check-in</CardTitle>
				<CardDescription>
					เช็คอินเข้าร่วมกิจกรรมด้วย QR Code
				</CardDescription>
			</CardHeader>
			<CardContent>
				<p class="text-sm text-muted-foreground">
					สะดวก รวดเร็ว และแม่นยำ
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader>
				<CardTitle>จัดการแบบ Real-time</CardTitle>
				<CardDescription>
					ข้อมูลอัพเดทแบบเรียลไทม์
				</CardDescription>
			</CardHeader>
			<CardContent>
				<p class="text-sm text-muted-foreground">
					เห็นผลการเข้าร่วมกิจกรรมทันที
				</p>
			</CardContent>
		</Card>
	</div>
</div>