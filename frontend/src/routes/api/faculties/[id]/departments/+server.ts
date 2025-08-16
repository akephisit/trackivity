import { requireAdmin } from '$lib/server/auth';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { AdminLevel } from '$lib/types/admin';
import { PUBLIC_API_URL } from '$env/static/public';

export const GET: RequestHandler = async (event) => {
    // Extract facultyId outside try block to make it available in catch block
    const { params } = event;
    const facultyId = params.id;
    
    try {
        // Require authentication
        const user = await requireAdmin(event);
        
        // Authorization check - SuperAdmin can access any faculty, FacultyAdmin only their own
        if (user.admin_role?.admin_level === AdminLevel.FacultyAdmin && 
            user.admin_role?.faculty_id !== facultyId) {
            throw error(403, 'Access denied: You can only view departments from your own faculty');
        }
        
        // Validate backend URL configuration
        if (!PUBLIC_API_URL) {
            console.error('PUBLIC_API_URL environment variable is not configured');
            throw error(500, 'API configuration error: Backend URL not configured');
        }
        
        // Make request to backend API
        const backendUrl = `${PUBLIC_API_URL}/api/faculties/${facultyId}/departments`;
        const response = await event.fetch(backendUrl, {
            method: 'GET',
            headers: {
                'Cookie': `session_id=${event.cookies.get('session_id')}`,
                'Content-Type': 'application/json'
            }
        });
        
        if (!response.ok) {
            let errorText: string;
            try {
                errorText = await response.text();
            } catch (textError) {
                errorText = 'Failed to read error response';
            }
            
            console.error('Backend API error:', {
                url: backendUrl,
                status: response.status,
                statusText: response.statusText,
                error: errorText
            });
            
            if (response.status === 404) {
                throw error(404, 'Faculty not found or has no departments');
            } else if (response.status === 403) {
                throw error(403, 'Access denied');
            } else if (response.status === 401) {
                throw error(401, 'Authentication required');
            } else if (response.status >= 500) {
                throw error(502, 'Backend service unavailable');
            } else {
                throw error(response.status, `Failed to fetch departments: ${errorText}`);
            }
        }
        
        let data;
        try {
            data = await response.json();
        } catch (parseError) {
            console.error('Failed to parse response JSON:', parseError);
            throw error(502, 'Invalid response from backend service');
        }
        
        return json(data);
        
    } catch (err: any) {
        console.error('Error in faculty departments API:', {
            error: err,
            facultyId,
            url: event.url.pathname
        });
        
        // Re-throw SvelteKit errors (these have a status property)
        if (err.status) {
            throw err;
        }
        
        // Handle network or other fetch errors
        if (err.name === 'TypeError' && err.message.includes('fetch')) {
            throw error(503, 'Unable to connect to backend service');
        }
        
        // Generic fallback error
        throw error(500, 'Failed to fetch departments');
    }
};

export const POST: RequestHandler = async (event) => {
    const { params } = event;
    const facultyId = params.id;

    try {
        const user = await requireAdmin(event);

        // FacultyAdmin can only create in their own faculty
        if (user.admin_role?.admin_level === AdminLevel.FacultyAdmin && user.admin_role?.faculty_id !== facultyId) {
            throw error(403, 'Access denied: You can only create departments in your faculty');
        }

        if (!PUBLIC_API_URL) {
            console.error('PUBLIC_API_URL is not configured');
            throw error(500, 'API configuration error');
        }

        const backendUrl = `${PUBLIC_API_URL}/api/faculties/${facultyId}/departments`;

        // Pass-through body as-is (validated upstream via superforms)
        const body = await event.request.text();

        const response = await event.fetch(backendUrl, {
            method: 'POST',
            headers: {
                'Cookie': `session_id=${event.cookies.get('session_id')}`,
                'Content-Type': 'application/json'
            },
            body
        });

        const ct = response.headers.get('content-type') || '';
        if (!response.ok) {
            if (ct.includes('application/json')) {
                const data = await response.json();
                return json(data, { status: response.status });
            } else {
                const text = await response.text().catch(() => '');
                return new Response(text || 'Department creation failed', { status: response.status });
            }
        }

        if (ct.includes('application/json')) {
            const data = await response.json();
            return json(data, { status: response.status });
        }

        // No/invalid JSON: return empty success
        return json({ success: true }, { status: response.status });
    } catch (err: any) {
        if (err.status) throw err;
        if (err.name === 'TypeError' && err.message.includes('fetch')) {
            throw error(503, 'Unable to connect to backend');
        }
        throw error(500, 'Failed to create department');
    }
};
