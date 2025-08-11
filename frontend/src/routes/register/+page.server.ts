import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { registerSchema } from '$lib/schemas/auth';
import type { Actions, PageServerLoad } from './$types';
import type { Faculty } from '$lib/types/admin';
import { PUBLIC_API_URL } from '$env/static/public';

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';


// ฟังก์ชันสำหรับตรวจสอบการเชื่อมต่อ backend
async function checkBackendConnection(): Promise<boolean> {
	try {
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), 3000); // timeout 3 วินาที
		
		const response = await fetch(`${API_BASE_URL}/health`, {
			signal: controller.signal,
			headers: {
				'Accept': 'application/json'
			}
		});
		
		clearTimeout(timeoutId);
		return response.ok;
	} catch (error) {
		return false;
	}
}

export const load: PageServerLoad = async ({ cookies }) => {
	// ตรวจสอบว่ามี session อยู่แล้วหรือไม่
	const sessionId = cookies.get('session_id');
	
	if (sessionId) {
		try {
			const controller = new AbortController();
			const timeoutId = setTimeout(() => controller.abort(), 5000);
			
			const response = await fetch(`${API_BASE_URL}/api/auth/me`, {
				signal: controller.signal,
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});
			
			clearTimeout(timeoutId);
			
			if (response.ok) {
				throw redirect(303, '/admin');
			}
		} catch (error) {
			// ถ้า error เป็น redirect ให้ throw ต่อไป
			if (error instanceof Response) {
				throw error;
			}
			
			// ถ้าเป็น connection error ให้ลบ session
			console.warn('Backend not available for session check:', error);
			cookies.delete('session_id', { path: '/' });
		}
	}

	// โหลดรายการคณะจากฐานข้อมูล
	let faculties: Faculty[] = [];

	try {
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), 5000);
		
		const response = await fetch(`${API_BASE_URL}/api/faculties`, {
			signal: controller.signal,
			headers: {
				'Accept': 'application/json'
			}
		});
		
		clearTimeout(timeoutId);
		
		if (response.ok) {
			const result = await response.json();
			faculties = result.data?.faculties || [];
		} else {
			throw new Error(`HTTP ${response.status}: ${response.statusText}`);
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
	default: async ({ request, cookies }) => {
		const form = await superValidate(request, zod(registerSchema));

		if (!form.valid) {
			return fail(400, { form });
		}


		try {
			// ส่งข้อมูลการสมัครไป backend
			const controller = new AbortController();
			const timeoutId = setTimeout(() => controller.abort(), 10000); // timeout 10 วินาที
			
			// Use department_id directly (validation handled by client)
			const departmentId = form.data.department_id;
			
			const requestBody = {
				student_id: form.data.student_id,
				email: form.data.email,
				password: form.data.password,
				first_name: form.data.first_name,
				last_name: form.data.last_name,
				department_id: departmentId
			};
			
			
			const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
				method: 'POST',
				signal: controller.signal,
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify(requestBody)
			});

			clearTimeout(timeoutId);
			
			const result = await response.json();

			if (!response.ok) {
				// การสมัครล้มเหลว
				if (result.message?.includes('student_id')) {
					form.errors.student_id = [result.message || 'รหัสนักศึกษานี้ถูกใช้งานแล้ว'];
				} else if (result.message?.includes('email')) {
					form.errors.email = [result.message || 'อีเมลนี้ถูกใช้งานแล้ว'];
				} else {
					form.errors._errors = [result.message || 'เกิดข้อผิดพลาดในการสมัครสมาชิก'];
				}
				return fail(400, { form });
			}

			if (result.success) {
				// สมัครสำเร็จ - redirect ไป login พร้อมข้อความแจ้ง
				clearTimeout(timeoutId);
				throw redirect(303, '/login?registered=true');
			} else {
				form.errors._errors = [result.message || 'เกิดข้อผิดพลาดในการสมัครสมาชิก'];
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
			
			// จัดการ error แต่ละประเภท
			if (error instanceof Error) {
				if (error.name === 'AbortError') {
					form.errors._errors = ['การเชื่อมต่อใช้เวลานานเกินไป กรุณาลองใหม่อีกครั้ง'];
				} else if (error.message.includes('ECONNREFUSED')) {
					form.errors._errors = ['ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้ กรุณาตรวจสอบการเชื่อมต่ออินเทอร์เน็ต'];
				} else if (error.message.includes('fetch')) {
					form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อ กรุณาลองใหม่อีกครั้ง'];
				} else {
					form.errors._errors = ['เกิดข้อผิดพลาดที่ไม่คาดคิด กรุณาลองใหม่อีกครั้ง'];
				}
			} else {
				form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อ กรุณาลองใหม่อีกครั้ง'];
			}
			
			return fail(503, { form });
		}
	}
};