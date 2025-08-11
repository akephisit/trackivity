import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_BASE_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ params, fetch }) => {
    const facultyId = params.id;
    
    if (!facultyId) {
        throw error(400, 'Invalid faculty ID');
    }
    
    try {
        // Make request to backend API without authentication (public endpoint)
        const response = await fetch(`${API_BASE_URL}/api/faculties/${facultyId}/departments/public`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        
        if (!response.ok) {
            console.error('Backend API error:', {
                url: `${API_BASE_URL}/api/faculties/${facultyId}/departments/public`,
                status: response.status,
                statusText: response.statusText
            });
            
            if (response.status === 404) {
                return json({
                    success: false,
                    data: [],
                    message: 'Faculty not found or has no departments'
                });
            }
            
            return json({
                success: false,
                data: [],
                message: 'Failed to fetch departments from backend'
            });
        }
        
        const data = await response.json();
        return json(data);
        
    } catch (error) {
        console.error('Error fetching departments:', error);
        
        return json({
            success: false,
            data: [],
            message: 'Network error while fetching departments'
        });
    }
};