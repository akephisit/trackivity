<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { browser } from '$app/environment';
  
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import { Badge } from '$lib/components/ui/badge';
  
  import {
    IconWifiOff,
    IconRefresh,
    IconHome,
    IconQrcode,
    IconActivity,
    IconAlertCircle
  } from '@tabler/icons-svelte';

  let isOnline = true;
  let retryCount = 0;
  let isRetrying = false;

  // Check online status
  onMount(() => {
    if (browser) {
      isOnline = navigator.onLine;
      
      const handleOnline = () => {
        isOnline = true;
        // Auto-redirect when back online
        setTimeout(() => {
          goto('/', { replaceState: true });
        }, 1000);
      };
      
      const handleOffline = () => {
        isOnline = false;
      };
      
      window.addEventListener('online', handleOnline);
      window.addEventListener('offline', handleOffline);
      
      return () => {
        window.removeEventListener('online', handleOnline);
        window.removeEventListener('offline', handleOffline);
      };
    }
  });

  async function handleRetry() {
    if (isRetrying) return;
    
    isRetrying = true;
    retryCount++;
    
    try {
      // Try to fetch a simple endpoint to check connectivity
      const response = await fetch('/', { 
        method: 'HEAD',
        cache: 'no-cache'
      });
      
      if (response.ok) {
        goto('/', { replaceState: true });
      } else {
        throw new Error('Network request failed');
      }
    } catch (error) {
      console.error('Retry failed:', error);
      // Show error feedback
    } finally {
      isRetrying = false;
    }
  }

  function goHome() {
    goto('/');
  }
</script>

<svelte:head>
  <title>ออฟไลน์ - Trackivity</title>
  <meta name="robots" content="noindex, nofollow" />
</svelte:head>

<div class="min-h-screen bg-background flex items-center justify-center p-4">
  <div class="w-full max-w-md space-y-6">
    <!-- Main Offline Card -->
    <Card class="text-center">
      <CardHeader class="pb-4">
        <div class="mx-auto w-16 h-16 bg-muted rounded-full flex items-center justify-center mb-4">
          <IconWifiOff class="size-8 text-muted-foreground" />
        </div>
        <CardTitle class="text-2xl">ไม่สามารถเชื่อมต่อได้</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <p class="text-muted-foreground text-sm leading-relaxed">
          ขณะนี้ไม่สามารถเชื่อมต่ออินเทอร์เน็ตได้ กรุณาตรวจสอบการเชื่อมต่อของคุณและลองใหม่อีกครั้ง
        </p>
        
        <!-- Connection Status -->
        <div class="flex justify-center">
          <Badge variant={isOnline ? "default" : "destructive"} class="text-xs">
            {isOnline ? "🟢 ออนไลน์" : "🔴 ออฟไลน์"}
          </Badge>
        </div>
        
        <!-- Action Buttons -->
        <div class="space-y-2">
          <Button 
            onclick={handleRetry}
            disabled={isRetrying}
            class="w-full"
          >
            {#if isRetrying}
              <IconRefresh class="size-4 mr-2 animate-spin" />
              กำลังลองใหม่...
            {:else}
              <IconRefresh class="size-4 mr-2" />
              ลองใหม่อีกครั้ง
            {/if}
          </Button>
          
          <Button 
            variant="outline"
            onclick={goHome}
            class="w-full"
          >
            <IconHome class="size-4 mr-2" />
            กลับหน้าหลัก
          </Button>
        </div>
        
        {#if retryCount > 0}
          <p class="text-xs text-muted-foreground">
            ลองเชื่อมต่อแล้ว {retryCount} ครั้ง
          </p>
        {/if}
      </CardContent>
    </Card>
    
    <!-- Offline Features -->
    <Card>
      <CardHeader>
        <CardTitle class="text-lg flex items-center gap-2">
          <IconActivity class="size-5" />
          ฟีเจอร์ออฟไลน์
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <div class="flex items-center gap-3 p-2 rounded bg-muted/50">
          <IconQrcode class="size-4 text-muted-foreground" />
          <div>
            <p class="text-sm font-medium">QR Code ส่วนตัว</p>
            <p class="text-xs text-muted-foreground">ดู QR Code ที่บันทึกไว้</p>
          </div>
        </div>
        
        <div class="flex items-center gap-3 p-2 rounded bg-muted/50">
          <IconActivity class="size-4 text-muted-foreground" />
          <div>
            <p class="text-sm font-medium">ข้อมูลที่เคชไว้</p>
            <p class="text-xs text-muted-foreground">ดูกิจกรรมที่บันทึกไว้</p>
          </div>
        </div>
      </CardContent>
    </Card>
    
    <!-- Tips -->
    <Alert>
      <IconAlertCircle class="size-4" />
      <AlertDescription class="text-sm">
        <strong>เคล็ดลับ:</strong> แอปจะกลับมาทำงานอัตโนมัติเมื่อการเชื่อมต่อกลับมาปกติ
        ข้อมูลที่ป้อนขณะออฟไลน์จะถูกซิงค์เมื่อเชื่อมต่อใหม่
      </AlertDescription>
    </Alert>
  </div>
</div>

<style>
  /* Offline page specific styles */
  @keyframes pulse-offline {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }
  
  :global(.offline-pulse) {
    animation: pulse-offline 2s ease-in-out infinite;
  }
  
  /* Better visibility for offline state */
  @media (prefers-color-scheme: dark) {
    :global(.offline-card) {
      background: hsl(220 13% 18%);
      border: 1px solid hsl(220 13% 25%);
    }
  }
</style>