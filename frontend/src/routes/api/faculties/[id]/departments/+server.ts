import { requireAdmin } from '$lib/server/auth';
import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { AdminLevel } from '$lib/types/admin';

export const GET: RequestHandler = async (event) => {
    try {
        // Require authentication
        const user = await requireAdmin(event);
        const { params } = event;
        const facultyId = params.id;
        
        // Authorization check - SuperAdmin can access any faculty, FacultyAdmin only their own
        if (user.admin_role?.admin_level === AdminLevel.FacultyAdmin && 
            user.admin_role?.faculty_id !== facultyId) {
            throw error(403, 'Access denied: You can only view departments from your own faculty');
        }
        
        // Make request to backend API
        const backendUrl = `${process.env.BACKEND_URL}/api/admin/faculties/${facultyId}/departments`;
        const response = await fetch(backendUrl, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${event.cookies.get('auth_token')}`,
                'Content-Type': 'application/json'
            }
        });
        
        if (!response.ok) {
            const errorText = await response.text();
            console.error('Backend API error:', response.status, errorText);
            
            if (response.status === 404) {
                throw error(404, 'Faculty not found or has no departments');
            } else if (response.status === 403) {
                throw error(403, 'Access denied');
            } else {
                throw error(response.status, `Failed to fetch departments: ${errorText}`);
            }
        }
        
        const data = await response.json();
        return json(data);
        
    } catch (err: any) {
        console.error('Error in faculty departments API:', err);
        
        if (err.status) {
            throw err; // Re-throw SvelteKit errors
        }
        
        throw error(500, 'Failed to fetch departments');
    }
};