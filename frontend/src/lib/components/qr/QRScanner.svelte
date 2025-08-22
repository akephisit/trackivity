<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { browser } from '$app/environment';
  import { toast } from 'svelte-sonner';
  import jsQR from 'jsqr';
  
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
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
  let {
    activity_id = '',
    isActive = false,
    showHistory = true,
    maxHistoryItems = 10,
    onScan = undefined,
    onError = undefined,
    onStatusChange = undefined
  }: {
    activity_id?: string;
    isActive?: boolean;
    showHistory?: boolean;
    maxHistoryItems?: number;
    onScan?: ((result: ScanResult, qrData: string) => void) | undefined;
    onError?: ((message: string) => void) | undefined;
    onStatusChange?: ((status: 'idle' | 'requesting' | 'active' | 'error') => void) | undefined;
  } = $props();
  
  // Component state
  let videoElement = $state<HTMLVideoElement>();
  let stream = $state<MediaStream | null>(null);
  let isScanning = $state(false);
  let error = $state<string | null>(null);
  let lastScanTime = 0;
  let scanCooldown = 2000; // 2 seconds between scans
  let debugInfo = $state({
    hasCamera: false,
    cameraPermission: 'unknown',
    videoReady: false,
    streamActive: false
  });
  
  // Scanner state
  let cameraStatus = $state<'idle' | 'requesting' | 'active' | 'error'>('idle');
  let scanHistory = $state<ScannedUser[]>([]);
  let isProcessingScan = $state(false);
  

  // Reactive effect
  $effect(() => {
    if (browser && isActive && activity_id && cameraStatus === 'idle') {
      console.log('Effect: Starting camera due to state change');
      // Small delay to ensure DOM is ready
      setTimeout(() => {
        if (cameraStatus === 'idle') { // Check again in case status changed
          startCamera();
        }
      }, 100);
    } else if (browser && !isActive && cameraStatus !== 'idle') {
      console.log('Effect: Stopping camera due to state change');
      stopCamera();
    }
  });

  onMount(async () => {
    console.log('onMount: Component mounted', { isActive, activity_id, cameraStatus });
    
    if (browser) {
      // Check camera availability
      debugInfo.hasCamera = !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia);
      
      // Check camera permission if available
      if (navigator.permissions) {
        try {
          const permissionStatus = await navigator.permissions.query({ name: 'camera' as PermissionName });
          debugInfo.cameraPermission = permissionStatus.state;
        } catch (e) {
          console.log('Cannot check camera permission:', e);
        }
      }
      
      if (isActive && activity_id && cameraStatus === 'idle') {
        console.log('onMount: Starting camera');
        startCamera();
      }
    }
  });

  onDestroy(() => {
    stopCamera();
  });

  async function startCamera() {
    if (!browser || !activity_id) {
      console.log('startCamera: browser or activity_id not available');
      return;
    }
    
    // Prevent multiple simultaneous calls
    if (cameraStatus === 'requesting' || cameraStatus === 'active') {
      console.log('startCamera: Camera already starting/active, skipping');
      return;
    }
    
    console.log('startCamera: Starting camera...');
    
    // Check if getUserMedia is supported
    if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
      error = 'เบราว์เซอร์นี้ไม่รองรับการเข้าถึงกล้อง';
      cameraStatus = 'error';
      onStatusChange?.(cameraStatus);
      onError?.(error);
      return;
    }
    
    cameraStatus = 'requesting';
    error = null;
    onStatusChange?.(cameraStatus);
    
    try {
      // Request camera permissions with mobile-optimized constraints
      const constraints = {
        video: { 
          facingMode: { ideal: 'environment' }, // Prefer back camera but allow front if needed
          width: { min: 320, ideal: 1280, max: 1920 },
          height: { min: 240, ideal: 720, max: 1080 },
          frameRate: { ideal: 30, max: 60 }
        },
        audio: false
      };
      
      console.log('Requesting camera with constraints:', constraints);
      try {
        stream = await navigator.mediaDevices.getUserMedia(constraints);
        console.log('Camera stream obtained successfully:', stream);
      } catch (getUserMediaError) {
        console.error('getUserMedia failed:', getUserMediaError);
        throw getUserMediaError;
      }
      
      console.log('Checking video element and stream:', { 
        hasVideoElement: !!videoElement, 
        hasStream: !!stream,
        videoElementTagName: videoElement?.tagName,
        streamId: stream?.id
      });
      
      if (!videoElement) {
        console.error('Video element not available, waiting for DOM...');
        // Wait for video element to be available in DOM
        await new Promise(resolve => {
          const checkElement = () => {
            // Try to find video element in DOM
            const domVideoElement = document.querySelector('#video-container video');
            console.log('Checking for video element in DOM:', {
              domVideoElement: !!domVideoElement,
              bindVideoElement: !!videoElement,
              domElementTag: domVideoElement?.tagName
            });
            
            if (domVideoElement) {
              console.log('Video element found in DOM, binding...');
              videoElement = domVideoElement as HTMLVideoElement;
              resolve(true);
            } else if (videoElement) {
              console.log('Video element now available via binding');
              resolve(true);
            } else {
              console.log('Video element still not found, retrying...');
              setTimeout(checkElement, 100);
            }
          };
          checkElement();
        });
      }
      
      if (videoElement && stream) {
        console.log('Setting video stream');
        videoElement.srcObject = stream;
        
        // Wait for video metadata to load
        await new Promise((resolve, reject) => {
          if (!videoElement) {
            reject(new Error('Video element not available'));
            return;
          }
          
          const timeoutId = setTimeout(() => reject(new Error('Video load timeout')), 10000);
          
          const onLoadedMetadata = () => {
            if (!videoElement) return;
            clearTimeout(timeoutId);
            console.log('Video metadata loaded:', videoElement.videoWidth, 'x', videoElement.videoHeight);
            console.log('Video element dimensions:', videoElement.offsetWidth, 'x', videoElement.offsetHeight);
            console.log('Video element styles:', window.getComputedStyle(videoElement));
            debugInfo.videoReady = true;
            debugInfo.streamActive = true;
            resolve(true);
          };
          
          const onVideoError = (e: Event) => {
            clearTimeout(timeoutId);
            console.error('Video error:', e);
            reject(new Error('Video element error'));
          };
          
          videoElement.addEventListener('loadedmetadata', onLoadedMetadata, { once: true });
          videoElement.addEventListener('error', onVideoError, { once: true });
          
          // Also listen for canplay event as backup
          videoElement.addEventListener('canplay', () => {
            console.log('Video can start playing');
            debugInfo.videoReady = true;
          }, { once: true });
        });
        
        // Play video with error handling for mobile
        console.log('Starting video playback');
        
        if (!videoElement) {
          throw new Error('Video element not available for playback');
        }

        try {
          const playPromise = videoElement.play();
          if (playPromise !== undefined) {
            await playPromise;
          }
        } catch (playError) {
          console.warn('Video play failed, trying again:', playError);
          // Sometimes the first play fails, try again after a short delay
          await new Promise(resolve => setTimeout(resolve, 100));
          if (videoElement) {
            await videoElement.play();
          }
        }

        // Set mobile-specific attributes
        if (videoElement) {
          videoElement.setAttribute('webkit-playsinline', 'true');
          videoElement.setAttribute('data-webkit-playsinline', 'true');
          
          // Force video visibility and correct sizing
          videoElement.style.visibility = 'visible';
          videoElement.style.opacity = '1';
          videoElement.style.display = 'block';
          videoElement.style.width = '100%';
          videoElement.style.height = '100%';
          videoElement.style.objectFit = 'cover';
          videoElement.style.zIndex = '1';
          
          // Force a reflow to ensure proper sizing
          videoElement.offsetHeight;
        }
        
        // Additional checks for video display
        if (videoElement) {
          console.log('Video element computed styles:', {
            display: window.getComputedStyle(videoElement).display,
            visibility: window.getComputedStyle(videoElement).visibility,
            opacity: window.getComputedStyle(videoElement).opacity,
            position: window.getComputedStyle(videoElement).position,
            width: window.getComputedStyle(videoElement).width,
            height: window.getComputedStyle(videoElement).height,
            objectFit: window.getComputedStyle(videoElement).objectFit,
            transform: window.getComputedStyle(videoElement).transform
          });
        }
        
        // Additional debugging for video display
        setTimeout(() => {
          if (videoElement) {
            console.log('Video final check:', {
              videoWidth: videoElement.videoWidth,
              videoHeight: videoElement.videoHeight,
              offsetWidth: videoElement.offsetWidth,
              offsetHeight: videoElement.offsetHeight,
              clientWidth: videoElement.clientWidth,
              clientHeight: videoElement.clientHeight,
              readyState: videoElement.readyState,
              paused: videoElement.paused,
              srcObject: !!videoElement.srcObject,
              parentElement: !!videoElement.parentElement,
              isConnected: videoElement.isConnected,
              style: videoElement.style.cssText
            });
            
            // Try to force repaint
            const parent = videoElement.parentElement;
            if (parent) {
              console.log('Parent element info:', {
                offsetWidth: parent.offsetWidth,
                offsetHeight: parent.offsetHeight,
                className: parent.className,
                computedStyle: window.getComputedStyle(parent).cssText
              });
              
              // Force repaint by temporarily hiding and showing
              parent.style.display = 'none';
              parent.offsetHeight; // Force reflow
              parent.style.display = '';
            }
          }
        }, 1000);
        
        cameraStatus = 'active';
        isScanning = true;
        debugInfo.streamActive = true;
        console.log('Camera started successfully');
        
        // Start QR detection after a short delay to ensure video is rendering
        setTimeout(() => {
          startQRDetection();
        }, 500);
      }
    } catch (err) {
      console.error('Failed to start camera:', err);
      
      // Provide more specific error messages
      const error_obj = err as Error & { name?: string };
      if (error_obj.name === 'NotAllowedError') {
        error = 'กรุณาอนุญาตการใช้งานกล้องในเบราว์เซอร์';
      } else if (error_obj.name === 'NotFoundError') {
        error = 'ไม่พบกล้องในอุปกรณ์';
      } else if (error_obj.name === 'NotReadableError') {
        error = 'กล้องถูกใช้งานโดยแอปพลิเคชันอื่น';
      } else if (error_obj.name === 'OverconstrainedError') {
        error = 'กล้องไม่รองรับการตั้งค่าที่ต้องการ';
      } else {
        error = `ไม่สามารถเข้าถึงกล้องได้: ${error_obj.message || 'ข้อผิดพลาดไม่ทราบสาเหตุ'}`;
      }
      
      cameraStatus = 'error';
      onError?.(error);
    }
    
    onStatusChange?.(cameraStatus);
  }

  function stopCamera() {
    console.log('Stopping camera');
    
    if (stream) {
      stream.getTracks().forEach(track => {
        console.log('Stopping track:', track.label);
        track.stop();
      });
      stream = null;
    }
    
    if (videoElement) {
      videoElement.pause();
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
    
    // Check if video is ready
    if (videoElement.videoWidth === 0 || videoElement.videoHeight === 0) {
      return; // Video not ready yet
    }

    // Create canvas to capture video frame
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    
    if (!ctx) return;

    canvas.width = videoElement.videoWidth;
    canvas.height = videoElement.videoHeight;
    
    // Draw video frame normally (no flipping)
    ctx.drawImage(videoElement, 0, 0, canvas.width, canvas.height);
    
    // Get image data
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    
    // Use jsQR library for QR code detection
    try {
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
        <div class="aspect-video bg-muted rounded-lg overflow-hidden border-2 border-dashed relative" id="video-container">
          {#if cameraStatus === 'active' || cameraStatus === 'requesting'}
            <!-- svelte-ignore a11y_media_has_caption -->
            <video
              bind:this={videoElement}
              class="absolute inset-0 w-full h-full object-cover bg-black"
              playsinline={true}
              muted={true}
              autoplay={true}
              controls={false}
              preload="auto"
              style="width: 100% !important; height: 100% !important; object-fit: cover !important; background-color: black !important; z-index: 10;"
              onloadstart={() => console.log('Video load start')}
              onloadeddata={() => console.log('Video data loaded')}
              onloadedmetadata={() => {
                if (videoElement) {
                  console.log('Video metadata loaded - size:', videoElement.videoWidth, 'x', videoElement.videoHeight);
                  // Force video to be visible after metadata loads
                  videoElement.style.opacity = '1';
                  videoElement.style.visibility = 'visible';
                  videoElement.style.display = 'block';
                }
              }}
              oncanplay={() => {
                if (videoElement) {
                  console.log('Video can play');
                  // Ensure video is visible when it can play
                  videoElement.style.opacity = '1';
                  videoElement.style.visibility = 'visible';
                  videoElement.style.display = 'block';
                }
              }}
              oncanplaythrough={() => console.log('Video can play through')}
              onplaying={() => {
                if (videoElement) {
                  console.log('Video playing');
                  // Final check to make sure video is visible
                  videoElement.style.opacity = '1';
                  videoElement.style.visibility = 'visible';
                  videoElement.style.display = 'block';
                }
              }}
              onerror={(e) => console.error('Video element error:', e)}
            ></video>
            
            <!-- Debug overlay -->
            {#if import.meta.env.DEV}
              <div class="absolute top-2 left-2 bg-black/70 text-white text-xs p-2 rounded z-30">
                Status: {cameraStatus}<br>
                Stream: {debugInfo.streamActive ? 'Yes' : 'No'}<br>
                Video: {videoElement?.videoWidth || 0}x{videoElement?.videoHeight || 0}<br>
                Element: {videoElement?.offsetWidth || 0}x{videoElement?.offsetHeight || 0}<br>
                Ready: {debugInfo.videoReady ? 'Yes' : 'No'}
              </div>
            {/if}
            
            <!-- Fallback debug overlay -->
            {#if debugInfo.streamActive && videoElement?.videoWidth === 0}
              <div class="absolute inset-0 bg-red-500/20 flex items-center justify-center text-white text-sm z-20">
                กล้องเชื่อมต่อแล้ว แต่ไม่มีภาพ
                <br>
                กรุณาตรวจสอบ Console
              </div>
            {/if}
            
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
          {:else if cameraStatus === 'error'}
            <div class="absolute inset-0 flex items-center justify-center">
              <div class="text-center space-y-3 text-muted-foreground">
                <IconCameraOff class="size-12 mx-auto text-destructive" />
                <div>
                  <p class="font-medium text-destructive">ไม่สามารถเข้าถึงกล้องได้</p>
                  <p class="text-sm">กรุณาอนุญาตการใช้งานกล้องและรีเฟรชหน้า</p>
                </div>
              </div>
            </div>
          {:else}
            <div class="absolute inset-0 flex items-center justify-center">
              <div class="text-center space-y-3 text-muted-foreground">
                <IconCamera class="size-12 mx-auto" />
                <div>
                  <p class="font-medium">เริ่มต้นการสแกน</p>
                  <p class="text-sm">กดปุ่มเพื่อเปิดกล้อง</p>
                </div>
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
        {:else if cameraStatus === 'active' || cameraStatus === 'requesting'}
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
        
        <!-- Debug Information (development only) -->
        {#if import.meta.env.DEV}
          <div class="mt-4 p-2 bg-muted/50 rounded text-left">
            <p class="font-semibold text-xs mb-1">Debug Info:</p>
            <div class="text-xs space-y-1">
              <p>Camera Status: {cameraStatus}</p>
              <p>Video Ready: {debugInfo.videoReady}</p>
              <p>Stream Active: {debugInfo.streamActive}</p>
              {#if videoElement}
                <p>Video Size: {videoElement.videoWidth}x{videoElement.videoHeight}</p>
              {/if}
            </div>
          </div>
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
