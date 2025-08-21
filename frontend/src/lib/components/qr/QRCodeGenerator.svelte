<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentUser } from '$lib/stores/auth';
  import { useQRCode } from '$lib/qr/client';
  import type { QRStatus } from '$lib/qr/client';
  
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Skeleton } from '$lib/components/ui/skeleton';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Progress } from '$lib/components/ui/progress';
  
  import { 
    IconRefresh, 
    IconDownload, 
    IconClock, 
    IconCheck,
    IconX,
    IconQrcode,
    IconAlertTriangle
  } from '@tabler/icons-svelte';

  // QR Code functionality
  const { qrCode, qrDataURL, status, error, generate, download } = useQRCode();
  
  // Component props
  interface Props {
    autoRefresh?: boolean;
    showDownload?: boolean;
    showStatus?: boolean;
    size?: 'small' | 'medium' | 'large';
  }
  
  const { 
    autoRefresh = true,
    showDownload = true, 
    showStatus = true,
    size = 'medium'
  }: Props = $props();
  
  // Reactive variables
  let refreshTimer: NodeJS.Timeout | null = null;
  let timeUntilExpiry = $state(0);
  let expiryTimer: NodeJS.Timeout | null = null;
  
  // Size configurations - responsive classes
  const sizeConfig = {
    small: 'w-48 h-48 max-w-[85vw] max-h-[85vw]',
    medium: 'w-56 h-56 max-w-[80vw] max-h-[80vw]',
    large: 'w-64 h-64 max-w-[75vw] max-h-[75vw] sm:w-80 sm:h-80 md:w-96 md:h-96'
  };

  // Status configurations
  const statusConfig: Record<QRStatus, { color: 'secondary' | 'default' | 'destructive' | 'outline'; icon: any; text: string }> = {
    idle: { color: 'secondary', icon: IconClock, text: 'พร้อมสร้าง' },
    generating: { color: 'secondary', icon: IconRefresh, text: 'กำลังสร้าง...' },
    ready: { color: 'default', icon: IconCheck, text: 'พร้อมใช้งาน' },
    expired: { color: 'destructive', icon: IconX, text: 'หมดอายุ' },
    error: { color: 'destructive', icon: IconAlertTriangle, text: 'เกิดข้อผิดพลาด' }
  };

  // Initialize QR code
  onMount(async () => {
    console.log('[QRGenerator] Component mounted, user:', $currentUser ? 'available' : 'not available');
    if ($currentUser) {
      try {
        await generate();
        startTimers();
      } catch (error) {
        console.error('[QRGenerator] Failed to generate QR on mount:', error);
      }
    }
  });

  // Cleanup timers
  onDestroy(() => {
    clearTimers();
  });

  // Watch for user changes
  $effect(() => {
    if ($currentUser && ($status === 'idle' || !$qrCode)) {
      console.log('[QRGenerator] User available and QR needed, generating...');
      generate().catch(error => {
        console.error('[QRGenerator] Failed to generate QR for user:', error);
      });
    }
  });

  // Watch for QR code changes to update timers
  $effect(() => {
    if ($qrCode) {
      updateExpiryTimer();
    }
  });

  function startTimers() {
    if (!autoRefresh) return;
    
    // Auto-refresh every 2.5 minutes (before 3-minute expiry)
    refreshTimer = setInterval(() => {
      if ($status === 'ready' || $status === 'expired') {
        generate();
      }
    }, 150000); // 2.5 minutes
  }

  function clearTimers() {
    if (refreshTimer) {
      clearTimeout(refreshTimer);
      refreshTimer = null;
    }
    if (expiryTimer) {
      clearInterval(expiryTimer);
      expiryTimer = null;
    }
  }

  function updateExpiryTimer() {
    if (expiryTimer) {
      clearInterval(expiryTimer);
    }

    if (!$qrCode) return;

    expiryTimer = setInterval(() => {
      const expiresAt = new Date($qrCode.expires_at).getTime();
      const now = Date.now();
      const remaining = Math.max(0, expiresAt - now);
      
      timeUntilExpiry = Math.floor(remaining / 1000);

      // Auto-refresh when expired
      if (remaining <= 0 && $status === 'ready') {
        generate();
      }
    }, 1000);
  }

  function handleRefresh() {
    console.log('[QRGenerator] Manual refresh triggered');
    generate().catch(error => {
      console.error('[QRGenerator] Manual refresh failed:', error);
    });
  }

  function handleDownload() {
    if ($qrDataURL) {
      const filename = `qr-code-${$currentUser?.student_id || 'user'}-${Date.now()}.png`;
      download(filename);
    }
  }

  function formatTimeRemaining(seconds: number): string {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  function getExpiryProgress(): number {
    if (!$qrCode) return 0;
    
    const expiresAt = new Date($qrCode.expires_at).getTime();
    const createdAt = new Date($qrCode.created_at).getTime();
    const now = Date.now();
    
    const totalDuration = expiresAt - createdAt;
    const elapsed = now - createdAt;
    
    return Math.min(100, Math.max(0, (elapsed / totalDuration) * 100));
  }

  const currentStatusConfig = $derived(statusConfig[$status]);
  const isExpiringSoon = $derived(timeUntilExpiry > 0 && timeUntilExpiry <= 60); // Less than 1 minute
</script>

<Card class="w-full max-w-sm mx-auto md:max-w-md">
  <CardHeader class="text-center">
    <CardTitle class="flex items-center justify-center gap-2">
      <IconQrcode class="size-5" />
      QR Code ของฉัน
    </CardTitle>
    
    {#if showStatus}
      <div class="flex items-center justify-center gap-2 mt-2">
        <Badge variant={currentStatusConfig.color} class="flex items-center gap-1">
          {@const IconComponent = currentStatusConfig.icon}
          <IconComponent 
            class="size-3 {$status === 'generating' ? 'animate-spin' : ''}" 
          />
          {currentStatusConfig.text}
        </Badge>
      </div>
    {/if}
  </CardHeader>

  <CardContent class="space-y-4">
    <!-- QR Code Display -->
    <div class="flex items-center justify-center p-2 bg-white rounded-lg border-2 border-dashed border-muted transition-all duration-300">
      <div class="flex items-center justify-center {sizeConfig[size]}">
      {#if $status === 'generating'}
        <div class="space-y-4">
          <Skeleton class="w-full h-32 rounded" />
          <Skeleton class="w-3/4 h-4 mx-auto" />
        </div>
      {:else if $qrDataURL && $status === 'ready'}
        <img 
          src={$qrDataURL} 
          alt="QR Code สำหรับเช็คชื่อกิจกรรม" 
          class="max-w-full max-h-full object-contain transition-opacity duration-300 {isExpiringSoon ? 'opacity-60' : 'opacity-100'}"
        />
      {:else if $status === 'error'}
        <div class="text-center text-muted-foreground space-y-2">
          <IconAlertTriangle class="size-8 mx-auto text-destructive" />
          <p class="text-sm">ไม่สามารถสร้าง QR Code ได้</p>
        </div>
      {:else if $status === 'expired'}
        <div class="text-center text-muted-foreground space-y-2">
          <IconX class="size-8 mx-auto text-muted-foreground" />
          <p class="text-sm">QR Code หมดอายุแล้ว</p>
        </div>
      {:else}
        <div class="text-center text-muted-foreground space-y-2">
          <IconQrcode class="size-8 mx-auto" />
          <p class="text-sm">กดปุ่มเพื่อสร้าง QR Code</p>
        </div>
      {/if}
      </div>
    </div>

    <!-- Error Alert -->
    {#if $error}
      <Alert variant="destructive">
        <IconAlertTriangle class="h-4 w-4" />
        <AlertDescription>{$error}</AlertDescription>
      </Alert>
    {/if}

    <!-- Expiry Information -->
    {#if $qrCode && $status === 'ready'}
      <div class="space-y-2">
        <div class="flex items-center justify-between text-sm text-muted-foreground">
          <span>เหลือเวลา:</span>
          <span class="font-mono {isExpiringSoon ? 'text-destructive font-semibold' : ''}">
            {formatTimeRemaining(timeUntilExpiry)}
          </span>
        </div>
        
        <Progress 
          value={getExpiryProgress()} 
          class="h-2 {isExpiringSoon ? 'bg-destructive/20' : ''}"
        />
        
        {#if isExpiringSoon}
          <p class="text-xs text-destructive text-center">
            QR Code กำลังจะหมดอายุ กรุณารอการสร้างใหม่อัตโนมัติ
          </p>
        {/if}
      </div>
    {/if}

    <!-- User Information -->
    {#if $currentUser}
      <div class="text-center space-y-1 py-2 border-t">
        <p class="text-sm font-medium">
          {$currentUser.first_name} {$currentUser.last_name}
        </p>
        <p class="text-xs text-muted-foreground">
          รหัสนักศึกษา: {$currentUser.student_id}
        </p>
      </div>
    {/if}

    <!-- Action Buttons -->
    <div class="flex gap-2 pt-2">
      <Button 
        onclick={handleRefresh} 
        variant="outline" 
        size="sm" 
        class="flex-1"
        disabled={$status === 'generating'}
      >
        <IconRefresh class="size-4 mr-2 {$status === 'generating' ? 'animate-spin' : ''}" />
        {$status === 'generating' ? 'กำลังสร้าง...' : 'สร้างใหม่'}
      </Button>
      
      {#if showDownload}
        <Button 
          onclick={handleDownload} 
          variant="outline" 
          size="sm"
          disabled={!$qrDataURL || $status !== 'ready'}
        >
          <IconDownload class="size-4 mr-2" />
          ดาวน์โหลด
        </Button>
      {/if}
    </div>

    <!-- Usage Instructions -->
    <div class="text-xs text-muted-foreground text-center space-y-1 pt-2 border-t">
      <p>แสดง QR Code นี้ให้ผู้ดูแลกิจกรรมสแกน</p>
      <p>QR Code จะสร้างใหม่อัตโนมัติทุก 3 นาที</p>
    </div>
  </CardContent>
</Card>