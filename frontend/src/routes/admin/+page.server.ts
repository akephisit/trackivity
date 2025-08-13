import { requireAdmin } from '$lib/server/auth';
import type { PageServerLoad } from './$types';
import type { AdminDashboardStats } from '$lib/types/admin';

export const load: PageServerLoad = async (event) => {
	const user = await requireAdmin(event);

	// โหลดสถิติแดชบอร์ด
	let stats: AdminDashboardStats = {
		total_users: 0,
		total_activities: 0,
		total_participations: 0,
		active_sessions: 0,
		ongoing_activities: 0,
		user_registrations_today: 0,
		recent_activities: []
	};

	try {
		const response = await event.fetch(`/api/admin/dashboard`);

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
		const response = await event.fetch(`/api/admin/activities?limit=10&recent=true`);

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
