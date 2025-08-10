import { redirect, fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';
import type { Department, Faculty } from '$lib/types/admin';
import { PUBLIC_API_URL } from '$env/static/public';
import { requireAdmin } from '$lib/server/auth';

// Department schemas
const departmentCreateSchema = z.object({
	name: z.string().min(1, 'กรุณากรอกชื่อภาควิชา'),
	code: z.string().min(1, 'กรุณากรอกรหัสภาควิชา'),
	description: z.string().optional(),
	head_name: z.string().optional(),
	head_email: z.string().email('รูปแบบอีเมลไม่ถูกต้อง').optional().or(z.literal('')),
	status: z.boolean().default(true)
});

const departmentUpdateSchema = z.object({
	name: z.string().min(1, 'กรุณากรอกชื่อภาควิชา').optional(),
	code: z.string().min(1, 'กรุณากรอกรหัสภาควิชา').optional(),
	description: z.string().optional(),
	head_name: z.string().optional(),
	head_email: z.string().email('รูปแบบอีเมลไม่ถูกต้อง').optional().or(z.literal('')),
	status: z.boolean().optional()
});

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async ({ cookies, depends, fetch, parent }) => {
	depends('app:page-data');
	
	const sessionId = cookies.get('session_id');
	if (!sessionId) {
		throw redirect(302, '/admin/login');
	}

	// Get parent data for user context
	const { admin_role } = await parent();

	// For SuperAdmin, show all departments; for FacultyAdmin, show only their faculty's departments
	let apiEndpoint = `${API_BASE_URL}/api/departments`;
	if (admin_role?.admin_level === 'FacultyAdmin' && admin_role.faculty_id) {
		apiEndpoint = `${API_BASE_URL}/api/faculties/${admin_role.faculty_id}/departments`;
	}

	try {
		// Fetch departments
		const departmentsResponse = await fetch(apiEndpoint, {
			headers: {
				'Cookie': `session_id=${sessionId}`
			}
		});

		let departments: Department[] = [];
		if (departmentsResponse.ok) {
			const departmentsData = await departmentsResponse.json();
			if (departmentsData.status === 'success') {
				departments = departmentsData.data.departments || [];
			}
		}

		// For FacultyAdmin, get their faculty info
		let currentFaculty: Faculty | null = null;
		if (admin_role?.admin_level === 'FacultyAdmin' && admin_role.faculty_id) {
			const facultyResponse = await fetch(`${API_BASE_URL}/api/faculties/${admin_role.faculty_id}`, {
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			if (facultyResponse.ok) {
				const facultyData = await facultyResponse.json();
				if (facultyData.status === 'success') {
					currentFaculty = facultyData.data.faculty;
				}
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
			userRole: admin_role?.admin_level || 'RegularAdmin'
		};
	} catch (error) {
		console.error('Failed to load departments data:', error);
		return {
			departments: [],
			currentFaculty: null,
			createForm: await superValidate(zod(departmentCreateSchema)),
			updateForm: await superValidate(zod(departmentUpdateSchema)),
			userRole: admin_role?.admin_level || 'RegularAdmin'
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

		const user = await requireAdmin(event);
		const admin_role = user.admin_role;
		const form = await superValidate(request, zod(departmentCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		// Determine the API endpoint based on user role
		let apiEndpoint = `${API_BASE_URL}/api/departments`;
		if (admin_role?.admin_level === 'FacultyAdmin' && admin_role.faculty_id) {
			apiEndpoint = `${API_BASE_URL}/api/faculties/${admin_role.faculty_id}/departments`;
		}

		try {
			const response = await fetch(apiEndpoint, {
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
					error: result.message || result.error || 'เกิดข้อผิดพลาดในการสร้างภาควิชา'
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

	update: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;
		const updateData = JSON.parse(formData.get('updateData') as string);

		const form = await superValidate(updateData, zod(departmentUpdateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const response = await fetch(`${API_BASE_URL}/api/departments/${departmentId}`, {
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
					error: result.message || 'เกิดข้อผิดพลาดในการแก้ไขภาควิชา'
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

	delete: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;

		try {
			const response = await fetch(`${API_BASE_URL}/api/departments/${departmentId}`, {
				method: 'DELETE',
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					error: result.message || result.error || 'เกิดข้อผิดพลาดในการลบภาควิชา'
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

	toggleStatus: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;

		try {
			const response = await fetch(`${API_BASE_URL}/api/departments/${departmentId}/toggle-status`, {
				method: 'PUT',
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการเปลี่ยนสถานะภาควิชา'
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

	assignAdmin: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;
		const adminId = formData.get('adminId') as string;

		try {
			const response = await fetch(`${API_BASE_URL}/api/departments/${departmentId}/assign-admin`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify({ admin_id: adminId })
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการมอบหมายแอดมิน'
				});
			}

			return { success: true };
		} catch (error) {
			console.error('Failed to assign department admin:', error);
			return fail(500, { 
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	},

	removeAdmin: async ({ request, cookies }) => {
		const sessionId = cookies.get('session_id');
		if (!sessionId) {
			throw redirect(302, '/admin/login');
		}

		const formData = await request.formData();
		const departmentId = formData.get('departmentId') as string;
		const adminId = formData.get('adminId') as string;

		try {
			const response = await fetch(`${API_BASE_URL}/api/departments/${departmentId}/remove-admin`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify({ admin_id: adminId })
			});

			const result = await response.json();

			if (!response.ok) {
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการถอดถอนแอดมิน'
				});
			}

			return { success: true };
		} catch (error) {
			console.error('Failed to remove department admin:', error);
			return fail(500, { 
				error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์'
			});
		}
	}
};