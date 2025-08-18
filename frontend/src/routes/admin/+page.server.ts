import { requireAdmin } from '$lib/server/auth';
import type { PageServerLoad } from './$types';
import type { AdminDashboardStats } from '$lib/types/admin';
import { api } from '$lib/server/api-client';

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
        const response = await api.get<AdminDashboardStats>(event, '/api/admin/dashboard', undefined, {
            throwOnHttpError: false
        });
        if (response.success && response.data) {
            stats = response.data;
        }
    } catch (error) {
        console.warn('Failed to load dashboard stats:', error);
    }

	// โหลดกิจกรรมล่าสุด (optional)
	let recentActivities: any[] = [];
    try {
        const response = await api.get<any[]>(
            event,
            '/api/admin/activities',
            { limit: '10', recent: 'true' },
            { throwOnHttpError: false }
        );
        if (response.success && response.data) {
            recentActivities = response.data || [];
        }
    } catch (error) {
        console.warn('Failed to load recent activities:', error);
    }

	return {
		user,
		stats,
		recentActivities
	};
};
