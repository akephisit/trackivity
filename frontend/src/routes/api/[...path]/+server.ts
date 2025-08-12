import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { PUBLIC_API_URL } from '$env/static/public';

// Generic proxy for backend API. Any /api/* path not matched by a more specific
// route will be forwarded to the Rust backend, attaching the httpOnly session
// cookie so the backend can authenticate the request.

async function proxy(event: Parameters<RequestHandler>[0]) {
  if (!PUBLIC_API_URL) {
    throw error(500, 'Backend URL not configured');
  }

  const { url, request, cookies } = event;

  // Build target URL by replacing the origin with backend base
  const target = new URL(url);
  const backendUrl = new URL(PUBLIC_API_URL);
  target.protocol = backendUrl.protocol;
  target.host = backendUrl.host;
  target.port = backendUrl.port;

  // Forward request headers, excluding host and cookies we'll set explicitly
  const headers = new Headers(request.headers);
  headers.delete('host');
  headers.delete('cookie');

  // Attach session cookie for backend authentication
  const sessionId = cookies.get('session_id');
  if (sessionId) {
    headers.set('cookie', `session_id=${sessionId}`);
  }

  // Preserve method and body as applicable
  const method = request.method;
  const body = ['GET', 'HEAD'].includes(method) ? undefined : await request.arrayBuffer();

  // Perform the proxied request using server-side fetch
  const resp = await event.fetch(target.toString(), {
    method,
    headers,
    body: body ? body : undefined,
    // Let backend handle CORS; same-origin to SvelteKit
  });

  // Stream response back to the client
  const respHeaders = new Headers(resp.headers);
  // Remove hop-by-hop headers if any
  respHeaders.delete('transfer-encoding');

  return new Response(resp.body, {
    status: resp.status,
    statusText: resp.statusText,
    headers: respHeaders
  });
}

export const GET: RequestHandler = async (event) => proxy(event);
export const POST: RequestHandler = async (event) => proxy(event);
export const PUT: RequestHandler = async (event) => proxy(event);
export const PATCH: RequestHandler = async (event) => proxy(event);
export const DELETE: RequestHandler = async (event) => proxy(event);
