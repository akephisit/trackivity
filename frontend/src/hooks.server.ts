import { type Handle, type HandleServerError, redirect } from '@sveltejs/kit';
import { PUBLIC_API_URL as PUBLIC_API_URL_RAW } from '$env/static/public';
import type { SessionUser, Permission } from '$lib/types';
import { apiClient } from '$lib/server/api-client';

// Determine if path is considered public (never forces login)
function isPublicRoute(pathname: string): boolean {
  // Public pages and assets; extend as needed
  const PUBLIC_PREFIXES = [
    '/api', // proxied backend API
    '/_app', '/build', '/static', '/assets',
    '/favicon', '/manifest', '/robots.txt', '/service-worker.js',
    '/', '/login', '/admin/login', '/register', '/unauthorized', '/offline', '/qr'
  ];
  return PUBLIC_PREFIXES.some((p) => pathname === p || pathname.startsWith(p + (p.endsWith('/') ? '' : '/')));
}

// Which routes need any authenticated user
function isStudentProtected(pathname: string): boolean {
  const PROTECTED_PREFIXES = ['/student', '/dashboard', '/profile', '/activities'];
  return PROTECTED_PREFIXES.some((p) => pathname === p || pathname.startsWith(p + '/'));
}

// Which routes need admin login
function isAdminProtected(pathname: string): boolean {
  const ADMIN_PREFIX = '/admin';
  // Admin login itself is public
  if (pathname === '/admin/login' || pathname.startsWith('/admin/login')) return false;
  return pathname === ADMIN_PREFIX || pathname.startsWith(ADMIN_PREFIX + '/');
}

// Session validation function (student or admin)
async function validateSession(
  sessionId: string,
  eventFetch: typeof fetch,
  kind: 'student' | 'admin'
): Promise<SessionUser | null> {
    try {
        // Create a mock event object for the API client
        const mockEvent = {
            fetch: eventFetch,
            cookies: { get: (name: string) => (name === 'session_id' ? sessionId : undefined) }
        } as any;

        const endpoint = kind === 'admin' ? '/api/admin/auth/me' : '/api/auth/me';
        const response = await apiClient.get(mockEvent, endpoint);

        if (response.success) {
            // Try common shapes
            const d: any = response.data;
            const user = d?.user ?? d?.data ?? d?.session?.user ?? null;
            return (user ?? null) as SessionUser | null;
        }
    } catch (error) {
        console.error('Session validation failed:', error);
    }

    return null;
}
// Permission helpers (for fine-grained admin pages)
function hasPermission(user: SessionUser, permission: Permission): boolean {
    return user.permissions.includes(permission);
}

function isAdmin(user: SessionUser): boolean {
    return user.admin_role !== undefined;
}

function isFacultyAdmin(user: SessionUser): boolean {
    return user.admin_role?.admin_level === 'FacultyAdmin' || 
           user.admin_role?.admin_level === 'SuperAdmin';
}

function isSuperAdmin(user: SessionUser): boolean {
    return user.admin_role?.admin_level === 'SuperAdmin';
}

// Main handle function
export const handle: Handle = async ({ event, resolve }) => {
    const { url, cookies } = event;
    const pathname = url.pathname;

    // Initialize locals early
    const sessionId = cookies.get('session_id');
    event.locals.user = null;
    event.locals.session_id = sessionId || null;

    // Skip auth enforcement for public routes entirely to avoid loops
    if (isPublicRoute(pathname)) {
        return resolve(event);
    }

    // Validate session if present for protected pages
    if (sessionId) {
        const kind = isAdminProtected(pathname) ? 'admin' : 'student';
        const user = await validateSession(sessionId, event.fetch, kind);
        if (user) {
            event.locals.user = user;
        } else {
            // Only clear cookie if we are about to enforce auth
            cookies.delete('session_id', { path: '/' });
            event.locals.session_id = null;
        }
    }

    // Enforce authentication/authorization
    if (isStudentProtected(pathname) || isAdminProtected(pathname)) {
        if (!event.locals.user) {
            const returnUrl = encodeURIComponent(url.pathname + url.search);
            // Admin sections redirect to admin login
            if (isAdminProtected(pathname)) {
                throw redirect(303, `/admin/login?redirectTo=${returnUrl}`);
            }
            // Student sections go to normal login
            throw redirect(303, `/login?redirectTo=${returnUrl}`);
        }

        // Extra admin checks for admin area
        if (isAdminProtected(pathname)) {
            if (!isAdmin(event.locals.user)) {
                throw redirect(303, '/unauthorized');
            }
            // Example fine-grained checks
            if (pathname.startsWith('/admin/sessions') && !hasPermission(event.locals.user, 'ViewAllSessions')) {
                throw redirect(303, '/unauthorized');
            }
        }
    }

    // Resolve the request
    const response = await resolve(event);

    // Add security headers
    response.headers.set('X-Frame-Options', 'DENY');
    response.headers.set('X-Content-Type-Options', 'nosniff');
    response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');

    const apiUrl = (PUBLIC_API_URL_RAW || '').trim().replace(/\/$/, '');
    const connectSrc = ["'self'", 'ws:', 'wss:'];
    if (apiUrl) connectSrc.push(apiUrl);

    response.headers.set(
        'Content-Security-Policy',
        `default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; connect-src ${connectSrc.join(' ')};`
    );

    return response;
};

// Error handling
export const handleError: HandleServerError = async ({ error, event }) => {
    console.error('Server error:', error);

    // Log error details for debugging
    if (event.locals.user) {
        console.error('User context:', {
            user_id: event.locals.user.user_id,
            email: event.locals.user.email,
            url: event.url.pathname
        });
    }

    // Return user-friendly error message
    return {
        message: 'An unexpected error occurred. Please try again later.',
        code: error instanceof Error ? error.message : 'UNKNOWN_ERROR'
    };
};
