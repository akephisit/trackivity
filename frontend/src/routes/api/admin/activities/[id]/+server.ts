import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BACKEND_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

// GET /api/admin/activities/[id]
export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
  const sessionId = cookies.get('session_id');
  if (!sessionId) {
    throw error(401, 'Unauthorized');
  }

  const id = params.id;
  if (!id) {
    throw error(400, 'Missing activity id');
  }

  try {
    // Try admin details first; fallback to public details on 404
    const adminRes = await fetch(`${BACKEND_URL}/api/admin/activities/${id}`, {
      headers: {
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      }
    });

    if (adminRes.ok) {
      const data = await adminRes.json();
      return json(data);
    }

    if (adminRes.status === 404) {
      const publicRes = await fetch(`${BACKEND_URL}/api/activities/${id}`, {
        headers: {
          'Cookie': `session_id=${sessionId}`,
          'X-Session-ID': sessionId
        }
      });
      const data = await publicRes.json().catch(() => ({}));
      if (!publicRes.ok) {
        throw error(publicRes.status, (data as any).message || 'Failed to load activity');
      }
      return json(data);
    }

    const text = await adminRes.text().catch(() => '');
    throw error(adminRes.status, text || 'Failed to load admin activity');
  } catch (err) {
    console.error('Proxy GET /api/admin/activities/[id] failed:', err);
    throw error(500, 'Internal server error');
  }
};

