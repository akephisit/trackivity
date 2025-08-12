import { requireAdmin } from '$lib/server/auth';
import type { PageServerLoad } from './$types';
import type { AdminDashboardStats } from '$lib/types/admin';
import { PUBLIC_API_URL } from '$env/static/public';

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async (event) => {
	const user = await requireAdmin(event);
	const sessionId = event.cookies.get('session_id');

	// Re-enable dashboard API calls to see what errors we get
	console.log('[Dashboard] Re-enabling API calls to debug auth issue');
	
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
		console.log(`[Dashboard] Loading stats for session: ${sessionId?.substring(0, 8)}...`);
		const response = await fetch(`${API_BASE_URL}/api/admin/dashboard`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		console.log(`[Dashboard] Stats response status: ${response.status}`);
		if (response.ok) {
			const result = await response.json();
			console.log(`[Dashboard] Stats result:`, result);
			if (result.status === 'success' && result.data) {
				stats = result.data;
			}
		} else {
			console.log(`[Dashboard] Stats failed with status ${response.status}`);
			const errorText = await response.text();
			console.log(`[Dashboard] Error response:`, errorText);
		}
	} catch (error) {
		console.error('[Dashboard] Failed to load dashboard stats:', error);
	}

	// โหลดกิจกรรมล่าสุด (optional) - Skip this for now to focus on stats
	let recentActivities: any[] = [];

	return {
		user,
		stats,
		recentActivities
	};
};