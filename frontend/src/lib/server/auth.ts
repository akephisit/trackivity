import { redirect } from '@sveltejs/kit';
import type { RequestEvent } from '@sveltejs/kit';
import type { User, AdminRole, AdminLevel } from '$lib/types/admin';

export interface AuthenticatedUser extends User {
	admin_role?: AdminRole;
}

// Backend API base URL (server-side only)
const API_BASE_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

/**
 * ตรวจสอบการ authentication ของผู้ใช้
 */
export async function requireAuth(event: RequestEvent): Promise<AuthenticatedUser> {
	const sessionId = event.cookies.get('session_id');
	
	if (!sessionId) {
		const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
		throw redirect(303, `/login?redirectTo=${redirectTo}`);
	}

	try {
        const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
            headers: {
                'Cookie': `session_id=${sessionId}`,
                'X-Session-ID': sessionId
            }
        });

        if (!response.ok) {
            // Session หมดอายุหรือไม่ถูกต้อง
            event.cookies.delete('session_id', { path: '/' });
            const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
            throw redirect(303, `/login?redirectTo=${redirectTo}`);
        }

        const data = await response.json();
        const user = (data as any)?.user ?? (data as any)?.data ?? (data as any)?.session?.user;

        if (!user) {
            event.cookies.delete('session_id', { path: '/' });
            const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
            throw redirect(303, `/login?redirectTo=${redirectTo}`);
        }

        return user as AuthenticatedUser;
	} catch (error) {
		console.error('Auth check failed:', error);
		event.cookies.delete('session_id', { path: '/' });
		const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
		throw redirect(303, `/login?redirectTo=${redirectTo}`);
	}
}

/**
 * ตรวจสอบว่าผู้ใช้เป็นแอดมินหรือไม่
 */
export async function requireAdmin(event: RequestEvent): Promise<AuthenticatedUser> {
	const sessionId = event.cookies.get('session_id');
	
	if (!sessionId) {
		const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
		throw redirect(303, `/admin/login?redirectTo=${redirectTo}`);
	}

	try {
		// Use admin-specific endpoint, call backend directly
        const response = await fetch(`${API_BASE_URL}/api/admin/auth/me`, {
            headers: {
                'Cookie': `session_id=${sessionId}`,
                'X-Session-ID': sessionId
            }
        });

		if (!response.ok) {
			// Session หมดอายุหรือไม่ถูกต้อง
			event.cookies.delete('session_id', { path: '/' });
			const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
			throw redirect(303, `/admin/login?redirectTo=${redirectTo}`);
		}

		const result = await response.json();
		
		if (!result.user || !result.user.admin_role) {
			event.cookies.delete('session_id', { path: '/' });
			const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
			throw redirect(303, `/admin/login?redirectTo=${redirectTo}`);
		}

		return result.user as AuthenticatedUser;
	} catch (error) {
		console.error('Admin auth check failed:', error);
		event.cookies.delete('session_id', { path: '/' });
		const redirectTo = encodeURIComponent(event.url.pathname + event.url.search);
		throw redirect(303, `/admin/login?redirectTo=${redirectTo}`);
	}
}

/**
 * ตรวจสอบระดับแอดมิน
 */
export async function requireAdminLevel(
	event: RequestEvent, 
	requiredLevel: AdminLevel
): Promise<AuthenticatedUser> {
	const user = await requireAdmin(event);
	
	if (!hasAdminLevel(user.admin_role!.admin_level, requiredLevel)) {
		throw redirect(303, '/unauthorized');
	}

	return user;
}

/**
 * ตรวจสอบ permission
 */
export async function requirePermission(
	event: RequestEvent, 
	permission: string
): Promise<AuthenticatedUser> {
	const user = await requireAdmin(event);
	
	if (!user.admin_role?.permissions.includes(permission)) {
		throw redirect(303, '/unauthorized');
	}

	return user;
}

/**
 * ตรวจสอบว่าเป็น SuperAdmin หรือไม่
 */
export async function requireSuperAdmin(event: RequestEvent): Promise<AuthenticatedUser> {
	return requireAdminLevel(event, 'SuperAdmin' as AdminLevel);
}

/**
 * ตรวจสอบว่าเป็น FacultyAdmin หรือ SuperAdmin
 */
export async function requireFacultyAdmin(event: RequestEvent): Promise<AuthenticatedUser> {
	const user = await requireAdmin(event);
	
	const level = user.admin_role!.admin_level;
	if (level !== 'SuperAdmin' && level !== 'FacultyAdmin') {
		throw redirect(303, '/unauthorized');
	}

	return user;
}

/**
 * ตรวจสอบว่าแอดมินสามารถจัดการคณะนี้ได้หรือไม่
 */
export async function requireFacultyAccess(
    event: RequestEvent,
    facultyId: string
): Promise<AuthenticatedUser> {
	const user = await requireAdmin(event);
	
	const level = user.admin_role!.admin_level;
	
	// SuperAdmin สามารถเข้าถึงทุกคณะได้
	if (level === 'SuperAdmin') {
		return user;
	}
	
    // FacultyAdmin สามารถเข้าถึงเฉพาะคณะของตัวเอง (UUID string)
    if (level === 'FacultyAdmin' && user.admin_role!.faculty_id === facultyId) {
        return user;
    }
	
	throw redirect(303, '/unauthorized');
}

/**
 * ฟังก์ชันตรวจสอบระดับแอดมิน
 */
export function hasAdminLevel(userLevel: AdminLevel, requiredLevel: AdminLevel): boolean {
	const hierarchy = {
		'SuperAdmin': 3,
		'FacultyAdmin': 2,
		'RegularAdmin': 1
	};
	
	return hierarchy[userLevel] >= hierarchy[requiredLevel];
}

/**
 * ตรวจสอบการ authentication แบบ optional (ไม่ redirect หากไม่ได้ login)
 */
export async function getAuthUser(event: RequestEvent): Promise<AuthenticatedUser | null> {
	const sessionId = event.cookies.get('session_id');
	
	if (!sessionId) {
		return null;
	}

	try {
        const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
            headers: {
                'Cookie': `session_id=${sessionId}`,
                'X-Session-ID': sessionId
            }
        });

        if (!response.ok) {
            event.cookies.delete('session_id', { path: '/' });
            return null;
        }

        const data = await response.json();
        const user = (data as any)?.user ?? (data as any)?.data ?? (data as any)?.session?.user;

        if (!user) {
            event.cookies.delete('session_id', { path: '/' });
            return null;
        }

        return user as AuthenticatedUser;
	} catch (error) {
		console.error('Optional auth check failed:', error);
		event.cookies.delete('session_id', { path: '/' });
		return null;
	}
}

/**
 * Logout helper
 */
export async function logout(event: RequestEvent): Promise<void> {
    const sessionId = event.cookies.get('session_id');
	
	if (sessionId) {
		try {
			// Call backend directly to avoid recursive call into this same SvelteKit route
			await fetch(`${API_BASE_URL}/api/auth/logout`, {
				method: 'POST',
				headers: {
					'Cookie': `session_id=${sessionId}`,
					'X-Session-ID': sessionId
				}
			});
		} catch (error) {
			console.error('Logout request failed:', error);
		}
	}
	
    event.cookies.delete('session_id', { path: '/' });
}
