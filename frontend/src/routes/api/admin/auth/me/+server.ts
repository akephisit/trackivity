import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BACKEND_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ cookies }) => {
	const sessionId = cookies.get('session_id');

	if (!sessionId) {
		throw error(401, 'Unauthorized');
	}

	try {
		const response = await fetch(`${BACKEND_URL}/api/admin/auth/me`, {
			method: 'GET',
			headers: {
				'Authorization': `Bearer ${sessionId}`,
				'Content-Type': 'application/json',
			},
		});

		const data = await response.json();

		if (!response.ok) {
			throw error(response.status, data.message || 'Failed to get admin info');
		}

		return json(data);
	} catch (err) {
		console.error('Failed to get admin info:', err);
		throw error(500, 'Internal server error');
	}
};