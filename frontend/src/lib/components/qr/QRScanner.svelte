<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import { toast } from 'svelte-sonner';
  import jsQR from 'jsqr';
  
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Separator } from '$lib/components/ui/separator';
  
  import { 
    IconCamera, 
    IconCameraOff, 
    IconReload,
    IconAlertTriangle,
    IconCheck,
    IconQrcode,
    IconX,
    IconUser,
    IconClock
  } from '@tabler/icons-svelte';

  // Types
  interface ScanResult {
    success: boolean;
    message: string;
    user_name?: string;
    student_id?: string;
    participation_status?: string;
    checked_in_at?: string;
  }

  interface ScannedUser {
    user_name: string;
    student_id: string;
    participation_status: string;
    checked_in_at: string;
    timestamp: number;
  }

  // Props
  export let activity_id: string = '';
  export let isActive = false;
  export let showHistory = true;
  export let maxHistoryItems = 10;
  
  // Component state
  let videoElement: HTMLVideoElement;
  let stream: MediaStream | null = null;
  let isScanning = false;
  let error: string | null = null;
  let lastScanTime = 0;
  let scanCooldown = 2000; // 2 seconds between scans
  
  // Scanner state
  let cameraStatus: 'idle' | 'requesting' | 'active' | 'error' = 'idle';
  let scanHistory: ScannedUser[] = [];
  let isProcessingScan = false;
  
  // Event callbacks
  export let onScan: ((result: ScanResult, qrData: string) => void) | undefined = undefined;
  export let onError: ((message: string) => void) | undefined = undefined;
  export let onStatusChange: ((status: typeof cameraStatus) => void) | undefined = undefined;

  // Reactive statements
  $: if (browser && isActive && activity_id) {
    startCamera();
  } else if (browser && !isActive) {
    stopCamera();
  }

  onMount(() => {
    if (browser && isActive && activity_id) {
      startCamera();
    }
  });

  onDestroy(() => {
    stopCamera();
  });

  async function startCamera() {
    if (!browser || !activity_id) return;
    
    cameraStatus = 'requesting';
    error = null;
    onStatusChange?.(cameraStatus);
    
    try {
      // Request camera permissions
      stream = await navigator.mediaDevices.getUserMedia({ 
        video: { 
          facingMode: 'environment', // Prefer back camera
          width: { ideal: 1280 },
          height: { ideal: 720 }
        } 
      });
      
      if (videoElement) {
        videoElement.srcObject = stream;
        await videoElement.play();
        cameraStatus = 'active';
        isScanning = true;
        startQRDetection();
      }
    } catch (err) {
      console.error('Failed to start camera:', err);
      error = 'ไม่สามารถเข้าถึงกล้องได้ กรุณาอนุญาตการใช้งานกล้อง';
      cameraStatus = 'error';
      onError?.(error);
    }
    
    onStatusChange?.(cameraStatus);
  }

  function stopCamera() {
    if (stream) {
      stream.getTracks().forEach(track => track.stop());
      stream = null;
    }
    
    if (videoElement) {
      videoElement.srcObject = null;
    }
    
    isScanning = false;
    cameraStatus = 'idle';
    onStatusChange?.(cameraStatus);
  }

  async function startQRDetection() {
    if (!isScanning || !videoElement || cameraStatus !== 'active') return;

    try {
      // Use HTML5 QRCode library or create canvas-based detection
      await detectQRCode();
    } catch (err) {
      console.error('QR Detection error:', err);
    }

    // Continue scanning
    if (isScanning) {
      requestAnimationFrame(startQRDetection);
    }
  }

  async function detectQRCode() {
    if (!videoElement || isProcessingScan) return;

    // Create canvas to capture video frame
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    
    if (!ctx) return;

    canvas.width = videoElement.videoWidth;
    canvas.height = videoElement.videoHeight;
    
    // Draw current video frame to canvas
    ctx.drawImage(videoElement, 0, 0, canvas.width, canvas.height);
    
    // Get image data
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    
    // Use jsQR library for QR code detection (needs to be installed)
    try {
      // This would use a QR code detection library
      // For now, we'll simulate QR detection with manual input
      await handleQRDetection(imageData);
    } catch (err) {
      console.error('QR Code detection error:', err);
    }
  }

  async function handleQRDetection(imageData: ImageData) {
    // Use jsQR to detect QR codes in the image data
    try {
      const code = jsQR(imageData.data, imageData.width, imageData.height, {
        inversionAttempts: "dontInvert",
      });

      if (code) {
        // Check scan cooldown
        const now = Date.now();
        if (now - lastScanTime < scanCooldown) return;
        
        // Found QR code, process it
        await processQRCode(code.data);
      }
    } catch (err) {
      console.error('jsQR detection error:', err);
    }
  }

  async function processQRCode(qrData: string) {
    if (isProcessingScan) return;
    
    isProcessingScan = true;
    const now = Date.now();
    
    // Check cooldown
    if (now - lastScanTime < scanCooldown) {
      isProcessingScan = false;
      return;
    }
    
    lastScanTime = now;
    
    try {
      const response = await fetch(`/api/activities/${activity_id}/checkin`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify({ qr_data: qrData })
      });
      
      const result = await response.json();
      
      if (response.ok && (result.success === true)) {
        const scanResult: ScanResult = {
          success: true,
          message: result.message || 'สแกนสำเร็จ',
          user_name: result.data?.user_name,
          student_id: result.data?.student_id,
          participation_status: result.data?.participation_status,
          checked_in_at: result.data?.checked_in_at
        };
        
        // Add to history
        if (scanResult.user_name && scanResult.student_id) {
          const historyItem: ScannedUser = {
            user_name: scanResult.user_name,
            student_id: scanResult.student_id,
            participation_status: scanResult.participation_status || 'checked_in',
            checked_in_at: scanResult.checked_in_at || new Date().toISOString(),
            timestamp: now
          };
          
          scanHistory = [historyItem, ...scanHistory.slice(0, maxHistoryItems - 1)];
        }
        
        toast.success(`สแกนสำเร็จ: ${scanResult.user_name}`);
        onScan?.(scanResult, qrData);
      } else {
        const scanResult: ScanResult = {
          success: false,
          message: result.message || 'เกิดข้อผิดพลาดในการสแกน'
        };
        
        toast.error(scanResult.message);
        onScan?.(scanResult, qrData);
      }
    } catch (err) {
      console.error('Scan processing error:', err);
      const errorMessage = 'เกิดข้อผิดพลาดในการเชื่อมต่อ';
      toast.error(errorMessage);
      onError?.(errorMessage);
    } finally {
      isProcessingScan = false;
    }
  }

  // Manual scan trigger for testing/accessibility
  async function triggerManualScan() {
    // In development or for testing, allow manual QR data input
    const qrData = prompt('กรุณาป้อน QR Data สำหรับทดสอบ:');
    if (qrData) {
      await processQRCode(qrData);
    }
  }

  function clearHistory() {
    scanHistory = [];
    toast.success('ล้างประวัติการสแกนเรียบร้อย');
  }

  function formatDateTime(dateString: string): string {
    try {
      const date = new Date(dateString);
      return date.toLocaleString('th-TH', {
        year: '2-digit',
        month: '2-digit', 
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return 'ไม่ระบุ';
    }
  }

  function getStatusBadgeVariant(status: string): "default" | "secondary" | "destructive" | "outline" {
    switch (status.toLowerCase()) {
      case 'checked_in':
      case 'checkedin':
        return 'default';
      case 'registered':
        return 'secondary';
      default:
        return 'outline';
    }
  }

  function getStatusText(status: string): string {
    switch (status.toLowerCase()) {
      case 'checked_in':
      case 'checkedin':
        return 'เข้าร่วมแล้ว';
      case 'registered':
        return 'ลงทะเบียนแล้ว';
      default:
        return status;
    }
  }
</script>

<div class="space-y-4">
  <!-- Scanner Card -->
  <Card class="w-full">
    <CardHeader>
      <CardTitle class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <IconQrcode class="size-5" />
          QR Scanner
        </div>
        
        <div class="flex items-center gap-2">
          <Badge variant={cameraStatus === 'active' ? 'default' : 'secondary'}>
            {#if cameraStatus === 'requesting'}
              <IconCamera class="size-3 mr-1 animate-pulse" />
              กำลังเชื่อมต่อ...
            {:else if cameraStatus === 'active'}
              <IconCamera class="size-3 mr-1" />
              พร้อมสแกน
            {:else if cameraStatus === 'error'}
              <IconCameraOff class="size-3 mr-1" />
              ข้อผิดพลาด
            {:else}
              <IconCameraOff class="size-3 mr-1" />
              ปิด
            {/if}
          </Badge>
          
          {#if isProcessingScan}
            <Badge variant="secondary">
              <IconReload class="size-3 mr-1 animate-spin" />
              กำลังประมวลผล...
            </Badge>
          {/if}
        </div>
      </CardTitle>
    </CardHeader>

    <CardContent class="space-y-4">
      <!-- Camera Preview -->
      <div class="relative">
        <div class="aspect-video bg-muted rounded-lg overflow-hidden border-2 border-dashed flex items-center justify-center">
          {#if cameraStatus === 'active'}
            <video
              bind:this={videoElement}
              class="w-full h-full object-cover"
              playsinline
              muted
              autoplay
            ></video>
            
            <!-- Scanning overlay -->
            <div class="absolute inset-0 pointer-events-none">
              <!-- Scanning frame -->
              <div class="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2">
                <div class="w-48 h-48 border-2 border-primary rounded-lg relative">
                  <div class="absolute top-0 left-0 w-6 h-6 border-t-4 border-l-4 border-primary"></div>
                  <div class="absolute top-0 right-0 w-6 h-6 border-t-4 border-r-4 border-primary"></div>
                  <div class="absolute bottom-0 left-0 w-6 h-6 border-b-4 border-l-4 border-primary"></div>
                  <div class="absolute bottom-0 right-0 w-6 h-6 border-b-4 border-r-4 border-primary"></div>
                </div>
              </div>
              
              <!-- Instructions -->
              <div class="absolute bottom-4 left-1/2 transform -translate-x-1/2 bg-black/50 text-white px-3 py-1 rounded text-sm">
                วางกรอบให้อยู่บน QR Code
              </div>
            </div>
          {:else if cameraStatus === 'requesting'}
            <div class="text-center space-y-4">
              <Skeleton class="w-16 h-16 rounded-full mx-auto" />
              <div class="space-y-2">
                <Skeleton class="h-4 w-32 mx-auto" />
                <Skeleton class="h-3 w-24 mx-auto" />
              </div>
            </div>
          {:else if cameraStatus === 'error'}
            <div class="text-center space-y-3 text-muted-foreground">
              <IconCameraOff class="size-12 mx-auto text-destructive" />
              <div>
                <p class="font-medium text-destructive">ไม่สามารถเข้าถึงกล้องได้</p>
                <p class="text-sm">กรุณาอนุญาตการใช้งานกล้องและรีเฟรชหน้า</p>
              </div>
            </div>
          {:else}
            <div class="text-center space-y-3 text-muted-foreground">
              <IconCamera class="size-12 mx-auto" />
              <div>
                <p class="font-medium">เริ่มต้นการสแกน</p>
                <p class="text-sm">กดปุ่มเพื่อเปิดกล้อง</p>
              </div>
            </div>
          {/if}
        </div>
      </div>

      <!-- Error Alert -->
      {#if error}
        <Alert variant="destructive">
          <IconAlertTriangle class="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      {/if}

      <!-- Control Buttons -->
      <div class="flex gap-2 justify-center">
        {#if cameraStatus === 'idle' || cameraStatus === 'error'}
          <Button onclick={startCamera} disabled={!activity_id}>
            <IconCamera class="size-4 mr-2" />
            เริ่มสแกน
          </Button>
        {:else if cameraStatus === 'active'}
          <Button onclick={stopCamera} variant="outline">
            <IconCameraOff class="size-4 mr-2" />
            หยุดสแกน
          </Button>
        {/if}
        
        <!-- Development: Manual scan trigger -->
        {#if cameraStatus === 'active'}
          <Button onclick={triggerManualScan} variant="outline" size="sm">
            <IconQrcode class="size-4 mr-2" />
            สแกนด้วยตนเอง
          </Button>
        {/if}
      </div>

      <!-- Scanner Info -->
      <div class="text-xs text-muted-foreground text-center space-y-1">
        {#if !activity_id}
          <p class="text-destructive">กรุณาเลือกกิจกรรมก่อนเริ่มสแกน</p>
        {:else}
          <p>วาง QR Code ของนักศึกษาให้อยู่ในกรอบเพื่อสแกน</p>
          <p>ระบบจะประมวลผลอัตโนมัติเมื่อตรวจพบ QR Code</p>
        {/if}
      </div>
    </CardContent>
  </Card>

  <!-- Scan History -->
  {#if showHistory && scanHistory.length > 0}
    <Card>
      <CardHeader>
        <div class="flex items-center justify-between">
          <CardTitle class="flex items-center gap-2">
            <IconUser class="size-5" />
            ประวัติการสแกน
            <Badge variant="outline">{scanHistory.length}</Badge>
          </CardTitle>
          
          <Button onclick={clearHistory} variant="outline" size="sm">
            <IconX class="size-4 mr-2" />
            ล้างประวัติ
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        <div class="space-y-3">
          {#each scanHistory as item, index (item.timestamp)}
            <div class="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                  <IconCheck class="size-4 text-green-600" />
                  <span class="font-medium">{item.user_name}</span>
                  <Badge variant={getStatusBadgeVariant(item.participation_status)} class="text-xs">
                    {getStatusText(item.participation_status)}
                  </Badge>
                </div>
                <div class="flex items-center gap-4 text-sm text-muted-foreground">
                  <span>รหัส: {item.student_id}</span>
                  <div class="flex items-center gap-1">
                    <IconClock class="size-3" />
                    {formatDateTime(item.checked_in_at)}
                  </div>
                </div>
              </div>
            </div>
            
            {#if index < scanHistory.length - 1}
              <Separator />
            {/if}
          {/each}
        </div>
      </CardContent>
    </Card>
  {/if}
</div>
