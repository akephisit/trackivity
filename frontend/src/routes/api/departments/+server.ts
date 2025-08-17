import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BACKEND_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ cookies, fetch }) => {
  const sessionId = cookies.get('session_id');
  if (!sessionId) throw error(401, 'Unauthorized');
  try {
    const res = await fetch(`${BACKEND_URL}/api/departments`, {
      headers: {
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      }
    });
    const ct = res.headers.get('content-type') || '';
    if (!res.ok) {
      if (ct.includes('application/json')) {
        const data = await res.json().catch(() => ({}));
        throw error(res.status, (data as any).message || 'Failed to load departments');
      }
      const text = await res.text().catch(() => '');
      throw error(res.status, text || 'Failed to load departments');
    }
    const data = await res.json();
    return json(data);
  } catch (err) {
    console.error('Proxy GET /api/departments failed:', err);
    throw error(500, 'Internal server error');
  }
};
