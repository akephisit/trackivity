<script lang="ts">
  import { onMount } from 'svelte';
  import { deviceInfo } from '$lib/hooks/use-mobile.svelte';
  import { enableTouchOptimizations } from '$lib/hooks/use-mobile.svelte';
  
  import MobileLayout from './MobileLayout.svelte';
  import RoleBasedSidebar from '../navigation/RoleBasedSidebar.svelte';
  
  // Component props
  export let children: any;

  onMount(() => {
    // Enable touch optimizations on mobile devices
    enableTouchOptimizations();
    
    // Add mobile-specific CSS custom properties
    document.documentElement.style.setProperty('--mobile-vh', `${window.innerHeight * 0.01}px`);
    
    // Update mobile viewport height on resize/orientation change
    const updateMobileVh = () => {
      document.documentElement.style.setProperty('--mobile-vh', `${window.innerHeight * 0.01}px`);
    };
    
    window.addEventListener('resize', updateMobileVh);
    window.addEventListener('orientationchange', () => {
      // Delay to ensure accurate measurements after orientation change
      setTimeout(updateMobileVh, 100);
    });
    
    return () => {
      window.removeEventListener('resize', updateMobileVh);
      window.removeEventListener('orientationchange', updateMobileVh);
    };
  });
</script>

{#if $deviceInfo.isMobile}
  <!-- Mobile Layout -->
  <MobileLayout {children} />
{:else}
  <!-- Desktop Layout -->
  <div class="flex min-h-screen bg-background">
    <RoleBasedSidebar />
    <main class="flex-1 overflow-auto">
      <div class="container mx-auto px-6 py-8 max-w-7xl">
        {@render children()}
      </div>
    </main>
  </div>
{/if}

<style>
  /* Use CSS custom property for mobile viewport height */
  :global(html) {
    --mobile-vh: 1vh;
  }
  
  /* Mobile-first responsive design utilities */
  @media (max-width: 767px) {
    :global(.mobile-full-height) {
      height: calc(var(--mobile-vh, 1vh) * 100);
      min-height: calc(var(--mobile-vh, 1vh) * 100);
    }
    
    :global(.mobile-safe-area-top) {
      padding-top: env(safe-area-inset-top, 0);
    }
    
    :global(.mobile-safe-area-bottom) {
      padding-bottom: env(safe-area-inset-bottom, 0);
    }
    
    :global(.mobile-safe-area-left) {
      padding-left: env(safe-area-inset-left, 0);
    }
    
    :global(.mobile-safe-area-right) {
      padding-right: env(safe-area-inset-right, 0);
    }
    
    :global(.mobile-safe-area) {
      padding-top: env(safe-area-inset-top, 0);
      padding-bottom: env(safe-area-inset-bottom, 0);
      padding-left: env(safe-area-inset-left, 0);
      padding-right: env(safe-area-inset-right, 0);
    }
  }
  
  /* Touch-friendly button sizes */
  @media (pointer: coarse) {
    :global(button), 
    :global(a[role="button"]), 
    :global(input[type="button"]), 
    :global(input[type="submit"]) {
      min-height: 44px;
      min-width: 44px;
    }
  }
  
  /* High DPI display optimizations */
  @media (-webkit-min-device-pixel-ratio: 2), (min-resolution: 192dpi) {
    :global(img), 
    :global(svg) {
      image-rendering: -webkit-optimize-contrast;
      image-rendering: crisp-edges;
    }
  }
  
  /* Reduce motion for users with vestibular motion disorders */
  @media (prefers-reduced-motion: reduce) {
    :global(*) {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
      scroll-behavior: auto !important;
    }
  }
  
  /* High contrast mode support */
  @media (prefers-contrast: high) {
    :global(button), 
    :global(a), 
    :global(.clickable) {
      outline: 2px solid currentColor;
      outline-offset: 2px;
    }
  }
  
  /* Dark mode optimizations for mobile */
  @media (prefers-color-scheme: dark) and (max-width: 767px) {
    :global(body) {
      background: hsl(222.2 84% 4.9%);
      color: hsl(210 40% 98%);
    }
  }
</style>