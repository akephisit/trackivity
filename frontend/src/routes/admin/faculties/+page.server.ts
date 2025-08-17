import { redirect, fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';
import { api } from '$lib/server/api-client';

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

export const load: PageServerLoad = async (event) => {
	const { cookies, depends } = event;
	depends('app:page-data');
	
	const sessionId = cookies.get('session_id');
	if (!sessionId) {
		throw redirect(302, '/admin/login');
	}

	try {
		// Fetch all faculties for admin (including inactive ones)
		const facultiesResponse = await api.get(event, '/api/admin/faculties');

        let faculties = [];
        if (facultiesResponse.success) {
            faculties = facultiesResponse.data.faculties || [];
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
	create: async (event) => {
		const { request, cookies } = event;
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const form = await superValidate(request, zod(facultyCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const response = await api.post(event, '/api/faculties', form.data);

            if (!response.success) {
                return fail(400, { 
                    form,
                    error: response.error || 'เกิดข้อผิดพลาดในการสร้างคณะ'
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

	update: async (event) => {
		const { request, cookies } = event;
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
			const response = await api.put(event, `/api/faculties/${facultyId}`, form.data);

            if (!response.success) {
                return fail(400, { 
                    form,
                    error: response.error || 'เกิดข้อผิดพลาดในการแก้ไขคณะ'
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

	delete: async (event) => {
		const { request, cookies } = event;
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const facultyId = formData.get('facultyId') as string;

		try {
			const response = await api.delete(event, `/api/faculties/${facultyId}`);

            if (!response.success) {
                return fail(400, { 
                    error: response.error || 'เกิดข้อผิดพลาดในการลบคณะ'
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

	toggleStatus: async (event) => {
		const { request, cookies } = event;
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const facultyId = formData.get('facultyId') as string;

		try {
			const response = await api.put(event, `/api/faculties/${facultyId}/toggle-status`);

            if (!response.success) {
                return fail(400, { 
                    error: response.error || 'เกิดข้อผิดพลาดในการเปลี่ยนสถานะคณะ'
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
