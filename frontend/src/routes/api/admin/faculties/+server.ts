import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { PUBLIC_API_URL } from '$env/static/public';

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ fetch, cookies }) => {
	const sessionId = cookies.get('session_id');
	
	if (!sessionId) {
		return json({ error: 'Unauthorized' }, { status: 401 });
	}

	try {
		const response = await fetch(`${API_BASE_URL}/api/admin/faculties`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});
		
		if (!response.ok) {
			return json({ error: 'Failed to fetch faculties' }, { status: response.status });
		}

		const data = await response.json();
		return json(data);
	} catch (error) {
		console.error('Failed to fetch admin faculties:', error);
		return json({ error: 'Internal server error' }, { status: 500 });
	}
};