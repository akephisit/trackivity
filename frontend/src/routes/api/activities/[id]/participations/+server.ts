import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BACKEND_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ params, cookies, fetch, url }) => {
  const sessionId = cookies.get('session_id');
  if (!sessionId) throw error(401, 'Unauthorized');
  const id = params.id;
  if (!id) throw error(400, 'Missing activity id');

  // Optional status filter passthrough
  const status = url.searchParams.get('status');
  const qs = status ? `?status=${encodeURIComponent(status)}` : '';

  try {
    const res = await fetch(`${BACKEND_URL}/api/activities/${id}/participations${qs}`, {
      headers: {
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      }
    });
    const ct = res.headers.get('content-type') || '';
    if (!res.ok) {
      if (ct.includes('application/json')) {
        const data = await res.json().catch(() => ({}));
        throw error(res.status, (data as any).message || 'Failed to load participations');
      }
      const text = await res.text().catch(() => '');
      throw error(res.status, text || 'Failed to load participations');
    }
    const data = await res.json();
    return json(data);
  } catch (err) {
    console.error('Proxy GET /api/activities/[id]/participations failed:', err);
    throw error(500, 'Internal server error');
  }
};

