import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { adminLoginSchema } from '$lib/schemas/auth';
import { PUBLIC_API_URL } from '$env/static/public';
import type { Actions, PageServerLoad } from './$types';

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async ({ cookies }) => {
	// Check if admin already logged in
	const sessionId = cookies.get('session_id');
	
	if (sessionId) {
		try {
			const response = await fetch(`${API_BASE_URL}/api/admin/auth/me`, {
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});
			
			if (response.ok) {
				throw redirect(303, '/admin');
			}
		} catch (error) {
			// If redirect, throw it
			if (error instanceof Response) {
				throw error;
			}
			
			// If other error, clear invalid session
			cookies.delete('session_id', { path: '/' });
		}
	}

	const form = await superValidate(zod(adminLoginSchema));
	
	return {
		form,
		isDevelopment: process.env.NODE_ENV === 'development'
	};
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await superValidate(request, zod(adminLoginSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const response = await fetch(`${API_BASE_URL}/api/admin/auth/login`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				credentials: 'include',
				body: JSON.stringify({
					email: form.data.email,
					password: form.data.password,
					remember_me: form.data.remember_me
				})
			});

			const result = await response.json();

			if (!response.ok) {
				form.errors._errors = [result.message || 'การเข้าสู่ระบบไม่สำเร็จ'];
				return fail(400, { form });
			}

			if (result.success && result.session) {
				// Set session cookie
				cookies.set('session_id', result.session.session_id, {
					path: '/',
					httpOnly: true,
					secure: process.env.NODE_ENV === 'production',
					sameSite: 'lax',
					maxAge: form.data.remember_me ? 30 * 24 * 60 * 60 : 24 * 60 * 60 // 30 days or 1 day
				});

				throw redirect(303, '/admin');
			} else {
				form.errors._errors = [result.message || 'การเข้าสู่ระบบไม่สำเร็จ'];
				return fail(400, { form });
			}
		} catch (error) {
			// Check if this is a SvelteKit redirect (success case)
			if (error && typeof error === 'object' && 'status' in error && 'location' in error) {
				throw error;
			}
			
			// If redirect, throw it (this is success case)
			if (error instanceof Response) {
				throw error;
			}
			
			// Only set actual error messages for real errors
			if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
				form.errors._errors = [error.message];
			} else {
				form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อ'];
			}
			return fail(400, { form });
		}
	}
};