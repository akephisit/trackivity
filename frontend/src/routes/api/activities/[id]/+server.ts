import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BACKEND_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
  const sessionId = cookies.get('session_id');
  if (!sessionId) throw error(401, 'Unauthorized');
  const id = params.id;
  try {
    const res = await fetch(`${BACKEND_URL}/api/activities/${id}`, {
      headers: {
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      }
    });
    const ct = res.headers.get('content-type') || '';
    if (!res.ok) {
      if (ct.includes('application/json')) {
        const data = await res.json().catch(() => ({}));
        throw error(res.status, (data as any).message || 'Failed to load activity');
      }
      const text = await res.text().catch(() => '');
      throw error(res.status, text || 'Failed to load activity');
    }
    const data = await res.json();
    return json(data);
  } catch (err) {
    console.error('Proxy GET /api/activities/[id] failed:', err);
    throw error(500, 'Internal server error');
  }
};

export const PUT: RequestHandler = async ({ params, cookies, fetch, request }) => {
  const sessionId = cookies.get('session_id');
  if (!sessionId) throw error(401, 'Unauthorized');
  const id = params.id;
  const body = await request.text();
  try {
    const res = await fetch(`${BACKEND_URL}/api/activities/${id}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      },
      body
    });
    const ct = res.headers.get('content-type') || '';
    if (!res.ok) {
      if (ct.includes('application/json')) {
        const data = await res.json().catch(() => ({}));
        throw error(res.status, (data as any).message || 'Failed to update activity');
      }
      const text = await res.text().catch(() => '');
      throw error(res.status, text || 'Failed to update activity');
    }
    const data = ct.includes('application/json') ? await res.json().catch(() => ({})) : {};
    return json(data);
  } catch (err) {
    console.error('Proxy PUT /api/activities/[id] failed:', err);
    throw error(500, 'Internal server error');
  }
};

export const DELETE: RequestHandler = async ({ params, cookies, fetch }) => {
  const sessionId = cookies.get('session_id');
  if (!sessionId) throw error(401, 'Unauthorized');
  const id = params.id;
  try {
    const res = await fetch(`${BACKEND_URL}/api/activities/${id}`, {
      method: 'DELETE',
      headers: {
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      }
    });
    const ct = res.headers.get('content-type') || '';
    if (!res.ok) {
      if (ct.includes('application/json')) {
        const data = await res.json().catch(() => ({}));
        throw error(res.status, (data as any).message || 'Failed to delete activity');
      }
      const text = await res.text().catch(() => '');
      throw error(res.status, text || 'Failed to delete activity');
    }
    return json({ success: true });
  } catch (err) {
    console.error('Proxy DELETE /api/activities/[id] failed:', err);
    throw error(500, 'Internal server error');
  }
};
