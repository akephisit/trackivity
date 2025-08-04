<script lang="ts">
	import CalendarIcon from "@tabler/icons-svelte/icons/calendar";
	import ChartBarIcon from "@tabler/icons-svelte/icons/chart-bar";
	import DashboardIcon from "@tabler/icons-svelte/icons/dashboard";
	import FileDescriptionIcon from "@tabler/icons-svelte/icons/file-description";
	import HelpIcon from "@tabler/icons-svelte/icons/help";
	import ReportIcon from "@tabler/icons-svelte/icons/report";
	import SettingsIcon from "@tabler/icons-svelte/icons/settings";
	import UsersIcon from "@tabler/icons-svelte/icons/users";
	import NavMain from "./nav-main.svelte";
	import NavSecondary from "./nav-secondary.svelte";
	import NavUser from "./nav-user.svelte";
	import * as Sidebar from "$lib/components/ui/sidebar/index.js";
	import { authStore } from '$lib/stores/auth';
	import type { ComponentProps } from "svelte";

	const data = {
		navMain: [
			{
				title: "แดชบอร์ด",
				url: "/dashboard",
				icon: DashboardIcon,
			},
			{
				title: "กิจกรรม",
				url: "/dashboard/activities",
				icon: CalendarIcon,
			},
			{
				title: "การเข้าร่วม",
				url: "/dashboard/participations",
				icon: UsersIcon,
			},
			{
				title: "รายงาน",
				url: "/dashboard/reports",
				icon: ChartBarIcon,
			},
		],
		navSecondary: [
			{
				title: "การตั้งค่า",
				url: "/dashboard/settings",
				icon: SettingsIcon,
			},
			{
				title: "คู่มือการใช้งาน",
				url: "/dashboard/help",
				icon: HelpIcon,
			},
		],
		adminNav: [
			{
				title: "จัดการผู้ใช้",
				url: "/dashboard/admin/users",
				icon: UsersIcon,
			},
			{
				title: "รายงานระบบ",
				url: "/dashboard/admin/reports",
				icon: ReportIcon,
			},
		],
	};

	let { ...restProps }: ComponentProps<typeof Sidebar.Root> = $props();
</script>

<Sidebar.Root collapsible="offcanvas" {...restProps}>
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton class="data-[slot=sidebar-menu-button]:!p-1.5">
					{#snippet child({ props })}
						<a href="/dashboard" {...props}>
							<div class="w-6 h-6 bg-primary rounded-md flex items-center justify-center">
								<span class="text-white font-bold text-xs">T</span>
							</div>
							<span class="text-base font-semibold">Trackivity</span>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>
	<Sidebar.Content>
		<NavMain items={data.navMain} />
		<NavSecondary items={data.navSecondary} class="mt-auto" />
	</Sidebar.Content>
	<Sidebar.Footer>
		{#if $authStore.user}
			<NavUser user={{
				name: `${$authStore.user.first_name} ${$authStore.user.last_name}`,
				email: $authStore.user.email,
				avatar: ""
			}} />
		{/if}
	</Sidebar.Footer>
</Sidebar.Root>
