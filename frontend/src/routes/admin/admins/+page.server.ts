import { requireSuperAdmin } from '$lib/server/auth';
import { fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { adminCreateSchema } from '$lib/schemas/auth';
import type { PageServerLoad, Actions } from './$types';
import type { AdminRole, Faculty } from '$lib/types/admin';

const API_BASE_URL = process.env.PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async (event) => {
	const user = await requireSuperAdmin(event);
	const sessionId = event.cookies.get('session_id');

	// โหลดรายการแอดมิน
	let admins: AdminRole[] = [];
	try {
		const response = await fetch(`${API_BASE_URL}/api/admin/users`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		if (response.ok) {
			const result = await response.json();
			if (result.status === 'success' && result.data && Array.isArray(result.data)) {
				admins = result.data;
			}
		}
	} catch (error) {
		console.error('Failed to load admins:', error);
	}

	// โหลดรายการคณะ
	let faculties: Faculty[] = [];
	try {
		const response = await fetch(`${API_BASE_URL}/api/faculties`);
		if (response.ok) {
			const result = await response.json();
			faculties = result.data?.faculties || result.data || [];
		}
	} catch (error) {
		console.error('Failed to load faculties:', error);
	}

	const form = await superValidate(zod(adminCreateSchema));

	return {
		user,
		admins,
		faculties,
		form
	};
};

export const actions: Actions = {
	create: async ({ request, cookies }) => {
		const form = await superValidate(request, zod(adminCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const sessionId = cookies.get('session_id');
			
			// Transform form data to match backend expectations
			const transformedData = {
				student_id: `A${Date.now()}`, // Generate temporary student_id for admin with prefix
				email: form.data.email,
				password: 'TempPass123!', // Temporary password - should be changed on first login
				first_name: form.data.name.split(' ')[0] || form.data.name,
				last_name: form.data.name.split(' ').slice(1).join(' ') || 'Admin',
				department_id: null,
				admin_level: form.data.admin_level,
				faculty_id: form.data.faculty_id && form.data.faculty_id !== '' ? form.data.faculty_id : null,
				permissions: form.data.permissions || []
			};
			
			const response = await fetch(`${API_BASE_URL}/api/admin/create`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify(transformedData)
			});

			const result = await response.json();

			if (!response.ok) {
				form.errors._errors = [result.message || 'เกิดข้อผิดพลาดในการสร้างแอดมิน'];
				return fail(400, { form });
			}

			if (result.status === 'success') {
				return { form, success: true, message: 'สร้างแอดมินสำเร็จ' };
			} else {
				form.errors._errors = [result.message || 'เกิดข้อผิดพลาดในการสร้างแอดมิน'];
				return fail(400, { form });
			}
		} catch (error) {
			console.error('Create admin error:', error);
			
			// ตรวจสอบประเภทของ error เพื่อให้ข้อความที่เหมาะสม
			if (error instanceof TypeError && error.message.includes('fetch')) {
				form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อกับเซิร์ฟเวอร์ กรุณาตรวจสอบว่า Backend Server กำลังทำงานอยู่'];
			} else if (error instanceof Error) {
				form.errors._errors = [`เกิดข้อผิดพลาด: ${error.message}`];
			} else {
				form.errors._errors = ['เกิดข้อผิดพลาดไม่ทราบสาเหตุในการสร้างแอดมิน'];
			}
			return fail(500, { form });
		}
	},

	delete: async ({ request, cookies }) => {
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string;

		if (!adminId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		try {
			const sessionId = cookies.get('session_id');
			const response = await fetch(`${API_BASE_URL}/api/users/${adminId}`, {
				method: 'DELETE',
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(400, { error: result.message || 'เกิดข้อผิดพลาดในการลบแอดมิน' });
			}

			return { success: true, message: 'ลบแอดมินสำเร็จ' };
		} catch (error) {
			console.error('Delete admin error:', error);
			return fail(500, { error: 'เกิดข้อผิดพลาดในการเชื่อมต่อ' });
		}
	}
};