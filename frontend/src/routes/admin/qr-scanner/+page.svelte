<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { toast } from 'svelte-sonner';
  
  import QRScanner from '$lib/components/qr/QRScanner.svelte';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Separator } from '$lib/components/ui/separator';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  
  import { 
    IconQrcode, 
    IconActivity, 
    IconUsers, 
    IconMapPin, 
    IconCalendar,
    IconClock,
    IconCheck,
    IconX,
    IconRefresh,
    IconSettings,
    IconArrowBack
  } from '@tabler/icons-svelte';

  import type { PageData } from './$types';

  export let data: PageData;

  // Component state
  let selectedActivity: any = null;
  let selectedActivityId = data.selectedActivityId;
  let scannerActive = false;
  let scannerStatus: 'idle' | 'requesting' | 'active' | 'error' = 'idle';
  let totalScanned = 0;
  let sessionStats = {
    successful: 0,
    failed: 0,
    startTime: null as Date | null
  };

  // Reactive statements
  $: {
    if (selectedActivityId) {
      selectedActivity = data.activities.find((a: any) => a.id === selectedActivityId);
      // Update URL when activity changes  
      const url = new URL(window.location.href);
      if (url.searchParams.get('activity_id') !== selectedActivityId) {
        url.searchParams.set('activity_id', selectedActivityId);
        goto(url.toString(), { replaceState: true, noScroll: true });
      }
    } else {
      selectedActivity = null;
    }
  }

  onMount(() => {
    if (data.selectedActivityId && data.activities.length > 0) {
      selectedActivityId = data.selectedActivityId;
    } else if (data.activities.length === 1) {
      // Auto-select if only one activity
      selectedActivityId = data.activities[0].id;
    }
  });

  function handleActivityChange(activityId: string) {
    selectedActivityId = activityId;
    // Stop scanner when changing activities
    if (scannerActive) {
      scannerActive = false;
    }
  }

  function startScanning() {
    if (!selectedActivityId) {
      toast.error('กรุณาเลือกกิจกรรมก่อนเริ่มสแกน');
      return;
    }
    
    scannerActive = true;
    sessionStats.startTime = new Date();
    toast.success('เริ่มระบบสแกน QR Code');
  }

  function stopScanning() {
    scannerActive = false;
    toast.info('หยุดระบบสแกน QR Code');
  }

  function handleScanResult(result: any, _qrData: string) {
    totalScanned++;
    
    if (result.success) {
      sessionStats.successful++;
      toast.success(`สแกนสำเร็จ: ${result.user_name}`);
    } else {
      sessionStats.failed++;
      toast.error(`สแกนไม่สำเร็จ: ${result.message}`);
    }
  }

  function handleScanError(message: string) {
    sessionStats.failed++;
    toast.error(`เกิดข้อผิดพลาด: ${message}`);
  }

  function handleStatusChange(status: typeof scannerStatus) {
    scannerStatus = status;
  }

  function resetStats() {
    sessionStats = {
      successful: 0,
      failed: 0,
      startTime: new Date()
    };
    totalScanned = 0;
    toast.success('รีเซ็ตสถิติเรียบร้อย');
  }

  function formatDateTime(date: string | Date): string {
    try {
      const d = typeof date === 'string' ? new Date(date) : date;
      return d.toLocaleString('th-TH', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return 'ไม่ระบุ';
    }
  }

  function formatDuration(start: Date): string {
    const now = new Date();
    const diff = now.getTime() - start.getTime();
    const minutes = Math.floor(diff / 60000);
    const seconds = Math.floor((diff % 60000) / 1000);
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  }

  function getSuccessRate(): number {
    if (totalScanned === 0) return 0;
    return Math.round((sessionStats.successful / totalScanned) * 100);
  }
</script>

<svelte:head>
  <title>QR Scanner - Admin Dashboard</title>
  <meta name="description" content="ระบบสแกน QR Code สำหรับผู้ดูแลระบบ" />
</svelte:head>

<div class="container mx-auto p-4 space-y-6 max-w-4xl">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold flex items-center gap-2">
        <IconQrcode class="size-6" />
        ระบบสแกน QR Code
      </h1>
      <p class="text-muted-foreground">สแกน QR Code เพื่อบันทึกการเข้าร่วมกิจกรรม</p>
    </div>
    
    <Button variant="outline" onclick={() => goto('/admin')}>
      <IconArrowBack class="size-4 mr-2" />
      กลับหน้าหลัก
    </Button>
  </div>

  <!-- Admin Info -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <IconSettings class="size-5" />
        ข้อมูลผู้ดูแลระบบ
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div>
          <p class="text-sm text-muted-foreground">ชื่อ</p>
          <p class="font-medium">{data.admin.first_name} {data.admin.last_name}</p>
        </div>
        <div>
          <p class="text-sm text-muted-foreground">ระดับสิทธิ์</p>
          <Badge variant="outline">{data.admin.admin_level}</Badge>
        </div>
        <div>
          <p class="text-sm text-muted-foreground">คณะ</p>
          <p class="font-medium">{data.admin.faculty_name || 'ทั้งหมด'}</p>
        </div>
      </div>
    </CardContent>
  </Card>

  <!-- Activity Selection -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <IconActivity class="size-5" />
        เลือกกิจกรรม
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      {#if data.activities.length === 0}
        <Alert>
          <IconX class="h-4 w-4" />
          <AlertDescription>
            ไม่พบกิจกรรมที่กำลังดำเนินการ (ongoing) ที่สามารถสแกน QR Code ได้ กรุณาติดต่อผู้ดูแลระบบ
          </AlertDescription>
        </Alert>
      {:else}
        <div class="space-y-2">
          <label for="activity-select" class="text-sm font-medium">เลือกกิจกรรมที่ต้องการสแกน (เฉพาะกิจกรรมที่กำลังดำเนินการ):</label>
          <select 
            bind:value={selectedActivityId}
            on:change={(e) => handleActivityChange(e.currentTarget.value)}
            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            <option value="">เลือกกิจกรรม...</option>
            {#each data.activities as activity}
              <option value={activity.id}>{activity.title}</option>
            {/each}
          </select>
        </div>

        <!-- Selected Activity Info -->
        {#if selectedActivity}
          <div class="p-4 bg-muted/50 rounded-lg space-y-3">
            <h3 class="font-semibold flex items-center gap-2">
              <IconActivity class="size-5" />
              {selectedActivity.title}
            </h3>
            
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div class="flex items-center gap-2">
                <IconMapPin class="size-4 text-muted-foreground" />
                <span>{selectedActivity.location}</span>
              </div>
              
              <div class="flex items-center gap-2">
                <IconUsers class="size-4 text-muted-foreground" />
                <span>{selectedActivity.current_participants} / {selectedActivity.max_participants || 'ไม่จำกัด'} คน</span>
              </div>
              
              <div class="flex items-center gap-2">
                <IconCalendar class="size-4 text-muted-foreground" />
                <span>{formatDateTime(selectedActivity.start_time)}</span>
              </div>
              
              <div class="flex items-center gap-2">
                <Badge variant="default" class="bg-green-600 hover:bg-green-700">
                  <div class="flex items-center gap-1">
                    <div class="w-2 h-2 bg-white rounded-full animate-pulse"></div>
                    กำลังดำเนินการ
                  </div>
                </Badge>
                <Badge variant="outline" class="text-green-600 border-green-600">
                  สแกน QR ได้
                </Badge>
              </div>
            </div>
            
            {#if selectedActivity.description}
              <p class="text-sm text-muted-foreground">{selectedActivity.description}</p>
            {/if}
          </div>
        {/if}
      {/if}
    </CardContent>
  </Card>

  <!-- Session Statistics -->
  {#if sessionStats.startTime}
    <Card>
      <CardHeader>
        <CardTitle class="flex items-center gap-2">
          <IconClock class="size-5" />
          สถิติการสแกนในเซสชันนี้
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="text-center">
            <div class="text-2xl font-bold text-green-600">{sessionStats.successful}</div>
            <div class="text-sm text-muted-foreground">สำเร็จ</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold text-red-600">{sessionStats.failed}</div>
            <div class="text-sm text-muted-foreground">ไม่สำเร็จ</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold">{totalScanned}</div>
            <div class="text-sm text-muted-foreground">ทั้งหมด</div>
          </div>
          <div class="text-center">
            <div class="text-2xl font-bold">{getSuccessRate()}%</div>
            <div class="text-sm text-muted-foreground">อัตราสำเร็จ</div>
          </div>
        </div>
        
        <Separator class="my-4" />
        
        <div class="flex items-center justify-between text-sm">
          <span>เริ่มเซสชัน: {formatDateTime(sessionStats.startTime)}</span>
          <span>ระยะเวลา: {formatDuration(sessionStats.startTime)}</span>
        </div>
        
        <div class="mt-2">
          <Button onclick={resetStats} variant="outline" size="sm">
            <IconRefresh class="size-4 mr-2" />
            รีเซ็ตสถิติ
          </Button>
        </div>
      </CardContent>
    </Card>
  {/if}

  <!-- Scanner Control -->
  <div class="flex gap-4 justify-center">
    {#if !scannerActive}
      <Button 
        onclick={startScanning}
        disabled={!selectedActivityId}
        class="flex items-center gap-2"
      >
        <IconQrcode class="size-5" />
        เริ่มสแกน QR Code
      </Button>
    {:else}
      <Button 
        onclick={stopScanning}
        variant="outline"
        class="flex items-center gap-2"
      >
        <IconX class="size-5" />
        หยุดสแกน
      </Button>
    {/if}
  </div>

  <!-- QR Scanner -->
  {#if scannerActive && selectedActivityId}
    <QRScanner
      activity_id={selectedActivityId}
      isActive={scannerActive}
      onScan={handleScanResult}
      onError={handleScanError}
      onStatusChange={handleStatusChange}
      showHistory={true}
      maxHistoryItems={20}
    />
  {/if}

  <!-- Instructions -->
  <Card>
    <CardHeader>
      <CardTitle>คำแนะนำการใช้งาน</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-2 text-sm text-muted-foreground">
        <div class="flex items-start gap-2">
          <IconCheck class="size-4 mt-0.5 text-green-600" />
          <span>เลือกกิจกรรมที่มีสถานะ "กำลังดำเนินการ" เท่านั้น</span>
        </div>
        <div class="flex items-start gap-2">
          <IconCheck class="size-4 mt-0.5 text-green-600" />
          <span>กดปุ่ม "เริ่มสแกน QR Code" เพื่อเปิดกล้อง</span>
        </div>
        <div class="flex items-start gap-2">
          <IconCheck class="size-4 mt-0.5 text-green-600" />
          <span>วาง QR Code ของนักศึกษาในกรอบที่กำหนด</span>
        </div>
        <div class="flex items-start gap-2">
          <IconCheck class="size-4 mt-0.5 text-green-600" />
          <span>ระบบจะบันทึกการเข้าร่วมอัตโนมัติเมื่อตรวจพบ QR Code</span>
        </div>
        <div class="flex items-start gap-2">
          <IconCheck class="size-4 mt-0.5 text-green-600" />
          <span>ตรวจสอบสถิติและประวัติการสแกน - สามารถสแกนได้เฉพาะกิจกรรมที่กำลังดำเนินการ</span>
        </div>
      </div>
    </CardContent>
  </Card>
</div>