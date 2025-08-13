import { json, redirect } from '@sveltejs/kit';
import { logout } from '$lib/server/auth';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async (event) => {
    try {
        await logout(event);
    } catch (error) {
        console.error('Logout error (GET):', error);
        event.cookies.delete('session_id', { path: '/' });
    }
    throw redirect(303, '/?logout=1');
};

export const POST: RequestHandler = async (event) => {
    try {
        await logout(event);
    } catch (error) {
        console.error('Logout error (POST):', error);
        event.cookies.delete('session_id', { path: '/' });
    }
    // For programmatic fetch-based logout, return JSON success
    return json({ success: true });
};
