import { type Handle, type HandleServerError } from '@sveltejs/kit';
import { redirect } from '@sveltejs/kit';

// Types for session management
interface AdminRole {
    id: string;
    admin_level: 'SuperAdmin' | 'FacultyAdmin' | 'RegularAdmin';
    faculty_id?: string;
    permissions: string[];
}

interface SessionUser {
    user_id: string;
    student_id: string;
    email: string;
    first_name: string;
    last_name: string;
    department_id?: string;
    admin_role?: AdminRole;
    permissions: string[];
    faculty_id?: string;
    session_id: string;
}

// Backend API base URL
const API_BASE_URL = process.env.BACKEND_URL || 'http://localhost:3000';

// Session validation function
async function validateSession(sessionId: string): Promise<SessionUser | null> {
    try {
        const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
            headers: {
                'Cookie': `session_id=${sessionId}`,
                'X-Session-ID': sessionId
            }
        });

        if (response.ok) {
            return await response.json();
        }
    } catch (error) {
        console.error('Session validation failed:', error);
    }

    return null;
}

// Protected routes configuration
const PROTECTED_ROUTES = [
    '/dashboard',
    '/profile',
    '/activities',
    '/admin'
];

const ADMIN_ROUTES = [
    '/admin'
];

const FACULTY_ADMIN_ROUTES = [
    '/admin/faculty',
    '/admin/students',
    '/admin/reports'
];

const SUPER_ADMIN_ROUTES = [
    '/admin/system',
    '/admin/faculties',
    '/admin/sessions'
];

// Route protection helper
function isProtectedRoute(pathname: string): boolean {
    return PROTECTED_ROUTES.some(route => pathname.startsWith(route));
}

function requiresAdmin(pathname: string): boolean {
    return ADMIN_ROUTES.some(route => pathname.startsWith(route));
}

function requiresFacultyAdmin(pathname: string): boolean {
    return FACULTY_ADMIN_ROUTES.some(route => pathname.startsWith(route));
}

function requiresSuperAdmin(pathname: string): boolean {
    return SUPER_ADMIN_ROUTES.some(route => pathname.startsWith(route));
}

// Permission checking
function hasPermission(user: SessionUser, permission: string): boolean {
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
    
    // Get session ID from cookie
    const sessionId = cookies.get('session_id');
    
    // Initialize locals
    event.locals.user = null;
    event.locals.session_id = sessionId || null;

    // Validate session if present
    if (sessionId) {
        const user = await validateSession(sessionId);
        
        if (user) {
            event.locals.user = user;
        } else {
            // Invalid session, clear cookie
            cookies.delete('session_id', { path: '/' });
            event.locals.session_id = null;
        }
    }

    // Route protection logic
    const pathname = url.pathname;

    // Skip protection for API routes, static files, and auth pages
    if (
        pathname.startsWith('/api') ||
        pathname.startsWith('/_app') ||
        pathname.startsWith('/favicon') ||
        pathname === '/login' ||
        pathname === '/admin/login' ||
        pathname === '/register' ||
        pathname === '/' ||
        pathname === '/unauthorized'
    ) {
        return resolve(event);
    }

    // Check if route requires authentication
    if (isProtectedRoute(pathname)) {
        if (!event.locals.user) {
            // Redirect to login with return URL
            const returnUrl = encodeURIComponent(url.pathname + url.search);
            throw redirect(307, `/login?redirect=${returnUrl}`);
        }

        // Check admin requirements
        if (requiresAdmin(pathname) && !isAdmin(event.locals.user)) {
            throw redirect(307, '/unauthorized');
        }

        // Check faculty admin requirements
        if (requiresFacultyAdmin(pathname) && !isFacultyAdmin(event.locals.user)) {
            throw redirect(307, '/unauthorized');
        }

        // Check super admin requirements
        if (requiresSuperAdmin(pathname) && !isSuperAdmin(event.locals.user)) {
            throw redirect(307, '/unauthorized');
        }

        // Additional permission checks can be added here based on specific routes
        // Example: Check specific permissions for certain routes
        if (pathname.startsWith('/admin/sessions') && !hasPermission(event.locals.user, 'ViewAllSessions')) {
            throw redirect(307, '/unauthorized');
        }
    }

    // Resolve the request
    const response = await resolve(event);

    // Add security headers
    response.headers.set('X-Frame-Options', 'DENY');
    response.headers.set('X-Content-Type-Options', 'nosniff');
    response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
    response.headers.set(
        'Content-Security-Policy',
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self' ws: wss:;"
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