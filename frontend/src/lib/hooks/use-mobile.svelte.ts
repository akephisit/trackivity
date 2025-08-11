/**
 * Mobile detection and responsive utilities for Svelte 5
 * ใช้สำหรับตรวจสอบการใช้งานบน mobile และ responsive behavior
 */

import { browser } from '$app/environment';
import { writable, type Writable } from 'svelte/store';

// ===== BREAKPOINT DEFINITIONS =====
export const BREAKPOINTS = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536
} as const;

export type Breakpoint = keyof typeof BREAKPOINTS;

// ===== DEVICE TYPE DETECTION =====
export interface DeviceInfo {
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  isPWA: boolean;
  isIOS: boolean;
  isAndroid: boolean;
  hasTouch: boolean;
  orientation: 'portrait' | 'landscape';
  screenSize: {
    width: number;
    height: number;
  };
  devicePixelRatio: number;
}

// ===== STORES =====
export const deviceInfo: Writable<DeviceInfo> = writable({
  isMobile: false,
  isTablet: false,
  isDesktop: true,
  isPWA: false,
  isIOS: false,
  isAndroid: false,
  hasTouch: false,
  orientation: 'landscape',
  screenSize: { width: 1024, height: 768 },
  devicePixelRatio: 1
});

export const windowSize: Writable<{ width: number; height: number }> = writable({
  width: 1024,
  height: 768
});

export const breakpoint: Writable<Breakpoint | null> = writable(null);

// ===== DETECTION FUNCTIONS =====
function detectDeviceInfo(): DeviceInfo {
  if (!browser) {
    return {
      isMobile: false,
      isTablet: false,
      isDesktop: true,
      isPWA: false,
      isIOS: false,
      isAndroid: false,
      hasTouch: false,
      orientation: 'landscape',
      screenSize: { width: 1024, height: 768 },
      devicePixelRatio: 1
    };
  }

  const userAgent = navigator.userAgent.toLowerCase();
  const width = window.innerWidth;
  const height = window.innerHeight;

  // Device type detection
  const isMobile = width < BREAKPOINTS.md || /android|iphone|ipod|blackberry|iemobile|opera mini/i.test(userAgent);
  const isTablet = !isMobile && (width < BREAKPOINTS.lg || /ipad|tablet|kindle|silk/i.test(userAgent));
  const isDesktop = !isMobile && !isTablet;

  // OS detection
  const isIOS = /iphone|ipad|ipod/i.test(userAgent);
  const isAndroid = /android/i.test(userAgent);

  // PWA detection
  const isPWA = window.matchMedia('(display-mode: standalone)').matches ||
                (window.navigator as any).standalone === true ||
                document.referrer.includes('android-app://');

  // Touch capability
  const hasTouch = 'ontouchstart' in window || navigator.maxTouchPoints > 0;

  // Orientation
  const orientation = width > height ? 'landscape' : 'portrait';

  return {
    isMobile,
    isTablet,
    isDesktop,
    isPWA,
    isIOS,
    isAndroid,
    hasTouch,
    orientation,
    screenSize: { width, height },
    devicePixelRatio: window.devicePixelRatio || 1
  };
}

function getCurrentBreakpointByWidth(width: number): Breakpoint | null {
  const breakpoints = Object.entries(BREAKPOINTS).reverse() as [Breakpoint, number][];
  
  for (const [name, minWidth] of breakpoints) {
    if (width >= minWidth) {
      return name;
    }
  }
  
  return null;
}

// ===== INITIALIZATION =====
if (browser) {
  // Initialize with current values
  const updateDeviceInfo = () => {
    deviceInfo.set(detectDeviceInfo());
  };

  const updateWindowSize = () => {
    const width = window.innerWidth;
    const height = window.innerHeight;
    
    windowSize.set({ width, height });
    breakpoint.set(getCurrentBreakpointByWidth(width));
  };

  // Initial setup
  updateDeviceInfo();
  updateWindowSize();

  // Listen for changes
  window.addEventListener('resize', () => {
    updateDeviceInfo();
    updateWindowSize();
  }, { passive: true });

  window.addEventListener('orientationchange', () => {
    // Delay to ensure dimensions are updated
    setTimeout(updateDeviceInfo, 100);
  }, { passive: true });

  // Listen for PWA state changes
  window.matchMedia('(display-mode: standalone)').addEventListener('change', updateDeviceInfo);
}

// ===== UTILITY FUNCTIONS =====
export function isMobileDevice(): boolean {
  let mobile = false;
  const unsubscribe = deviceInfo.subscribe(info => mobile = info.isMobile);
  unsubscribe();
  return mobile;
}

export function isTabletDevice(): boolean {
  let tablet = false;
  const unsubscribe = deviceInfo.subscribe(info => tablet = info.isTablet);
  unsubscribe();
  return tablet;
}

export function isDesktopDevice(): boolean {
  let desktop = false;
  const unsubscribe = deviceInfo.subscribe(info => desktop = info.isDesktop);
  unsubscribe();
  return desktop;
}

export function isPWAMode(): boolean {
  let pwa = false;
  const unsubscribe = deviceInfo.subscribe(info => pwa = info.isPWA);
  unsubscribe();
  return pwa;
}

export function hasTouch(): boolean {
  let touch = false;
  const unsubscribe = deviceInfo.subscribe(info => touch = info.hasTouch);
  unsubscribe();
  return touch;
}

export function isPortrait(): boolean {
  let portrait = false;
  const unsubscribe = deviceInfo.subscribe(info => portrait = info.orientation === 'portrait');
  unsubscribe();
  return portrait;
}

export function isLandscape(): boolean {
  let landscape = false;
  const unsubscribe = deviceInfo.subscribe(info => landscape = info.orientation === 'landscape');
  unsubscribe();
  return landscape;
}

export function getCurrentWindowSize(): { width: number; height: number } {
  let size = { width: 1024, height: 768 };
  const unsubscribe = windowSize.subscribe(s => size = s);
  unsubscribe();
  return size;
}

export function getCurrentBreakpoint(): Breakpoint | null {
  let bp = null;
  const unsubscribe = breakpoint.subscribe(b => bp = b);
  unsubscribe();
  return bp;
}

export function isBreakpoint(bp: Breakpoint): boolean {
  const current = getCurrentBreakpoint();
  if (!current) return false;
  
  const breakpoints = Object.keys(BREAKPOINTS) as Breakpoint[];
  const currentIndex = breakpoints.indexOf(current);
  const targetIndex = breakpoints.indexOf(bp);
  
  return currentIndex >= targetIndex;
}

export function isBreakpointOnly(bp: Breakpoint): boolean {
  return getCurrentBreakpoint() === bp;
}

export function isBreakpointDown(bp: Breakpoint): boolean {
  const { width } = getCurrentWindowSize();
  return width < BREAKPOINTS[bp];
}

export function isBreakpointUp(bp: Breakpoint): boolean {
  const { width } = getCurrentWindowSize();
  return width >= BREAKPOINTS[bp];
}

// ===== RESPONSIVE CLASSES =====
export function getResponsiveClasses(
  base: string,
  responsive: Partial<Record<Breakpoint | 'default', string>>
): string {
  const classes = [base];
  const currentBp = getCurrentBreakpoint();
  
  // Add default responsive class if available
  if (responsive.default) {
    classes.push(responsive.default);
  }
  
  // Add breakpoint-specific classes
  if (currentBp && responsive[currentBp]) {
    classes.push(responsive[currentBp]!);
  }
  
  return classes.filter(Boolean).join(' ');
}

// ===== MEDIA QUERIES =====
export function createMediaQuery(query: string): Writable<boolean> {
  const matches = writable(false);
  
  if (browser) {
    const mediaQuery = window.matchMedia(query);
    matches.set(mediaQuery.matches);
    
    const handler = (e: MediaQueryListEvent) => {
      matches.set(e.matches);
    };
    
    mediaQuery.addEventListener('change', handler);
    
    // Cleanup function would be needed in a proper implementation
    // This is a simplified version
  }
  
  return matches;
}

// ===== VIEWPORT UTILITIES =====
export function getViewportHeight(): number {
  if (!browser) return 768;
  return window.innerHeight;
}

export function getViewportWidth(): number {
  if (!browser) return 1024;
  return window.innerWidth;
}

export function getScrollPosition(): { x: number; y: number } {
  if (!browser) return { x: 0, y: 0 };
  return {
    x: window.scrollX || window.pageXOffset,
    y: window.scrollY || window.pageYOffset
  };
}

// ===== TOUCH UTILITIES =====
export function enableTouchOptimizations(): void {
  if (!browser || !hasTouch()) return;
  
  // Add touch-action CSS to prevent default behaviors
  document.documentElement.style.touchAction = 'manipulation';
  
  // Prevent zooming on double-tap
  let lastTouchEnd = 0;
  document.addEventListener('touchend', (event) => {
    const now = (new Date()).getTime();
    if (now - lastTouchEnd <= 300) {
      event.preventDefault();
    }
    lastTouchEnd = now;
  }, { passive: false });
}

// ===== PWA UTILITIES =====
export function installPWA(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (!browser) {
      reject(new Error('Not in browser'));
      return;
    }
    
    const deferredPrompt = (window as any).deferredPrompt;
    if (!deferredPrompt) {
      reject(new Error('PWA installation not available'));
      return;
    }
    
    deferredPrompt.prompt();
    deferredPrompt.userChoice.then((choiceResult: { outcome: string }) => {
      if (choiceResult.outcome === 'accepted') {
        resolve();
      } else {
        reject(new Error('User dismissed PWA installation'));
      }
      (window as any).deferredPrompt = null;
    });
  });
}

export function canInstallPWA(): boolean {
  return browser && !!(window as any).deferredPrompt;
}

// ===== SAFE AREA UTILITIES =====
export function getSafeAreaInsets(): {
  top: number;
  right: number;
  bottom: number;
  left: number;
} {
  if (!browser) return { top: 0, right: 0, bottom: 0, left: 0 };
  
  const computedStyle = getComputedStyle(document.documentElement);
  
  return {
    top: parseInt(computedStyle.getPropertyValue('env(safe-area-inset-top)')) || 0,
    right: parseInt(computedStyle.getPropertyValue('env(safe-area-inset-right)')) || 0,
    bottom: parseInt(computedStyle.getPropertyValue('env(safe-area-inset-bottom)')) || 0,
    left: parseInt(computedStyle.getPropertyValue('env(safe-area-inset-left)')) || 0
  };
}

// ===== PERFORMANCE UTILITIES =====
export function isReducedMotion(): boolean {
  if (!browser) return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

export function isHighContrast(): boolean {
  if (!browser) return false;
  return window.matchMedia('(prefers-contrast: high)').matches;
}

export function prefersDarkMode(): boolean {
  if (!browser) return false;
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}