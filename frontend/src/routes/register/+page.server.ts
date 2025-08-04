import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { registerSchema } from '$lib/schemas/auth';
import type { Actions, PageServerLoad } from './$types';
import type { Faculty } from '$lib/types/admin';

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:8000';

// Fallback faculties data สำหรับกรณีที่ backend ไม่สามารถเชื่อมต่อได้
const FALLBACK_FACULTIES: Faculty[] = [
	{
		id: 1,
		name: 'คณะวิทยาศาสตร์',
		code: 'SCI',
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString()
	},
	{
		id: 2,
		name: 'คณะวิศวกรรมศาสตร์',
		code: 'ENG',
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString()
	},
	{
		id: 3,
		name: 'คณะครุศาสตร์',
		code: 'EDU',
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString()
	},
	{
		id: 4,
		name: 'คณะมนุษยศาสตร์และสังคมศาสตร์',
		code: 'HUM',
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString()
	},
	{
		id: 5,
		name: 'คณะบริหารธุรกิจ',
		code: 'BUS',
		created_at: new Date().toISOString(),
		updated_at: new Date().toISOString()
	}
];

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
	let backendAvailable = true;
	
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
			
			// ถ้าเป็น connection error ให้ลบ session และทำเครื่องหมายว่า backend ไม่พร้อมใช้งาน
			console.warn('Backend not available for session check:', error);
			cookies.delete('session_id', { path: '/' });
			backendAvailable = false;
		}
	}

	// โหลดรายการคณะ
	let faculties: Faculty[] = [];
	let facultiesFromBackend = false;
	let backendErrorMessage: string | null = null;

	try {
		const controller = new AbortController();
		const timeoutId = setTimeout(() => controller.abort(), 5000); // timeout 5 วินาที
		
		const response = await fetch(`${API_BASE_URL}/api/faculties`, {
			signal: controller.signal,
			headers: {
				'Accept': 'application/json'
			}
		});
		
		clearTimeout(timeoutId);
		
		if (response.ok) {
			const result = await response.json();
			faculties = result.data || [];
			facultiesFromBackend = true;
			backendAvailable = true;
		} else {
			throw new Error(`HTTP ${response.status}: ${response.statusText}`);
		}
	} catch (error) {
		console.warn('Failed to load faculties from backend:', error);
		
		// ใช้ fallback data
		faculties = FALLBACK_FACULTIES;
		backendAvailable = false;
		
		// กำหนดข้อความ error ตามประเภทของ error
		if (error instanceof Error) {
			if (error.name === 'AbortError') {
				backendErrorMessage = 'การเชื่อมต่อกับเซิร์ฟเวอร์ใช้เวลานานเกินไป';
			} else if (error.message.includes('ECONNREFUSED')) {
				backendErrorMessage = 'ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้';
			} else {
				backendErrorMessage = 'เกิดข้อผิดพลาดในการโหลดข้อมูลจากเซิร์ฟเวอร์';
			}
		} else {
			backendErrorMessage = 'เกิดข้อผิดพลาดในการเชื่อมต่อ';
		}
	}

	const form = await superValidate(zod(registerSchema));
	
	return {
		form,
		faculties,
		backendAvailable,
		facultiesFromBackend,
		backendErrorMessage
	};
};

export const actions: Actions = {
	default: async ({ request, cookies }) => {
		const form = await superValidate(request, zod(registerSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		// ตรวจสอบการเชื่อมต่อ backend ก่อนส่งข้อมูล
		const backendConnected = await checkBackendConnection();
		
		if (!backendConnected) {
			form.errors._errors = [
				'ไม่สามารถเชื่อมต่อกับเซิร์ฟเวอร์ได้ในขณะนี้ กรุณาตรวจสอบการเชื่อมต่ออินเทอร์เน็ตและลองใหม่อีกครั้ง หรือติดต่อผู้ดูแลระบบ'
			];
			return fail(503, { 
				form,
				backendError: true,
				backendErrorMessage: 'เซิร์ฟเวอร์ไม่พร้อมใช้งาน'
			});
		}

		try {
			// ส่งข้อมูลการสมัครไป backend
			const controller = new AbortController();
			const timeoutId = setTimeout(() => controller.abort(), 10000); // timeout 10 วินาที
			
			const response = await fetch(`${API_BASE_URL}/api/auth/register`, {
				method: 'POST',
				signal: controller.signal,
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					email: form.data.email,
					password: form.data.password,
					name: form.data.name,
					admin_level: form.data.admin_level,
					faculty_id: form.data.faculty_id
				})
			});

			clearTimeout(timeoutId);
			
			const result = await response.json();

			if (!response.ok) {
				// การสมัครล้มเหลว
				if (result.message?.includes('email')) {
					form.errors.email = [result.message || 'อีเมลนี้ถูกใช้งานแล้ว'];
				} else {
					form.errors._errors = [result.message || 'เกิดข้อผิดพลาดในการสมัครสมาชิก'];
				}
				return fail(400, { form });
			}

			if (result.success) {
				// สมัครสำเร็จ - redirect ไป login พร้อมข้อความแจ้ง
				throw redirect(303, '/login?registered=true');
			} else {
				form.errors._errors = [result.message || 'เกิดข้อผิดพลาดในการสมัครสมาชิก'];
				return fail(400, { form });
			}
		} catch (error) {
			console.error('Register error:', error);
			
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
			
			return fail(503, { 
				form,
				backendError: true,
				backendErrorMessage: 'เกิดข้อผิดพลาดในการเชื่อมต่อกับเซิร์ฟเวอร์'
			});
		}
	}
};