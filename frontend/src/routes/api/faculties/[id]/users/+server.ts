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
        
        // Get query parameters
        const url = event.url;
        const searchParams = url.searchParams;
        
        const page = parseInt(searchParams.get('page') || '1');
        const limit = parseInt(searchParams.get('limit') || '10');
        const search = searchParams.get('search') || '';
        const status = searchParams.get('status') || 'all';
        const department_id = searchParams.get('department_id') || '';
        
        // Authorization check - SuperAdmin can access any faculty, FacultyAdmin only their own
        if (user.admin_role?.admin_level === AdminLevel.FacultyAdmin && 
            user.admin_role?.faculty_id !== facultyId) {
            throw error(403, 'Access denied: You can only view users from your own faculty');
        }
        
        // Build query parameters for backend API
        const queryParams = new URLSearchParams({
            page: page.toString(),
            limit: limit.toString(),
            ...(search && { search }),
            ...(status !== 'all' && { status }),
            ...(department_id && { department_id }),
            faculty_id: facultyId
        });
        
        // Validate backend URL configuration
        if (!PUBLIC_API_URL) {
            console.error('PUBLIC_API_URL environment variable is not configured');
            throw error(500, 'API configuration error: Backend URL not configured');
        }
        
        // Make request to backend API
        const backendUrl = `${PUBLIC_API_URL}/api/admin/users?${queryParams.toString()}`;
        const response = await event.fetch(backendUrl, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${event.cookies.get('auth_token')}`,
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
                throw error(404, 'Faculty not found or has no users');
            } else if (response.status === 403) {
                throw error(403, 'Access denied');
            } else if (response.status === 401) {
                throw error(401, 'Authentication required');
            } else if (response.status >= 500) {
                throw error(502, 'Backend service unavailable');
            } else {
                throw error(response.status, `Failed to fetch users: ${errorText}`);
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
        console.error('Error in faculty users API:', {
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
        throw error(500, 'Failed to fetch users');
    }
};