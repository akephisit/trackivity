import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_BASE_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ params, cookies }) => {
	const sessionId = cookies.get('session_id');
	if (!sessionId) {
		return json({ success: false, error: 'Not authenticated' }, { status: 401 });
	}

	const departmentId = params.id;
	if (!departmentId) {
		return json({ success: false, error: 'Department ID is required' }, { status: 400 });
	}

	try {
		const response = await fetch(`${API_BASE_URL}/api/departments/${departmentId}/available-admins`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		const result = await response.json();

		if (!response.ok) {
			return json({ 
				success: false, 
				error: result.message || 'Failed to fetch available admins' 
			}, { status: response.status });
		}

		return json({
			success: true,
			data: result.data
		});
	} catch (error) {
		console.error('Failed to fetch available admins:', error);
		return json({ 
			success: false, 
			error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์' 
		}, { status: 500 });
	}
};