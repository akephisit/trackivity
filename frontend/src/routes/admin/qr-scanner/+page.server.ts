import { error, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load = (async ({ cookies, url, fetch }) => {
	const sessionId = cookies.get('session_id');
	
	if (!sessionId) {
		throw redirect(302, '/admin/login');
	}

	try {
		// Get admin user info to check permissions
		const adminResponse = await fetch('/api/admin/auth/me', {
			headers: {
				'Authorization': `Bearer ${sessionId}`
			}
		});

		if (!adminResponse.ok) {
			throw redirect(302, '/admin/login');
		}

		const adminData = await adminResponse.json();
		
		// Check if user has permission to scan QR codes
		// FacultyAdmin and RegularAdmin should be able to scan
		const user = adminData.user || adminData.data;
		const adminLevel = user?.admin_role?.admin_level;
		
		if (!adminLevel) {
			throw error(403, 'ไม่สามารถระบุสิทธิ์แอดมินได้');
		}
		
		const allowedLevels = ['SuperAdmin', 'FacultyAdmin', 'RegularAdmin'];
		if (!allowedLevels.includes(adminLevel)) {
			throw error(403, 'คุณไม่มีสิทธิ์เข้าถึงระบบสแกน QR Code');
		}

		// Get assigned activities for the admin
		const activitiesResponse = await fetch('/api/admin/activities/assigned', {
			headers: {
				'Authorization': `Bearer ${sessionId}`
			}
		});

		let activities = [];
		if (activitiesResponse.ok) {
			const activitiesData = await activitiesResponse.json();
			activities = activitiesData.data?.activities || [];
		}

		return {
			admin: user,
			activities,
			selectedActivityId: url.searchParams.get('activity_id') || null
		};
	} catch (err) {
		console.error('Failed to load QR scanner page:', err);
		throw redirect(302, '/admin/login');
	}
}) satisfies PageServerLoad;