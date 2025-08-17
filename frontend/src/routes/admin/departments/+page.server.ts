import { redirect, fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';
import type { Department, Faculty } from '$lib/types/admin';
import { requireAdmin } from '$lib/server/auth';
import { api } from '$lib/server/api-client';

// Department schemas
const departmentCreateSchema = z.object({
	name: z.string().min(1, 'กรุณากรอกชื่อภาควิชา'),
	code: z.string().min(1, 'กรุณากรอกรหัสภาควิชา'),
	description: z.string().optional(),
	head_name: z.string().optional(),
	head_email: z.string().email('รูปแบบอีเมลไม่ถูกต้อง').optional().or(z.literal('')),
	status: z.boolean().default(true),
	// สำหรับ SuperAdmin ต้องเลือกคณะ
	faculty_id: z.string().uuid('รหัสคณะไม่ถูกต้อง').optional()
});

const departmentUpdateSchema = z.object({
	name: z.string().min(1, 'กรุณากรอกชื่อภาควิชา').optional(),
	code: z.string().min(1, 'กรุณากรอกรหัสภาควิชา').optional(),
	description: z.string().optional(),
	head_name: z.string().optional(),
	head_email: z.string().email('รูปแบบอีเมลไม่ถูกต้อง').optional().or(z.literal('')),
	status: z.boolean().optional()
});

export const load: PageServerLoad = async (event) => {
    const { cookies, depends, parent } = event;
	depends('app:page-data');
	
	const sessionId = cookies.get('session_id');
	if (!sessionId) {
		throw redirect(302, '/admin/login');
	}

	// Get parent data for user context
	const { admin_role } = await parent();

	// For SuperAdmin, show all departments; for FacultyAdmin, show only their faculty's departments
	let apiEndpoint = `/api/departments`;
	if (admin_role?.admin_level === 'FacultyAdmin' && admin_role.faculty_id) {
		apiEndpoint = `/api/faculties/${admin_role.faculty_id}/departments`;
	}

    try {
        // Fetch departments
        const departmentsResponse = await api.get(event, apiEndpoint);

        let departments: Department[] = [];
        if (departmentsResponse.success) {
            const departmentsData = departmentsResponse.data as any;
            departments = departmentsData.departments || departmentsData || [];
        }

		// For FacultyAdmin, get their faculty info
		let currentFaculty: Faculty | null = null;
        if (admin_role?.admin_level === 'FacultyAdmin' && admin_role.faculty_id) {
            const facultyResponse = await api.get(event, `/api/faculties/${admin_role.faculty_id}`);
            if (facultyResponse.success) {
                const facultyData = facultyResponse.data as any;
                currentFaculty = facultyData.faculty || facultyData;
            }
        }

		// If SuperAdmin, load faculties list for selection
		let faculties: Faculty[] | null = null;
        if (admin_role?.admin_level === 'SuperAdmin') {
            const facRes = await api.get(event, `/api/admin/faculties`);
            if (facRes.success) {
                const facData = facRes.data as any;
                faculties = facData.faculties || facData || [];
            }
        }

		// Create forms
		const createForm = await superValidate(zod(departmentCreateSchema));
		const updateForm = await superValidate(zod(departmentUpdateSchema));

		return {
			departments,
			currentFaculty,
			createForm,
			updateForm,
			userRole: admin_role?.admin_level || 'RegularAdmin',
			faculties
		};
	} catch (error) {
		console.error('Failed to load departments data:', error);
		return {
			departments: [],
			currentFaculty: null,
			createForm: await superValidate(zod(departmentCreateSchema)),
			updateForm: await superValidate(zod(departmentUpdateSchema)),
			userRole: admin_role?.admin_level || 'RegularAdmin',
			faculties: null
		};
	}
};

export const actions: Actions = {
    create: async (event) => {
        const { request } = event;

		const user = await requireAdmin(event);
		const admin_role = user.admin_role;

		// Only SuperAdmin and FacultyAdmin can create departments
		if (admin_role?.admin_level !== 'SuperAdmin' && admin_role?.admin_level !== 'FacultyAdmin') {
			return fail(403, { 
				error: 'คุณไม่มีสิทธิ์ในการสร้างภาควิชา'
			});
		}

		const form = await superValidate(request, zod(departmentCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		// Determine the API endpoint based on user role
        // Determine target faculty
        let targetFacultyId: string | null = null;
        if (admin_role?.admin_level === 'FacultyAdmin' && admin_role.faculty_id) {
            targetFacultyId = admin_role.faculty_id;
        } else {
            // SuperAdmin must select a faculty in the form
            const selected = (form.data as any).faculty_id as string | undefined;
            if (!selected) {
                return fail(400, { form, error: 'กรุณาเลือกคณะ' });
            }
            targetFacultyId = selected;
        }

        const apiEndpoint = `/api/faculties/${targetFacultyId}/departments`;

		try {
            const response = await api.post(event, apiEndpoint, form.data);
            if (!response.success) {
                return fail(400, { 
                    form,
                    error: response.error || 'เกิดข้อผิดพลาดในการสร้างภาควิชา'
                });
            }
            return { form, success: true };
		} catch (error) {
			console.error('Failed to create department:', error);
			return fail(500, { 
				form,
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	},

    update: async (event) => {
        const { request } = event;

		const user = await requireAdmin(event);
		const admin_role = user.admin_role;

		// Only SuperAdmin and FacultyAdmin can update departments
		if (admin_role?.admin_level !== 'SuperAdmin' && admin_role?.admin_level !== 'FacultyAdmin') {
			return fail(403, { 
				error: 'คุณไม่มีสิทธิ์ในการแก้ไขภาควิชา'
			});
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;
		const updateData = JSON.parse(formData.get('updateData') as string);

		const form = await superValidate(updateData, zod(departmentUpdateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
            const response = await api.put(event, `/api/departments/${departmentId}`, form.data);
            if (!response.success) {
                return fail(400, { 
                    form,
                    error: response.error || 'เกิดข้อผิดพลาดในการแก้ไขภาควิชา'
                });
            }

			return { form, success: true };
		} catch (error) {
			console.error('Failed to update department:', error);
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

		const user = await requireAdmin(event);
		const admin_role = user.admin_role;

		// Only SuperAdmin and FacultyAdmin can delete departments
		if (admin_role?.admin_level !== 'SuperAdmin' && admin_role?.admin_level !== 'FacultyAdmin') {
			return fail(403, { 
				error: 'คุณไม่มีสิทธิ์ในการลบภาควิชา'
			});
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;

        try {
            const response = await api.delete(event, `/api/departments/${departmentId}`);
            if (!response.success) {
                return fail(400, { 
                    error: response.error || 'เกิดข้อผิดพลาดในการลบภาควิชา'
                });
            }

			return { success: true };
		} catch (error) {
			console.error('Failed to delete department:', error);
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

		const user = await requireAdmin(event);
		const admin_role = user.admin_role;

		// Only SuperAdmin and FacultyAdmin can toggle department status
		if (admin_role?.admin_level !== 'SuperAdmin' && admin_role?.admin_level !== 'FacultyAdmin') {
			return fail(403, { 
				error: 'คุณไม่มีสิทธิ์ในการเปลี่ยนสถานะภาควิชา'
			});
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;

        try {
            const response = await api.put(event, `/api/departments/${departmentId}/toggle-status`);
            if (!response.success) {
                return fail(400, { 
                    error: response.error || 'เกิดข้อผิดพลาดในการเปลี่ยนสถานะภาควิชา'
                });
            }

			return { success: true };
		} catch (error) {
			console.error('Failed to toggle department status:', error);
			return fail(500, { 
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	},

};
