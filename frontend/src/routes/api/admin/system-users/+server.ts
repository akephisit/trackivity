import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { PUBLIC_API_URL } from '$env/static/public';

export const GET: RequestHandler = async (event) => {
    try {
        if (!PUBLIC_API_URL) {
            console.error('PUBLIC_API_URL is not configured');
            throw error(500, 'Backend URL not configured');
        }

        const sp = event.url.searchParams;
        const page = parseInt(sp.get('page') || '1');
        const limit = parseInt(sp.get('limit') || '20');
        const offset = parseInt(sp.get('offset') || String((page - 1) * limit));
        const search = sp.get('search') || '';
        const faculty_id = sp.get('faculty_id') || '';
        const department_id = sp.get('department_id') || '';
        const include_admins = sp.get('include_admins') ?? 'true';

        const backendParams = new URLSearchParams({
            limit: String(limit),
            offset: String(offset),
        });
        if (search) backendParams.set('search', search);
        if (faculty_id) backendParams.set('faculty_id', faculty_id);
        if (department_id) backendParams.set('department_id', department_id);
        if (include_admins) backendParams.set('include_admins', include_admins);

        const backendUrl = `${PUBLIC_API_URL}/api/admin/system-users?${backendParams.toString()}`;
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
            console.error('Backend error (system-users):', response.status, errText);
            throw error(response.status, 'Failed to fetch system users');
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

