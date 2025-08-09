<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { 
		IconUser, 
		IconMail, 
		IconPhone, 
		IconCalendar, 
		IconDots,
		IconEdit,
		IconTrash,
		IconUserCheck,
		IconUserX,
		IconUserPause,
		IconEye,
		IconCopy,
		IconDownload
	} from '@tabler/icons-svelte/icons';
	import type { User } from '$lib/types/admin';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{
		viewUser: User;
		editUser: User;
		updateStatus: { user: User; status: 'active' | 'inactive' | 'suspended' };
		updateRole: { user: User; role: 'student' | 'faculty' | 'staff' | 'admin' };
		deleteUser: User;
		transferUser: User;
		viewActivity: User;
	}>();

	// Helper functions for display formatting
	function getInitials(firstName: string, lastName: string): string {
		return `${firstName.charAt(0)}${lastName.charAt(0)}`.toUpperCase();
	}

	function formatDate(dateString: string | undefined): string {
		if (!dateString) return '-';
		return new Date(dateString).toLocaleDateString('th-TH', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
	}

	function formatRelativeTime(dateString: string | undefined): string {
		if (!dateString) return 'ไม่เคย';
		
		const date = new Date(dateString);
		const now = new Date();
		const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);
		
		if (diffInSeconds < 60) return 'เมื่อสักครู่';
		if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)} นาทีที่แล้ว`;
		if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)} ชั่วโมงที่แล้ว`;
		if (diffInSeconds < 604800) return `${Math.floor(diffInSeconds / 86400)} วันที่แล้ว`;
		
		return formatDate(dateString);
	}

	function getRoleLabel(role: string): string {
		const roleLabels = {
			student: 'นักศึกษา',
			faculty: 'อาจารย์',
			staff: 'เจ้าหน้าที่',
			admin: 'ผู้ดูแลระบบ'
		};
		return roleLabels[role as keyof typeof roleLabels] || role;
	}

	function getRoleVariant(role: string): "default" | "secondary" | "destructive" | "outline" {
		const variants = {
			student: 'outline' as const,
			faculty: 'default' as const,
			staff: 'secondary' as const,
			admin: 'destructive' as const
		};
		return variants[role as keyof typeof variants] || 'outline';
	}

	function getStatusLabel(status: string): string {
		const statusLabels = {
			active: 'เปิดใช้งาน',
			inactive: 'ปิดใช้งาน',
			suspended: 'ถูกระงับ'
		};
		return statusLabels[status as keyof typeof statusLabels] || status;
	}

	function getStatusVariant(status: string): "default" | "secondary" | "destructive" | "outline" {
		const variants = {
			active: 'default' as const,
			inactive: 'secondary' as const,
			suspended: 'destructive' as const
		};
		return variants[status as keyof typeof variants] || 'outline';
	}

	// Component props will be passed via renderSnippet calls
	export let user: User | undefined = undefined;
	export let role: string | undefined = undefined;
	export let status: string | undefined = undefined;
	export let phone: string | undefined = undefined;
	export let lastLogin: string | undefined = undefined;
	export let createdAt: string | undefined = undefined;
	export let emailVerifiedAt: string | undefined = undefined;
	export let checked: boolean | undefined = undefined;
	export let indeterminate: boolean | undefined = undefined;
	export let onCheckedChange: ((value: boolean) => void) | undefined = undefined;
</script>

<!-- Data Table Checkbox Component -->
{#if checked !== undefined}
<div class="flex items-center">
	<Checkbox 
		{checked}
		{indeterminate}
		onCheckedChange={onCheckedChange}
		aria-label="เลือกรายการ"
	/>
</div>
{/if}

<!-- User Profile Cell Component -->
{#if user}
<div class="flex items-center gap-3 min-w-0">
	<Avatar class="h-8 w-8 flex-shrink-0">
		{#if user.avatar}
			<AvatarImage src={user.avatar} alt="{user.first_name} {user.last_name}" />
		{/if}
		<AvatarFallback class="text-sm">
			{getInitials(user.first_name, user.last_name)}
		</AvatarFallback>
	</Avatar>
	<div class="min-w-0 flex-1">
		<div class="font-medium text-sm truncate">
			{user.first_name} {user.last_name}
		</div>
		<div class="text-xs text-muted-foreground truncate">
			{user.email}
		</div>
	</div>
</div>
{/if}

<!-- Email Cell Component -->
{#if user}
<div class="flex items-center gap-2">
	<IconMail class="h-4 w-4 text-muted-foreground flex-shrink-0" />
	<a 
		href="mailto:{user.email}" 
		class="text-sm hover:underline truncate"
		title={user.email}
	>
		{user.email}
	</a>
	{#if user.email_verified_at}
		<IconUserCheck class="h-3 w-3 text-green-600 flex-shrink-0" title="อีเมลได้รับการยืนยัน" />
	{/if}
</div>
{/if}

<!-- Identifier Cell Component (Student ID / Employee ID) -->
{#if user}
<div class="text-sm font-mono">
	{user.student_id || user.employee_id || '-'}
</div>
{/if}

<!-- Role Badge Component -->
{#if role}
<Badge variant={getRoleVariant(role)} class="text-xs">
	{getRoleLabel(role)}
</Badge>
{/if}

<!-- Status Badge Component -->
{#if status}
<Badge variant={getStatusVariant(status)} class="text-xs">
	{getStatusLabel(status)}
</Badge>
{/if}

<!-- Faculty Cell Component -->
{#if user}
<div class="text-sm truncate" title={user.faculty?.name || '-'}>
	{user.faculty?.name || '-'}
</div>
{/if}

<!-- Department Cell Component -->
{#if user}
<div class="text-sm truncate" title={user.department?.name || '-'}>
	{user.department?.name || '-'}
</div>
{/if}

<!-- Phone Cell Component -->
{#if phone !== undefined}
<div class="flex items-center gap-2">
	{#if phone}
		<IconPhone class="h-4 w-4 text-muted-foreground" />
		<a href="tel:{phone}" class="text-sm hover:underline">
			{phone}
		</a>
	{:else}
		<span class="text-sm text-muted-foreground">-</span>
	{/if}
</div>
{/if}

<!-- Last Login Cell Component -->
{#if lastLogin !== undefined}
<div class="text-sm" title={lastLogin ? formatDate(lastLogin) : ''}>
	{formatRelativeTime(lastLogin)}
</div>
{/if}

<!-- Created At Cell Component -->
{#if createdAt}
<div class="flex items-center gap-2">
	<IconCalendar class="h-4 w-4 text-muted-foreground" />
	<span class="text-sm">{formatDate(createdAt)}</span>
</div>
{/if}

<!-- Email Verified Cell Component -->
{#if emailVerifiedAt !== undefined}
<div class="flex items-center justify-center">
	{#if emailVerifiedAt}
		<IconUserCheck class="h-4 w-4 text-green-600" title="ยืนยันแล้ว" />
	{:else}
		<IconUserX class="h-4 w-4 text-orange-500" title="ยังไม่ยืนยัน" />
	{/if}
</div>
{/if}

<!-- User Actions Menu Component -->
{#if user}
<DropdownMenu.Root>
	<DropdownMenu.Trigger asChild let:builder>
		<Button
			variant="ghost"
			size="icon"
			class="h-8 w-8"
			builders={[builder]}
		>
			<IconDots class="h-4 w-4" />
			<span class="sr-only">เปิดเมนู</span>
		</Button>
	</DropdownMenu.Trigger>
	
	<DropdownMenu.Content align="end" class="w-56">
		<!-- View User Details -->
		<DropdownMenu.Item on:click={() => dispatch('viewUser', user)}>
			<IconEye class="mr-2 h-4 w-4" />
			ดูรายละเอียด
		</DropdownMenu.Item>
		
		<!-- Edit User -->
		<DropdownMenu.Item on:click={() => dispatch('editUser', user)}>
			<IconEdit class="mr-2 h-4 w-4" />
			แก้ไขข้อมูล
		</DropdownMenu.Item>
		
		<!-- Copy User ID -->
		<DropdownMenu.Item on:click={() => navigator.clipboard.writeText(user.id)}>
			<IconCopy class="mr-2 h-4 w-4" />
			คัดลอก ID
		</DropdownMenu.Item>
		
		<DropdownMenu.Separator />
		
		<!-- Status Management -->
		<DropdownMenu.Sub>
			<DropdownMenu.SubTrigger>
				<IconUserCheck class="mr-2 h-4 w-4" />
				เปลี่ยนสถานะ
			</DropdownMenu.SubTrigger>
			<DropdownMenu.SubContent>
				{#if user.status !== 'active'}
					<DropdownMenu.Item on:click={() => dispatch('updateStatus', { user, status: 'active' })}>
						<IconUserCheck class="mr-2 h-4 w-4 text-green-600" />
						เปิดใช้งาน
					</DropdownMenu.Item>
				{/if}
				{#if user.status !== 'inactive'}
					<DropdownMenu.Item on:click={() => dispatch('updateStatus', { user, status: 'inactive' })}>
						<IconUserX class="mr-2 h-4 w-4 text-orange-500" />
						ปิดใช้งาน
					</DropdownMenu.Item>
				{/if}
				{#if user.status !== 'suspended'}
					<DropdownMenu.Item on:click={() => dispatch('updateStatus', { user, status: 'suspended' })}>
						<IconUserPause class="mr-2 h-4 w-4 text-red-600" />
						ระงับการใช้งาน
					</DropdownMenu.Item>
				{/if}
			</DropdownMenu.SubContent>
		</DropdownMenu.Sub>
		
		<!-- Role Management -->
		<DropdownMenu.Sub>
			<DropdownMenu.SubTrigger>
				<IconUser class="mr-2 h-4 w-4" />
				เปลี่ยนบทบาท
			</DropdownMenu.SubTrigger>
			<DropdownMenu.SubContent>
				{#if user.role !== 'student'}
					<DropdownMenu.Item on:click={() => dispatch('updateRole', { user, role: 'student' })}>
						นักศึกษา
					</DropdownMenu.Item>
				{/if}
				{#if user.role !== 'faculty'}
					<DropdownMenu.Item on:click={() => dispatch('updateRole', { user, role: 'faculty' })}>
						อาจารย์
					</DropdownMenu.Item>
				{/if}
				{#if user.role !== 'staff'}
					<DropdownMenu.Item on:click={() => dispatch('updateRole', { user, role: 'staff' })}>
						เจ้าหน้าที่
					</DropdownMenu.Item>
				{/if}
				{#if user.role !== 'admin'}
					<DropdownMenu.Item on:click={() => dispatch('updateRole', { user, role: 'admin' })}>
						ผู้ดูแลระบบ
					</DropdownMenu.Item>
				{/if}
			</DropdownMenu.SubContent>
		</DropdownMenu.Sub>
		
		<DropdownMenu.Separator />
		
		<!-- Additional Actions -->
		<DropdownMenu.Item on:click={() => dispatch('viewActivity', user)}>
			<IconCalendar class="mr-2 h-4 w-4" />
			ดูกิจกรรม
		</DropdownMenu.Item>
		
		<DropdownMenu.Item on:click={() => dispatch('transferUser', user)}>
			<IconDownload class="mr-2 h-4 w-4" />
			ย้ายคณะ/สาขา
		</DropdownMenu.Item>
		
		<DropdownMenu.Separator />
		
		<!-- Dangerous Actions -->
		<DropdownMenu.Item 
			class="text-red-600 focus:text-red-600 focus:bg-red-50"
			on:click={() => dispatch('deleteUser', user)}
		>
			<IconTrash class="mr-2 h-4 w-4" />
			ลบผู้ใช้
		</DropdownMenu.Item>
	</DropdownMenu.Content>
</DropdownMenu.Root>
{/if}