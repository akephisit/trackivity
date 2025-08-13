import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { loginSchema } from '$lib/schemas/auth';
import type { Actions, PageServerLoad } from './$types';
 

export const load: PageServerLoad = async ({ cookies, url, fetch }) => {
	// ตรวจสอบว่ามี session อยู่แล้วหรือไม่
	const sessionId = cookies.get('session_id');
	if (sessionId) {
		// ตรวจสอบ session กับ backend
		try {
			const response = await fetch(`/api/auth/me`);
			
			if (response.ok) {
				const payload = await response.json();
				const hasUser = Boolean((payload as any)?.user ?? (payload as any)?.data);
				if (hasUser) {
					// มี session ที่ใช้งานได้ ให้ redirect ไปหน้าเป้าหมาย
					const redirectTo = url.searchParams.get('redirectTo') || '/';
					throw redirect(303, redirectTo);
				}
			}
		} catch (error) {
			// ถ้าเกิดข้อผิดพลาด ให้ล้าง session
			cookies.delete('session_id', { path: '/' });
		}
	}

	// สร้าง form สำหรับ login
	const form = await superValidate(zod(loginSchema));
	
	return {
		form
	};
};

export const actions: Actions = {
	default: async ({ request, cookies, url, fetch }) => {
		const form = await superValidate(request, zod(loginSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			// ส่งข้อมูล login ไป backend
			const response = await fetch(`/api/auth/login`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					student_id: form.data.student_id,
					password: form.data.password,
					remember_me: form.data.remember_me
				})
			});

			const result = await response.json();

			if (!response.ok) {
				// Login ล้มเหลว
				form.errors.student_id = [result.message || 'อีเมลหรือรหัسผ่านไม่ถูกต้อง'];
				return fail(400, { form });
			}

			if (result.success && result.session) {
				// Login สำเร็จ - เก็บ session
				cookies.set('session_id', result.session.session_id, {
					path: '/',
					httpOnly: true,
					secure: process.env.NODE_ENV === 'production',
					sameSite: 'lax',
					maxAge: form.data.remember_me ? 30 * 24 * 60 * 60 : 24 * 60 * 60 // 30 days or 1 day
				});


				// Redirect ไปยังหน้าที่ต้องการ
				const redirectTo = url.searchParams.get('redirectTo') || '/';
				throw redirect(303, redirectTo);
			} else {
				form.errors.student_id = [result.message || 'เกิดข้อผิดพลาดในการเข้าสู่ระบบ'];
				return fail(400, { form });
			}
		} catch (error) {
			// ถ้าเป็น redirect error ให้ผ่านต่อไป
			if (error && typeof error === 'object' && 'status' in error && 'location' in error) {
				throw error;
			}
			console.error('Login error:', error);
			form.errors.student_id = ['เกิดข้อผิดพลาดในการเชื่อมต่อ กรุณาลองใหม่อีกครั้ง'];
			return fail(500, { form });
		}
	}
};
