import { createRawSnippet } from 'svelte';
import type { User } from '$lib/types/admin';

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
    const statusLabels: Record<string, string> = {
        // New semantics aligned with admin page
        online: 'ใช้งานอยู่',
        offline: 'ไม่ออนไลน์',
        disabled: 'ปิดใช้งาน',
        // Backward compatibility
        active: 'ใช้งานอยู่',
        inactive: 'ไม่ออนไลน์',
        suspended: 'ถูกระงับ'
    };
    return statusLabels[status] || status;
}

function getStatusVariant(status: string): "default" | "secondary" | "destructive" | "outline" {
    // Map to visual variants; customize classes below
    const variants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline'> = {
        online: 'default',
        active: 'default',
        offline: 'secondary',
        inactive: 'secondary',
        disabled: 'destructive',
        suspended: 'destructive'
    };
    return variants[status] || 'outline';
}

// User Profile Cell Snippet
export const UserProfileCell = createRawSnippet<[{ user: User }]>((getProps) => {
	const { user } = getProps();
	return {
		render: () => `
			<div class="flex items-center gap-3 min-w-0">
				<div class="h-8 w-8 flex-shrink-0 rounded-full bg-gray-200 flex items-center justify-center text-sm font-medium">
					${getInitials(user.first_name, user.last_name)}
				</div>
				<div class="min-w-0 flex-1">
					<div class="font-medium text-sm truncate">
						${user.first_name} ${user.last_name}
					</div>
					<div class="text-xs text-gray-500 truncate">
						${user.email}
					</div>
				</div>
			</div>
		`
	};
});

// Email Cell Snippet
export const EmailCell = createRawSnippet<[{ user: User }]>((getProps) => {
	const { user } = getProps();
	const isVerified = !!user.email_verified_at;
	return {
		render: () => `
			<div class="flex items-center gap-2">
				<svg class="h-4 w-4 text-gray-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 4.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
				</svg>
				<a href="mailto:${user.email}" class="text-sm hover:underline truncate" title="${user.email}">
					${user.email}
				</a>
				${isVerified ? '<svg class="h-3 w-3 text-green-600 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/></svg>' : ''}
			</div>
		`
	};
});

// Identifier Cell Snippet
export const IdentifierCell = createRawSnippet<[{ user: User }]>((getProps) => {
    const { user } = getProps();
    return {
        render: () => `
            <div class="text-sm">
                ${user.student_id || user.employee_id || '-'}
            </div>
        `
    };
});

// Role Badge Snippet
export const RoleBadge = createRawSnippet<[{ role: string }]>((getProps) => {
	const { role } = getProps();
	const variant = getRoleVariant(role);
	const label = getRoleLabel(role);
    // Align colors with admin status badges
    const variantClasses = {
        default: 'bg-green-100 text-green-800',
        secondary: 'bg-yellow-100 text-yellow-800',
        destructive: 'bg-red-100 text-red-800',
        outline: 'border border-gray-200 text-gray-900'
    };
	return {
		render: () => `
			<span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${variantClasses[variant]}">
				${label}
			</span>
		`
	};
});

// Status Badge Snippet
export const StatusBadge = createRawSnippet<[{ status: string }]>((getProps) => {
    const { status } = getProps();
    const variant = getStatusVariant(status);
    const label = getStatusLabel(status);
    const variantClasses = {
        default: 'bg-green-100 text-green-800',
        secondary: 'bg-yellow-100 text-yellow-800',
        destructive: 'bg-red-100 text-red-800',
        outline: 'border border-gray-200 text-gray-900'
    } as const;
    const dotClasses: Record<string, string> = {
        default: 'bg-green-500',
        secondary: 'bg-yellow-500',
        destructive: 'bg-red-500',
        outline: 'bg-gray-400'
    };
    return {
        render: () => `
            <span class="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${variantClasses[variant]}">
                <span class="inline-block h-2 w-2 rounded-full mr-1 ${dotClasses[variant]}"></span>
                ${label}
            </span>
        `
    };
});

// Faculty Cell Snippet
export const FacultyCell = createRawSnippet<[{ user: User }]>((getProps) => {
	const { user } = getProps();
	return {
		render: () => `
			<div class="text-sm truncate" title="${user.faculty?.name || '-'}">
				${user.faculty?.name || '-'}
			</div>
		`
	};
});

// Department Cell Snippet
export const DepartmentCell = createRawSnippet<[{ user: User }]>((getProps) => {
	const { user } = getProps();
	return {
		render: () => `
			<div class="text-sm truncate" title="${user.department?.name || '-'}">
				${user.department?.name || '-'}
			</div>
		`
	};
});

// Phone Cell Snippet
export const PhoneCell = createRawSnippet<[{ phone?: string }]>((getProps) => {
	const { phone } = getProps();
	return {
		render: () => `
			<div class="flex items-center gap-2">
				${phone ? `
					<svg class="h-4 w-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z"/>
					</svg>
					<a href="tel:${phone}" class="text-sm hover:underline">${phone}</a>
				` : `
					<span class="text-sm text-gray-500">-</span>
				`}
			</div>
		`
	};
});

// Last Login Cell Snippet
export const LastLoginCell = createRawSnippet<[{ lastLogin?: string }]>((getProps) => {
	const { lastLogin } = getProps();
	return {
		render: () => `
			<div class="text-sm" title="${lastLogin ? formatDate(lastLogin) : ''}">
				${formatRelativeTime(lastLogin)}
			</div>
		`
	};
});

// Created At Cell Snippet
export const CreatedAtCell = createRawSnippet<[{ createdAt: string }]>((getProps) => {
	const { createdAt } = getProps();
	return {
		render: () => `
			<div class="flex items-center gap-2">
				<svg class="h-4 w-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v14a2 2 0 002 2z"/>
				</svg>
				<span class="text-sm">${formatDate(createdAt)}</span>
			</div>
		`
	};
});

// Email Verified Cell Snippet
export const EmailVerifiedCell = createRawSnippet<[{ emailVerifiedAt?: string }]>((getProps) => {
	const { emailVerifiedAt } = getProps();
	return {
		render: () => `
			<div class="flex items-center justify-center">
				${emailVerifiedAt ? `
					<svg class="h-4 w-4 text-green-600" fill="currentColor" viewBox="0 0 20 20" title="ยืนยันแล้ว">
						<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
					</svg>
				` : `
					<svg class="h-4 w-4 text-orange-500" fill="currentColor" viewBox="0 0 20 20" title="ยังไม่ยืนยัน">
						<path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
					</svg>
				`}
			</div>
		`
	};
});

// User Actions Menu Snippet
export const UserActionsMenu = createRawSnippet<[{ user: User }]>((getProps) => {
	const { user } = getProps();
	return {
		render: () => `
			<button class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 hover:bg-accent hover:text-accent-foreground h-8 w-8">
				<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"/>
				</svg>
				<span class="sr-only">เปิดเมนู</span>
			</button>
		`
	};
});
