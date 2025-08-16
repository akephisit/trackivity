import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const API_BASE_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const GET: RequestHandler = async ({ fetch }) => {
    try {
        // Make request to backend API without authentication (public endpoint)
        const response = await fetch(`${API_BASE_URL}/api/faculties`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            }
        });
        
        if (!response.ok) {
            console.error('Backend API error:', {
                status: response.status,
                statusText: response.statusText
            });
            
            // Return empty array instead of throwing error
            return json({
                success: false,
                data: [],
                message: 'Failed to fetch faculties from backend'
            });
        }
        
        const data = await response.json();
        return json(data);
        
    } catch (error) {
        console.error('Error fetching faculties:', error);
        
        // Return fallback empty response
        return json({
            success: false,
            data: [],
            message: 'Network error while fetching faculties'
        });
    }
};

export const POST: RequestHandler = async ({ request, cookies, fetch }) => {
    try {
        const sessionId = cookies.get('session_id');
        
        if (!sessionId) {
            return json({
                success: false,
                message: 'Authentication required'
            }, { status: 401 });
        }

        const body = await request.json();
        
        const response = await fetch(`${API_BASE_URL}/api/faculties`, {
            method: 'POST',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json',
                'Cookie': `session_id=${sessionId}`
            },
            body: JSON.stringify(body)
        });
        
        if (!response.ok) {
            const errorData = await response.json().catch(() => ({ message: 'Unknown error' }));
            return json({
                success: false,
                message: errorData.message || 'Failed to create faculty'
            }, { status: response.status });
        }
        
        const data = await response.json();
        return json(data);
        
    } catch (error) {
        console.error('Error creating faculty:', error);
        return json({
            success: false,
            message: 'Network error while creating faculty'
        }, { status: 500 });
    }
};