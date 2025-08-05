import { requireAdmin } from '$lib/server/auth';
import type { PageServerLoad } from './$types';
import type { AdminDashboardStats } from '$lib/types/admin';

const API_BASE_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async (event) => {
	const user = await requireAdmin(event);
	const sessionId = event.cookies.get('session_id');

	// โหลดสถิติแดชบอร์ด
	let stats: AdminDashboardStats = {
		total_users: 0,
		total_admins: 0,
		total_faculties: 0,
		recent_activities: 0
	};

	try {
		const response = await fetch(`${API_BASE_URL}/api/admin/dashboard`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		if (response.ok) {
			const result = await response.json();
			if (result.status === 'success' && result.data) {
				stats = result.data;
			}
		}
	} catch (error) {
		console.error('Failed to load dashboard stats:', error);
	}

	// โหลดกิจกรรมล่าสุด (optional)
	let recentActivities: any[] = [];
	try {
		const response = await fetch(`${API_BASE_URL}/api/admin/activities?limit=10&recent=true`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		if (response.ok) {
			const result = await response.json();
			if (result.status === 'success' && result.data) {
				recentActivities = result.data || [];
			}
		}
	} catch (error) {
		console.error('Failed to load recent activities:', error);
	}

	return {
		user,
		stats,
		recentActivities
	};
};