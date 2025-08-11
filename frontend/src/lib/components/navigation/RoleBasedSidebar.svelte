<script lang="ts">
  import { page } from '$app/stores';
  import { currentUser, isAuthenticated, isSuperAdmin, isFacultyAdmin } from '$lib/stores/auth';
  import { getNavigationItems } from '$lib/navigation/routes';
  
  import {
    Sidebar,
    SidebarContent,
    SidebarGroup,
    SidebarGroupContent,
    SidebarGroupLabel,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuItem,
    SidebarMenuButton,
    SidebarProvider,
    SidebarSeparator,
    SidebarFooter
  } from '$lib/components/ui/sidebar';
  import { Badge } from '$lib/components/ui/badge';
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger
  } from '$lib/components/ui/dropdown-menu';
  
  // Icons (using Tabler icons or similar)
  import { 
    IconChevronDown,
    IconSettings,
    IconLogout,
    IconUser,
    IconShield,
    IconHome,
    IconDashboard,
    IconQrcode,
    IconCalendarEvent,
    IconScan,
    IconClipboard,
    IconDevices,
    IconSchool,
    IconAnalyze,
    IconUsers,
    IconReport,
    IconScript,
    IconTools,
    IconBuilding
  } from '@tabler/icons-svelte';
  
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  // Reactive navigation items based on user permissions
  $: navigationItems = $isAuthenticated && $currentUser 
    ? getNavigationItems($currentUser.permissions, $currentUser.admin_role?.admin_level)
    : [];

  // Icon mapping
  const iconMap: Record<string, any> = {
    'dashboard': IconDashboard,
    'qr_code': IconQrcode,
    'event': IconCalendarEvent,
    'person': IconUser,
    'qr_code_scanner': IconScan,
    'assignment': IconClipboard,
    'devices': IconDevices,
    'school': IconSchool,
    'analytics': IconAnalyze,
    'group': IconUsers,
    'assessment': IconReport,
    'security': IconScript,
    'admin_panel_settings': IconTools,
    'account_balance': IconBuilding,
    'people': IconUsers,
    'settings': IconSettings
  };

  function getIcon(iconName?: string) {
    return iconName ? iconMap[iconName] || IconHome : IconHome;
  }

  function isActiveRoute(url: string): boolean {
    return $page.url.pathname === url || $page.url.pathname.startsWith(url + '/');
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

  async function handleLogout() {
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
</script>

{#if $isAuthenticated}
<SidebarProvider>
  <Sidebar variant="sidebar">
    <!-- Sidebar Header -->
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton size="lg" class="data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground">
            <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
              <IconShield class="size-4" />
            </div>
            <div class="grid flex-1 text-left text-sm leading-tight">
              <span class="truncate font-semibold">Trackivity</span>
              <span class="truncate text-xs">ระบบติดตามกิจกรรม</span>
            </div>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>

    <!-- Main Navigation -->
    <SidebarContent>
      {#if $currentUser?.admin_role}
        <!-- Admin Groups -->
        {#if $isSuperAdmin}
          <SidebarGroup>
            <SidebarGroupLabel>จัดการระบบ</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {#each navigationItems.filter(item => item.url?.includes('/admin/system')) as item}
                  <SidebarMenuItem>
                    <SidebarMenuButton 
                      isActive={isActiveRoute(item.url || '')}
                      class="w-full justify-start"
                    >
                      <a href={item.url} class="flex items-center gap-2 w-full">
                        <svelte:component this={getIcon(item.icon)} class="size-4" />
                        <span>{item.title}</span>
                        {#if item.badge}
                          <Badge variant="secondary" class="ml-auto text-xs">{item.badge}</Badge>
                        {/if}
                      </a>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                {/each}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
          <SidebarSeparator />
        {/if}

        {#if $isFacultyAdmin}
          <SidebarGroup>
            <SidebarGroupLabel>จัดการคณะ</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {#each navigationItems.filter(item => item.url?.includes('/admin/faculty')) as item}
                  <SidebarMenuItem>
                    <SidebarMenuButton 
                      isActive={isActiveRoute(item.url || '')}
                      class="w-full justify-start"
                    >
                      <a href={item.url} class="flex items-center gap-2 w-full">
                        <svelte:component this={getIcon(item.icon)} class="size-4" />
                        <span>{item.title}</span>
                      </a>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                {/each}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
          <SidebarSeparator />
        {/if}

        <!-- Regular Admin Tools -->
        <SidebarGroup>
          <SidebarGroupLabel>เครื่องมือ</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {#each navigationItems.filter(item => item.url?.includes('/admin') && !item.url?.includes('/system') && !item.url?.includes('/faculty')) as item}
                <SidebarMenuItem>
                  <SidebarMenuButton 
                    isActive={isActiveRoute(item.url || '')}
                    class="w-full justify-start"
                  >
                    <a href={item.url} class="flex items-center gap-2 w-full">
                      <svelte:component this={getIcon(item.icon)} class="size-4" />
                      <span>{item.title}</span>
                    </a>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              {/each}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
        <SidebarSeparator />
      {/if}

      <!-- Student/General Navigation -->
      <SidebarGroup>
        <SidebarGroupLabel>เมนูหลัก</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            {#each navigationItems.filter(item => !item.url?.includes('/admin')) as item}
              <SidebarMenuItem>
                <SidebarMenuButton 
                  isActive={isActiveRoute(item.url || '')}
                  class="w-full justify-start"
                >
                  <a href={item.url} class="flex items-center gap-2 w-full">
                    <svelte:component this={getIcon(item.icon)} class="size-4" />
                    <span>{item.title}</span>
                  </a>
                </SidebarMenuButton>
              </SidebarMenuItem>
            {/each}
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
    </SidebarContent>

    <!-- Sidebar Footer with User Info -->
    <SidebarFooter>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger>
              {#snippet child({ props })}
                <button
                  {...props}
                  class="flex items-center gap-3 w-full p-2 rounded-lg data-[state=open]:bg-sidebar-accent data-[state=open]:text-sidebar-accent-foreground hover:bg-sidebar-accent/50"
                >
                <Avatar class="h-8 w-8 rounded-lg">
                  <AvatarImage src={$currentUser?.avatar_url} alt={$currentUser?.first_name || 'User'} />
                  <AvatarFallback class="rounded-lg">
                    {getUserInitials($currentUser)}
                  </AvatarFallback>
                </Avatar>
                <div class="grid flex-1 text-left text-sm leading-tight">
                  <span class="truncate font-semibold">
                    {$currentUser?.first_name} {$currentUser?.last_name}
                  </span>
                  <span class="truncate text-xs text-muted-foreground">
                    {$currentUser?.email}
                  </span>
                </div>
                <IconChevronDown class="ml-auto size-4" />
                </button>
              {/snippet}
            </DropdownMenuTrigger>
            <DropdownMenuContent
              class="w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg"
              side="bottom"
              align="end"
              sideOffset={4}
            >
              <div class="p-2">
                <div class="flex items-center gap-2 px-1 py-1.5 text-left text-sm">
                  <Avatar class="h-8 w-8 rounded-lg">
                    <AvatarImage src={$currentUser?.avatar_url} alt={$currentUser?.first_name || 'User'} />
                    <AvatarFallback class="rounded-lg">
                      {getUserInitials($currentUser)}
                    </AvatarFallback>
                  </Avatar>
                  <div class="grid flex-1 text-left text-sm leading-tight">
                    <span class="truncate font-semibold">
                      {$currentUser?.first_name} {$currentUser?.last_name}
                    </span>
                    <span class="truncate text-xs text-muted-foreground">
                      {$currentUser?.email}
                    </span>
                  </div>
                </div>
              </div>
              <DropdownMenuSeparator />
              
              {#if $currentUser?.admin_role}
                <div class="px-2 py-1">
                  <Badge variant={getRoleBadgeVariant($currentUser.admin_role.admin_level)} class="text-xs">
                    {getRoleDisplayName($currentUser.admin_role.admin_level)}
                  </Badge>
                </div>
                <DropdownMenuSeparator />
              {/if}

              <DropdownMenuItem class="cursor-pointer">
                <a href="/profile" class="flex items-center w-full">
                  <IconUser class="mr-2 size-4" />
                  โปรไฟล์
                </a>
              </DropdownMenuItem>
              
              <DropdownMenuItem class="cursor-pointer">
                <a href="/profile/settings" class="flex items-center w-full">
                  <IconSettings class="mr-2 size-4" />
                  ตั้งค่า
                </a>
              </DropdownMenuItem>
              
              <DropdownMenuSeparator />
              
              <DropdownMenuItem class="cursor-pointer text-destructive" onclick={handleLogout}>
                <IconLogout class="mr-2 size-4" />
                ออกจากระบบ
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarFooter>
  </Sidebar>
</SidebarProvider>
{/if}

<style>
  :global(.sidebar) {
    --sidebar-width: 16rem;
    --sidebar-width-mobile: 16rem;
  }
</style>