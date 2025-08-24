import { requireSuperAdmin } from '$lib/server/auth';
import { fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { adminCreateSchema } from '$lib/schemas/auth';
import type { PageServerLoad, Actions } from './$types';
import type { AdminRole, Faculty } from '$lib/types/admin';
import { AdminLevel } from '$lib/types/admin';
import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
	const user = await requireSuperAdmin(event);

	// โหลดรายการคณะก่อน
	let faculties: Faculty[] = [];
	try {
    const response = await api.get(event, '/api/faculties');
    if (response.success) {
        faculties = response.data?.faculties || response.data || [];
    }
	} catch (error) {
		console.error('Failed to load faculties:', error);
	}

	// โหลดรายการแอดมิน - ใช้ system-admins endpoint เพื่อให้ได้ข้อมูล is_active
	let admins: AdminRole[] = [];
	try {
    const response = await api.get(event, '/api/admin/system-admins');

    if (response.success) {
        const result = response.data;
			console.log('=== SYSTEM ADMINS API RESPONSE ===');
			console.log('result type:', typeof result);
			if (result) {
				console.log('super_admins count:', result.super_admins?.length || 0);
				console.log('faculty_groups count:', result.faculty_groups?.length || 0);
			}
			console.log('================================');
			if (result) {
				// API ส่งข้อมูลในรูปแบบ { super_admins: [], faculty_groups: [...] }
				let adminUsers: any[] = [];
				
				// รวม super_admins เข้าด้วย
				if (result.super_admins && Array.isArray(result.super_admins)) {
					adminUsers = [...result.super_admins];
				}
				
				// รวม admins จาก faculty_groups
				if (result.faculty_groups && Array.isArray(result.faculty_groups)) {
					result.faculty_groups.forEach((group: any) => {
						if (group.admins && Array.isArray(group.admins)) {
							adminUsers = [...adminUsers, ...group.admins];
						}
					});
				}
				
				// Ensure adminUsers is an array to prevent .filter() error
				if (!Array.isArray(adminUsers)) {
					console.warn('adminUsers is not an array:', typeof adminUsers, adminUsers);
					throw new Error('Invalid data format received from server');
				}
				
				// Helper function to convert API AdminLevel to Frontend AdminLevel
				const mapAdminLevel = (apiLevel: string): AdminLevel => {
					switch (apiLevel) {
						case 'SuperAdmin':
						case 'super_admin':
							return AdminLevel.SuperAdmin;
						case 'FacultyAdmin':
						case 'faculty_admin':
							return AdminLevel.FacultyAdmin;
						case 'RegularAdmin':
						case 'regular_admin':
							return AdminLevel.RegularAdmin;
						default:
							return AdminLevel.RegularAdmin;
					}
				};

				// แปลงข้อมูลจาก system-admins API response ให้เป็น AdminRole format ที่ frontend ใช้
				// API response structure: { id, email, first_name, ..., admin_role: {...}, is_active, last_login }
				admins = adminUsers
					.filter((admin: any) => admin.admin_role) // เฉพาะ admin ที่มี admin_role
					.map((admin: any) => ({
						id: admin.admin_role.id,
						user_id: admin.id,
						admin_level: mapAdminLevel(admin.admin_role.admin_level),
						faculty_id: admin.admin_role.faculty_id,
						permissions: admin.admin_role.permissions || [],
						created_at: admin.admin_role.created_at,
						updated_at: admin.admin_role.updated_at,
						// เพิ่มข้อมูล user เข้าไปด้วยเพื่อให้ UI แสดงได้
						user: {
							id: admin.id,
							student_id: admin.student_id,
							email: admin.email,
							first_name: admin.first_name,
							last_name: admin.last_name,
							department_id: admin.department_id,
							faculty_id: admin.faculty_id,
							status: admin.status || 'active',
							role: admin.role || 'admin',
							phone: admin.phone,
							avatar: admin.avatar,
							last_login: admin.last_login,
							email_verified_at: admin.email_verified_at,
							created_at: admin.created_at,
							updated_at: admin.updated_at
						},
						// เพิ่ม faculty ข้อมูลถ้ามี
						faculty: admin.admin_role.faculty_id ? 
							faculties.find(f => f.id === admin.admin_role.faculty_id) : undefined,
						// เพิ่ม is_active (login session) และ is_enabled (account enabled) ข้อมูลจาก backend
						is_active: admin.is_active !== undefined ? admin.is_active : false,
						is_enabled: admin.is_enabled !== undefined ? admin.is_enabled : true
					}));
			}
		}
	} catch (error) {
		console.error('Failed to load admins:', error);
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
	create: async (event) => {
		const { request } = event;
		const form = await superValidate(request, zod(adminCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			// Define default permissions based on admin level
			const getDefaultPermissions = (adminLevel: string, _facultyId?: string) => {
				switch (adminLevel) {
					case 'SuperAdmin':
						return [
							'ManageUsers',
							'ManageAdmins',
							'ManageActivities',
							'ViewDashboard',
							'ManageFaculties',
							'ManageSessions'
						];
					case 'FacultyAdmin':
						return [
							'ViewDashboard',
							'ManageActivities',
							'ManageUsers'
						];
					case 'RegularAdmin':
					default:
						return [
							'ViewDashboard',
							'ManageActivities'
						];
				}
			};

			// Transform form data to match backend expectations
			const transformedData = {
				student_id: `A${Date.now()}`, // Generate temporary student_id for admin with prefix
				email: form.data.email,
				password: form.data.password || 'TempPass123!', // Use provided password or temp password
				prefix: form.data.prefix, // Add prefix field
				first_name: form.data.first_name,
				last_name: form.data.last_name,
				department_id: null,
				admin_level: form.data.admin_level, // ใช้ admin_level ที่ส่งมาจาก form โดยตรง
				faculty_id: form.data.faculty_id && form.data.faculty_id !== '' ? form.data.faculty_id : null,
				permissions: getDefaultPermissions(form.data.admin_level, form.data.faculty_id)
			};

			console.log('Creating admin with data:', {
				admin_level: transformedData.admin_level,
				faculty_id: transformedData.faculty_id,
				permissions: transformedData.permissions,
				form_data_admin_level: form.data.admin_level,
				form_data_faculty_id: form.data.faculty_id
			});

			
        const response = await api.post(event, '/api/admin/create', transformedData);

        if (!response.success) {
            form.errors._errors = [response.error || 'เกิดข้อผิดพลาดในการสร้างแอดมิน'];
            return fail(400, { form });
        }

        if (response.success) {
            return { form, success: true, message: 'สร้างแอดมินสำเร็จ' };
        } else {
            form.errors._errors = ['เกิดข้อผิดพลาดในการสร้างแอดมิน'];
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

	delete: async (event) => {
		const { request } = event;
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string;

		if (!adminId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		try {
        const response = await api.delete(event, `/api/users/${adminId}`);

        if (!response.success) {
            return fail(400, { 
                error: response.error || 'เกิดข้อผิดพลาดในการลบแอดมิน' 
            });
        }

        if (response.success) {
            return { 
                success: true, 
                message: 'ลบแอดมินสำเร็จ' 
            };
        } else {
            return fail(400, { 
                error: 'เกิดข้อผิดพลาดในการลบแอดมิน' 
            });
        }
		} catch (error) {
			console.error('Delete admin error:', error);
			
			if (error instanceof TypeError && error.message.includes('fetch')) {
				return fail(500, { error: 'เกิดข้อผิดพลาดในการเชื่อมต่อกับเซิร์ฟเวอร์' });
			} else if (error instanceof SyntaxError) {
				return fail(500, { error: 'เกิดข้อผิดพลาดในการประมวลผลข้อมูลจากเซิร์ฟเวอร์' });
			}
			return fail(500, { error: 'เกิดข้อผิดพลาดไม่ทราบสาเหตุในการลบแอดมิน' });
		}
	},

	toggleStatus: async (event) => {
		const { request } = event;
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string; // admin role id
		const isActive = formData.get('isActive') === 'true';

		if (!adminId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		try {
        const response = await api.put(event, `/api/admin/roles/${adminId}/toggle-status`, {
            is_enabled: isActive  // Send is_enabled instead of is_active
        });

        if (!response.success) {
            return fail(400, { 
                error: response.error || `เกิดข้อผิดพลาดในการ${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมิน` 
            });
        }

        if (response.success) {
            return { 
                success: true, 
                message: response.message || `${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมินสำเร็จ`,
                data: response.data
            };
        } else {
            return fail(400, { 
                error: `เกิดข้อผิดพลาดในการ${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมิน` 
            });
        }
		} catch (error) {
			console.error('Toggle admin status error:', error);
			
			if (error instanceof TypeError && error.message.includes('fetch')) {
				return fail(500, { error: 'เกิดข้อผิดพลาดในการเชื่อมต่อกับเซิร์ฟเวอร์' });
			} else if (error instanceof SyntaxError) {
				return fail(500, { error: 'เซิร์ฟเวอร์ส่งข้อมูลที่ไม่ถูกต้อง' });
			} else if (error instanceof Error) {
				return fail(500, { error: `เกิดข้อผิดพลาด: ${error.message}` });
			} else {
				return fail(500, { error: 'เกิดข้อผิดพลาดไม่ทราบสาเหตุในการเปลี่ยนสถานะแอดมิน' });
			}
		}
	},

	update: async (event) => {
		const { request } = event;
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string;
		const userId = formData.get('userId') as string; // รับ user_id แทน admin_id
		const updateDataString = formData.get('updateData') as string;

		if (!adminId && !userId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		if (!updateDataString) {
			return fail(400, { error: 'ไม่พบข้อมูลที่ต้องการอัพเดต' });
		}

		// ใช้ userId หากมี, ถ้าไม่มีให้ใช้ adminId
		const targetUserId = userId || adminId;

		let updateData;
		try {
			updateData = JSON.parse(updateDataString);
		} catch (parseError) {
			console.error('JSON parse error:', parseError, 'Raw data:', updateDataString);
			return fail(400, { error: 'ข้อมูลที่ส่งมาไม่อยู่ในรูปแบบที่ถูกต้อง' });
		}

		try {
			// ตรวจสอบว่ามีข้อมูลที่จำเป็นครบถ้วน
			const requiredFields = ['first_name', 'last_name', 'email'];
			const missingFields = requiredFields.filter(field => !updateData[field]);
			
			if (missingFields.length > 0) {
				return fail(400, { 
					error: `ข้อมูลไม่ครบถ้วน: ${missingFields.join(', ')}` 
				});
			}

			// เตรียมข้อมูลสำหรับส่งไป backend ผ่าน user endpoint
			// ตาม API structure ใน backend/handlers/user.rs
			const preparedData = {
				first_name: updateData.first_name,
				last_name: updateData.last_name,
				email: updateData.email,
				// department_id สำหรับ user update
				...(updateData.department_id !== undefined && { department_id: updateData.department_id || null })
				// Note: admin_level และ permissions จะต้องจัดการแยกผ่าน admin_roles table
				// ซึ่งตอนนี้ backend ยังไม่มี endpoint สำหรับนั้น
			};

			// ใช้ user endpoint ตาม backend routes
        const response = await api.put(event, `/api/users/${targetUserId}`, preparedData);

        if (!response.success) {
            return fail(400, { 
                error: response.error || 'เกิดข้อผิดพลาดในการอัพเดตข้อมูลแอดมิน' 
            });
        }

        if (response.success) {
            return { 
                success: true, 
                message: 'อัพเดตข้อมูลแอดมินสำเร็จ',
                data: response.data
            };
        } else {
            return fail(400, { 
                error: 'เกิดข้อผิดพลาดในการอัพเดตข้อมูลแอดมิน' 
            });
        }
		} catch (error) {
			console.error('Update admin error:', error);
			
			// ให้ error handling ที่ดีขึ้น
			if (error instanceof TypeError && error.message.includes('fetch')) {
				return fail(500, { error: 'เกิดข้อผิดพลาดในการเชื่อมต่อกับเซิร์ฟเวอร์ กรุณาตรวจสอบว่า Backend Server กำลังทำงานอยู่' });
			} else if (error instanceof SyntaxError) {
				return fail(500, { error: 'เกิดข้อผิดพลาดในการประมวลผลข้อมูลจากเซิร์ฟเวอร์' });
			} else if (error instanceof Error) {
				return fail(500, { error: `เกิดข้อผิดพลาด: ${error.message}` });
			} else {
				return fail(500, { error: 'เกิดข้อผิดพลาดไม่ทราบสาเหตุในการอัพเดตข้อมูลแอดมิน' });
			}
		}
	}
};
