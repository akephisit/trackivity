import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BACKEND_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const POST: RequestHandler = async ({ params, cookies, request }) => {
	const { id: activityId } = params;
	const sessionId = cookies.get('session_id');

	if (!sessionId) {
		throw error(401, 'Unauthorized');
	}

	if (!activityId) {
		throw error(400, 'Activity ID is required');
	}

	try {
		const body = await request.json();

		const response = await fetch(`${BACKEND_URL}/api/activities/${activityId}/checkin`, {
			method: 'POST',
			headers: {
				'Authorization': `Bearer ${sessionId}`,
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(body)
		});

		const data = await response.json();

		if (!response.ok) {
			throw error(response.status, data.message || 'Failed to check in');
		}

		return json(data);
	} catch (err) {
		console.error('Failed to check in:', err);
		if (err instanceof Error && err.message.includes('status')) {
			throw err; // Re-throw HTTP errors
		}
		throw error(500, 'Internal server error');
	}
};