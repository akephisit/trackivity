<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { currentUser, isAuthenticated } from '$lib/stores/auth';
  import { getNavigationItems } from '$lib/navigation/routes';
  import { deviceInfo } from '$lib/hooks/use-mobile.svelte';
  
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetTrigger } from '$lib/components/ui/sheet';
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
  
  import {
    IconMenu2,
    IconHome,
    IconUser,
    IconLogout,
    IconQrcode,
    IconCalendarEvent,
    IconScan,
    IconShield,
    IconSchool,
    IconAnalyze,
    IconSettings
  } from '@tabler/icons-svelte';

  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  // Component props
  export let children: any;

  // Mobile menu state
  let mobileMenuOpen = false;

  // Navigation items based on user permissions
  $: navigationItems = $isAuthenticated && $currentUser 
    ? getNavigationItems($currentUser.permissions, $currentUser.admin_role?.admin_level)
    : [];

  // Icon mapping
  const iconMap: Record<string, any> = {
    'dashboard': IconHome,
    'qr_code': IconQrcode,
    'event': IconCalendarEvent,
    'person': IconUser,
    'qr_code_scanner': IconScan,
    'assignment': IconCalendarEvent,
    'devices': IconShield,
    'school': IconSchool,
    'analytics': IconAnalyze,
    'group': IconUser,
    'assessment': IconAnalyze,
    'security': IconShield,
    'admin_panel_settings': IconSettings,
    'account_balance': IconSchool,
    'people': IconUser,
    'settings': IconSettings
  };

  function getIcon(iconName?: string) {
    return iconName ? iconMap[iconName] || IconHome : IconHome;
  }

  function isActiveRoute(url: string): boolean {
    return $page.url.pathname === url || $page.url.pathname.startsWith(url + '/');
  }

  function closeMobileMenu() {
    mobileMenuOpen = false;
  }

  async function handleLogout() {
    closeMobileMenu();
    await auth.logout();
    goto('/login');
  }

  function getUserInitials(user: any): string {
    if (user?.first_name && user?.last_name) {
      return `${user.first_name[0]}${user.last_name[0]}`.toUpperCase();
    }
    if (user?.email) {
      return user.email[0].toUpperCase();
    }
    return 'U';
  }

  function getRoleDisplayName(adminLevel?: string): string {
    switch (adminLevel) {
      case 'SuperAdmin': return 'ผู้ดูแลระบบสูงสุด';
      case 'FacultyAdmin': return 'ผู้ดูแลคณะ';
      case 'RegularAdmin': return 'ผู้ดูแลทั่วไป';
      default: return 'นักศึกษา';
    }
  }

  function getRoleBadgeVariant(adminLevel?: string): 'default' | 'secondary' | 'destructive' | 'outline' {
    switch (adminLevel) {
      case 'SuperAdmin': return 'destructive';
      case 'FacultyAdmin': return 'default';
      case 'RegularAdmin': return 'secondary';
      default: return 'outline';
    }
  }

  // Close mobile menu on route change
  $: if ($page.url.pathname) {
    closeMobileMenu();
  }

  onMount(() => {
    // Add viewport meta tag for mobile optimization
    const viewport = document.querySelector('meta[name="viewport"]');
    if (!viewport) {
      const meta = document.createElement('meta');
      meta.name = 'viewport';
      meta.content = 'width=device-width, initial-scale=1, maximum-scale=5, user-scalable=yes';
      document.head.appendChild(meta);
    }
  });
</script>

<div class="min-h-screen bg-background">
  {#if $deviceInfo.isMobile}
    <!-- Mobile Layout -->
    <div class="flex flex-col h-screen">
      <!-- Mobile Header -->
      <header class="sticky top-0 z-50 w-full border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div class="container flex h-14 items-center px-4">
          <Sheet bind:open={mobileMenuOpen}>
            <SheetTrigger>
              <Button variant="ghost" size="icon" class="-ml-2">
                <IconMenu2 class="size-5" />
                <span class="sr-only">Toggle menu</span>
              </Button>
            </SheetTrigger>
            <SheetContent side="left" class="w-[300px] sm:w-[400px]">
              <SheetHeader>
                <SheetTitle class="flex items-center gap-2">
                  <IconShield class="size-5" />
                  Trackivity
                </SheetTitle>
              </SheetHeader>

              {#if $isAuthenticated && $currentUser}
                <!-- User Info -->
                <div class="flex items-center gap-3 p-4 border rounded-lg mt-4">
                  <Avatar class="h-10 w-10">
                    <AvatarImage src={$currentUser.avatar_url} alt={$currentUser.first_name || 'User'} />
                    <AvatarFallback>
                      {getUserInitials($currentUser)}
                    </AvatarFallback>
                  </Avatar>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium truncate">
                      {$currentUser.first_name} {$currentUser.last_name}
                    </p>
                    <p class="text-xs text-muted-foreground truncate">
                      {$currentUser.email}
                    </p>
                    {#if $currentUser.admin_role}
                      <Badge variant={getRoleBadgeVariant($currentUser.admin_role.admin_level)} class="text-xs mt-1">
                        {getRoleDisplayName($currentUser.admin_role.admin_level)}
                      </Badge>
                    {/if}
                  </div>
                </div>

                <!-- Navigation -->
                <nav class="space-y-2 mt-6">
                  {#each navigationItems as item}
                    <a
                      href={item.url}
                      class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors hover:bg-accent hover:text-accent-foreground {isActiveRoute(item.url || '') ? 'bg-accent text-accent-foreground' : ''}"
                      onclick={closeMobileMenu}
                    >
                      <svelte:component this={getIcon(item.icon)} class="size-4" />
                      {item.title}
                    </a>
                  {/each}
                </nav>

                <!-- Logout Button -->
                <div class="absolute bottom-4 left-4 right-4">
                  <Button 
                    variant="outline" 
                    onclick={handleLogout}
                    class="w-full justify-start"
                  >
                    <IconLogout class="size-4 mr-2" />
                    ออกจากระบบ
                  </Button>
                </div>
              {:else}
                <!-- Not authenticated -->
                <div class="flex flex-col gap-2 mt-6">
                  <Button href="/login" onclick={closeMobileMenu}>
                    เข้าสู่ระบบ
                  </Button>
                  <Button href="/register" variant="outline" onclick={closeMobileMenu}>
                    สมัครสมาชิก
                  </Button>
                </div>
              {/if}
            </SheetContent>
          </Sheet>

          <div class="flex-1 text-center">
            <h1 class="text-lg font-semibold">Trackivity</h1>
          </div>

          {#if $isAuthenticated && $currentUser}
            <Button
              variant="ghost"
              size="icon"
              href="/profile"
              class="-mr-2"
            >
              <Avatar class="h-8 w-8">
                <AvatarImage src={$currentUser.avatar_url} alt={$currentUser.first_name || 'User'} />
                <AvatarFallback class="text-xs">
                  {getUserInitials($currentUser)}
                </AvatarFallback>
              </Avatar>
              <span class="sr-only">Profile</span>
            </Button>
          {/if}
        </div>
      </header>

      <!-- Mobile Content -->
      <main class="flex-1 overflow-y-auto overflow-x-hidden">
        <div class="container mx-auto px-4 py-6 min-h-fit">
          {@render children()}
        </div>
      </main>

      <!-- Mobile Bottom Navigation (if authenticated) -->
      {#if $isAuthenticated}
        <nav class="sticky bottom-0 z-50 w-full border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <div class="container flex h-16 items-center justify-around px-2">
            {#each navigationItems.slice(0, 4) as item}
              <a
                href={item.url}
                class="flex flex-col items-center justify-center gap-1 rounded-lg p-2 transition-colors hover:bg-accent hover:text-accent-foreground {isActiveRoute(item.url || '') ? 'text-primary' : 'text-muted-foreground'}"
              >
                <svelte:component this={getIcon(item.icon)} class="size-4" />
                <span class="text-xs truncate max-w-12">{item.title}</span>
              </a>
            {/each}
            
            {#if navigationItems.length > 4}
              <button
                onclick={() => mobileMenuOpen = true}
                class="flex flex-col items-center justify-center gap-1 rounded-lg p-2 transition-colors hover:bg-accent hover:text-accent-foreground text-muted-foreground"
              >
                <IconMenu2 class="size-4" />
                <span class="text-xs">เพิ่มเติม</span>
              </button>
            {/if}
          </div>
        </nav>
      {/if}
    </div>
  {:else}
    <!-- Desktop Layout (Fallback) -->
    <div class="flex">
      <main class="flex-1">
        {@render children()}
      </main>
    </div>
  {/if}
</div>

<style>
  /* Mobile-specific styles */
  @media (max-width: 768px) {
    /* Ensure proper touch targets */
    :global(button, a, input, textarea) {
      min-height: 44px;
    }

    /* Improve text readability on mobile */
    :global(body) {
      -webkit-text-size-adjust: none;
      -moz-text-size-adjust: none;
      -ms-text-size-adjust: none;
      text-size-adjust: none;
    }

    /* Optimize scrolling */
    :global(html) {
      scroll-behavior: smooth;
    }

    /* Prevent horizontal scrolling but allow vertical */
    :global(html, body) {
      overflow-x: hidden;
      overflow-y: auto;
    }

    /* Better form inputs on mobile */
    :global(input, textarea, select) {
      font-size: 16px; /* Prevents zoom on iOS */
    }
  }

  /* PWA-specific styles */
  @media (display-mode: standalone) {
    /* Add top padding for PWA status bar */
    :global(body) {
      padding-top: env(safe-area-inset-top);
      padding-bottom: env(safe-area-inset-bottom);
    }
  }

  /* Dark mode adjustments for mobile */
  @media (prefers-color-scheme: dark) {
    /* Better contrast for mobile screens */
    :global(.dark) {
      --background: 222.2 84% 4.9%;
      --foreground: 210 40% 98%;
    }
  }

  /* High contrast mode support */
  @media (prefers-contrast: high) {
    :global(button, a) {
      outline: 2px solid;
      outline-offset: 2px;
    }
  }

  /* Reduced motion support */
  @media (prefers-reduced-motion: reduce) {
    :global(*) {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
      scroll-behavior: auto !important;
    }
  }
</style>