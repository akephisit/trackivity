import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { PUBLIC_API_URL } from '$env/static/public';

export const GET: RequestHandler = async (event) => {
    try {
        if (!PUBLIC_API_URL) {
            console.error('PUBLIC_API_URL is not configured');
            throw error(500, 'Backend URL not configured');
        }

        const backendUrl = `${PUBLIC_API_URL}/api/admin/user-statistics`;
        const response = await event.fetch(backendUrl, {
            method: 'GET',
            headers: {
                'Cookie': `session_id=${event.cookies.get('session_id')}`,
                'Content-Type': 'application/json'
            }
        });

        if (!response.ok) {
            let errText = '';
            try { errText = await response.text(); } catch {}
            console.error('Backend error (user-statistics):', response.status, errText);
            throw error(response.status, 'Failed to fetch user statistics');
        }

        const data = await response.json();
        return json(data);
    } catch (err: any) {
        if (err.status) throw err;
        if (err.name === 'TypeError' && err.message.includes('fetch')) {
            throw error(503, 'Unable to connect to backend');
        }
        throw error(500, 'Internal error');
    }
};

