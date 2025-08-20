import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { adminLoginSchema } from '$lib/schemas/auth';
import { api } from '$lib/server/api-client';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	// Check if admin already logged in
	const sessionId = event.cookies.get('session_id');
	
	if (sessionId) {
		try {
        const response = await api.get(event, '/api/admin/auth/me');
        
        if (response.success) {
            throw redirect(303, '/admin');
        }
		} catch (error) {
			// If redirect, throw it
			if (error instanceof Response) {
				throw error;
			}
			
			// If other error, clear invalid session
			event.cookies.delete('session_id', { path: '/' });
		}
	}

	const form = await superValidate(zod(adminLoginSchema));
	
	return {
		form,
		isDevelopment: process.env.NODE_ENV === 'development'
	};
};

export const actions: Actions = {
	default: async (event) => {
		const form = await superValidate(event.request, zod(adminLoginSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			console.log('=== ADMIN LOGIN DEBUG START ===');
			console.log('Form data:', { 
				email: form.data.email,
				has_password: !!form.data.password,
				remember_me: form.data.remember_me 
			});
			
			const response = await api.post(event, '/api/admin/auth/login', {
				email: form.data.email,
				password: form.data.password,
				remember_me: form.data.remember_me
			});

			console.log('API Response:', {
				success: response.success,
				has_data: !!response.data,
				has_error: !!response.error,
				data_keys: response.data ? Object.keys(response.data) : null,
				response_sample: response.success ? 'SUCCESS' : response.error
			});

            if (!response.success) {
				console.log('Response failed:', response.error);
                form.errors._errors = [response.error || 'การเข้าสู่ระบบไม่สำเร็จ'];
                return fail(400, { form });
            }

			// Backend returns { success: true, session: {...} } directly in response.data
			if (response.success && response.data?.session) {
				console.log('Login successful, setting cookie');
				// Set session cookie
				event.cookies.set('session_id', response.data.session.session_id, {
					path: '/',
					httpOnly: true,
					secure: process.env.NODE_ENV === 'production',
					sameSite: 'lax',
					maxAge: form.data.remember_me ? 30 * 24 * 60 * 60 : 24 * 60 * 60 // 30 days or 1 day
				});

				throw redirect(303, '/admin');
			} else {
				console.log('Login failed - no session in response');
				form.errors._errors = [response.data?.message || response.error || 'การเข้าสู่ระบบไม่สำเร็จ'];
				return fail(400, { form });
			}
		} catch (error) {
			console.log('=== CAUGHT ERROR ===');
			console.log('Error type:', typeof error);
			console.log('Error details:', error);
			console.log('Is Response?', error instanceof Response);
			console.log('Has status?', error && typeof error === 'object' && 'status' in error);
			console.log('Has location?', error && typeof error === 'object' && 'location' in error);
			
			// Check if this is a SvelteKit redirect (success case)
			if (error && typeof error === 'object' && 'status' in error && 'location' in error) {
				console.log('Throwing SvelteKit redirect');
				throw error;
			}
			
			// If redirect, throw it (this is success case)
			if (error instanceof Response) {
				console.log('Throwing Response redirect');
				throw error;
			}
			
			// Only set actual error messages for real errors
			if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
				console.log('Setting error message from error object:', error.message);
				form.errors._errors = [error.message];
			} else {
				console.log('Setting generic connection error');
				form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อ'];
			}
			console.log('=== ERROR DEBUG END ===');
			return fail(400, { form });
		}
	}
};
