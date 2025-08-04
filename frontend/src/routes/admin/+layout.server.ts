import { requireAdmin } from '$lib/server/auth';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async (event) => {
	// ตรวจสอบว่าผู้ใช้เป็นแอดมิน
	const user = await requireAdmin(event);

	return {
		user,
		admin_role: user.admin_role
	};
};