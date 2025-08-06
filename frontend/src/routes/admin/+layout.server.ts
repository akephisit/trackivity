import { requireAdmin } from '$lib/server/auth';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	// Skip authentication for admin login page
	if (event.url.pathname === '/admin/login') {
		return {};
	}
	
	// ตรวจสอบว่าผู้ใช้เป็นแอดมิน
	const user = await requireAdmin(event);

	return {
		user,
		admin_role: user.admin_role
	};
};