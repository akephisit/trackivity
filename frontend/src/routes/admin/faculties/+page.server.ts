import { redirect, fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';
import { PUBLIC_API_URL } from '$env/static/public';

// Faculty schemas
const facultyCreateSchema = z.object({
	name: z.string().min(1, 'กรุณากรอกชื่อคณะ'),
	code: z.string().min(1, 'กรุณากรอกรหัสคณะ').max(10, 'รหัสคณะต้องไม่เกิน 10 ตัวอักษร'),
	description: z.string().optional(),
	status: z.boolean().default(true)
});

const facultyUpdateSchema = z.object({
	name: z.string().min(1, 'กรุณากรอกชื่อคณะ').optional(),
	code: z.string().min(1, 'กรุณากรอกรหัสคณะ').max(10, 'รหัสคณะต้องไม่เกิน 10 ตัวอักษร').optional(),
	description: z.string().optional(),
	status: z.boolean().optional()
});

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async ({ cookies, depends, fetch }) => {
	depends('app:page-data');
	
	const sessionId = cookies.get('session_id');
	if (!sessionId) {
		throw redirect(302, '/admin/login');
	}

	try {
		// Fetch all faculties for admin (including inactive ones)
		const facultiesResponse = await fetch(`${API_BASE_URL}/api/admin/faculties`, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		let faculties = [];
		if (facultiesResponse.ok) {
			const facultiesData = await facultiesResponse.json();
			if (facultiesData.status === 'success') {
				faculties = facultiesData.data.faculties || [];
			}
		}

		// Create forms
		const createForm = await superValidate(zod(facultyCreateSchema));
		const updateForm = await superValidate(zod(facultyUpdateSchema));

		return {
			faculties,
			createForm,
			updateForm
		};
	} catch (error) {
		console.error('Failed to load faculties data:', error);
		return {
			faculties: [],
			createForm: await superValidate(zod(facultyCreateSchema)),
			updateForm: await superValidate(zod(facultyUpdateSchema))
		};
	}
};

export const actions: Actions = {
	create: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const form = await superValidate(request, zod(facultyCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const response = await fetch(`${API_BASE_URL}/api/faculties`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify(form.data)
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					form,
					error: result.message || 'เกิดข้อผิดพลาดในการสร้างคณะ'
				});
			}

			return { form, success: true };
		} catch (error) {
			console.error('Failed to create faculty:', error);
			return fail(500, { 
				form,
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	},

	update: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const facultyId = formData.get('facultyId') as string;
		const updateData = JSON.parse(formData.get('updateData') as string);

		const form = await superValidate(updateData, zod(facultyUpdateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const response = await fetch(`${API_BASE_URL}/api/faculties/${facultyId}`, {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify(form.data)
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					form,
					error: result.message || 'เกิดข้อผิดพลาดในการแก้ไขคณะ'
				});
			}

			return { form, success: true };
		} catch (error) {
			console.error('Failed to update faculty:', error);
			return fail(500, { 
				form,
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	},

	delete: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const facultyId = formData.get('facultyId') as string;

		try {
			const response = await fetch(`${API_BASE_URL}/api/faculties/${facultyId}`, {
				method: 'DELETE',
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการลบคณะ'
				});
			}

			return { success: true };
		} catch (error) {
			console.error('Failed to delete faculty:', error);
			return fail(500, { 
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	},

	toggleStatus: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const facultyId = formData.get('facultyId') as string;

		try {
			const response = await fetch(`${API_BASE_URL}/api/faculties/${facultyId}/toggle-status`, {
				method: 'PUT',
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการเปลี่ยนสถานะคณะ'
				});
			}

			return { success: true };
		} catch (error) {
			console.error('Failed to toggle faculty status:', error);
			return fail(500, { 
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	}
};