import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { registerSchema } from '$lib/schemas/auth';
import type { Actions, PageServerLoad } from './$types';
import type { Faculty } from '$lib/types/admin';
import { api } from '$lib/server/api-client';



export const load: PageServerLoad = async (event) => {
	// ตรวจสอบว่ามี session อยู่แล้วหรือไม่
	const sessionId = event.cookies.get('session_id');
	
    if (sessionId) {
		try {
            const response = await api.get(event, '/api/auth/me');
            
            if (response.success) {
                throw redirect(303, '/admin');
            }
		} catch (error) {
			// ถ้า error เป็น redirect ให้ throw ต่อไป
			if (error instanceof Response) {
				throw error;
			}
			
			// ถ้าเป็น connection error ให้ลบ session
			console.warn('Backend not available for session check:', error);
			event.cookies.delete('session_id', { path: '/' });
		}
	}

	// โหลดรายการคณะจากฐานข้อมูล
	let faculties: Faculty[] = [];

    try {
        const response = await api.get(event, '/api/faculties');
        
        if (response.success) {
            faculties = response.data?.faculties || [];
        } else {
            throw new Error(response.error || 'Failed to load faculties');
        }
	} catch (error) {
		console.error('Failed to load faculties from backend:', error);
		// ไม่มี fallback data - ต้องเชื่อมต่อกับเซิร์ฟเวอร์ได้
		throw new Error('ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้ กรุณาลองใหม่อีกครั้ง');
	}

	const form = await superValidate(zod(registerSchema));
	
	return {
		form,
		faculties
	};
};

export const actions: Actions = {
	default: async (event) => {
		const { request } = event;
		const form = await superValidate(request, zod(registerSchema));

		if (!form.valid) {
			return fail(400, { form });
		}


		try {
			// ส่งข้อมูลการสมัครไป backend
			const requestBody = {
				student_id: form.data.student_id,
				email: form.data.email,
				password: form.data.password,
				prefix: form.data.prefix,
				first_name: form.data.first_name,
				last_name: form.data.last_name,
				department_id: form.data.department_id
			};
			
        const response = await api.post(event, '/api/auth/register', requestBody);

        if (!response.success) {
            // การสมัครล้มเหลว
            const errorMessage = response.error || 'เกิดข้อผิดพลาดในการสมัครสมาชิก';
            if (errorMessage.includes('student_id')) {
                form.errors.student_id = [errorMessage];
            } else if (errorMessage.includes('email')) {
                form.errors.email = [errorMessage];
            } else {
                form.errors._errors = [errorMessage];
            }
            return fail(400, { form });
        }

        if (response.success) {
            // สมัครสำเร็จ - redirect ไป login พร้อมข้อความแจ้ง
            throw redirect(303, '/login?registered=true');
        } else {
            form.errors._errors = ['เกิดข้อผิดพลาดในการสมัครสมาชิก'];
            return fail(400, { form });
        }
		} catch (error) {
			// ตรวจสอบว่าเป็น redirect object หรือไม่
			if (error instanceof Response || 
				(error && typeof error === 'object' && (error as any).status === 303) ||
				(error && typeof error === 'object' && (error as any).location) ||
				(error && error.toString && error.toString().includes('Redirect'))) {
				throw error;
			}
			
			console.error('Registration error:', error);
			form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อ กรุณาลองใหม่อีกครั้ง'];
			return fail(503, { form });
		}
	}
};
