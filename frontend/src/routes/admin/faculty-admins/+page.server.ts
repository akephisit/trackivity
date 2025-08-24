import { requireFacultyAdmin } from '$lib/server/auth';
import { redirect, fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { adminCreateSchema } from '$lib/schemas/auth';
import type { PageServerLoad, Actions } from './$types';
import type { 
	Faculty, 
	FacultyAdminStats, 
	ExtendedAdminRole
} from '$lib/types/admin';
import { AdminLevel, ADMIN_PERMISSIONS } from '$lib/types/admin';
import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
	// Role-based access: Both SuperAdmin and FacultyAdmin can access this page
	const user = await requireFacultyAdmin(event);
	const isSuperAdmin = user.admin_role?.admin_level === AdminLevel.SuperAdmin;
	const userFacultyId = user.admin_role?.faculty_id;

	// Load faculties list
	let faculties: Faculty[] = [];
	try {
        const response = await api.get(event, '/api/faculties');
        
        if (response.success) {
            faculties = response.data?.faculties || response.data || [];
        }
	} catch (error) {
		console.error('Failed to load faculties:', error);
	}

	// Load faculty admins based on user role
	let facultyAdmins: ExtendedAdminRole[] = [];
	try {
		let apiUrl: string;
		
		if (isSuperAdmin) {
			apiUrl = `/api/admin/system-admins`;
		} else {
			apiUrl = `/api/faculties/${userFacultyId}/admins`;
		}

        const response = await api.get(event, apiUrl);

        if (response.success && response.data) {
            const result = response.data;
				const adminData = result.users || result.admins || result || [];
				
				// Ensure adminData is an array
				if (!Array.isArray(adminData)) {
					console.warn('adminData is not an array:', typeof adminData, adminData);
					throw new Error('Invalid data format received from server');
				}
				
				// Helper function to convert API AdminLevel to Frontend AdminLevel
				const mapAdminLevel = (apiLevel: string): AdminLevel => {
					switch (apiLevel) {
						case 'SuperAdmin':
							return AdminLevel.SuperAdmin;
						case 'FacultyAdmin':
							return AdminLevel.FacultyAdmin;
						case 'RegularAdmin':
							return AdminLevel.RegularAdmin;
						default:
							return AdminLevel.RegularAdmin;
					}
				};

				// Enhanced mapping with additional properties
				facultyAdmins = adminData
					.filter((admin: any) => {
						if (admin.admin_role) {
							const level = mapAdminLevel(admin.admin_role.admin_level);
							// Show both FacultyAdmin and RegularAdmin (general faculty admins)
							// For SuperAdmin view: show all FacultyAdmin and RegularAdmin with faculty assignment
							// For FacultyAdmin view: show FacultyAdmin and RegularAdmin in their faculty
							return (level === AdminLevel.FacultyAdmin || level === AdminLevel.RegularAdmin);
						}
						return false;
					})
					.map((admin: any): ExtendedAdminRole => {
						const lastLogin = admin.last_login ? new Date(admin.last_login) : null;
						const createdAt = new Date(admin.admin_role.created_at);
						const now = new Date();
						
						return {
							id: admin.admin_role.id,
							user_id: admin.id,
							admin_level: mapAdminLevel(admin.admin_role.admin_level),
							faculty_id: admin.admin_role.faculty_id,
							permissions: admin.admin_role.permissions || [],
							is_enabled: admin.admin_role.is_enabled ?? true,
							created_at: admin.admin_role.created_at,
							updated_at: admin.admin_role.updated_at,
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
							faculty: admin.admin_role.faculty_id ? 
								faculties.find(f => f.id === admin.admin_role.faculty_id) : undefined,
							// Enhanced properties - use is_active from backend AdminUserInfo
							is_active: admin.is_active || false, // Backend calculates this based on active sessions
							last_login_formatted: lastLogin ? formatDateTime(lastLogin) : 'ยังไม่เคยเข้าใช้',
							created_at_formatted: formatDateTime(createdAt),
							permission_count: admin.admin_role.permissions.length,
							days_since_last_login: lastLogin ? Math.floor((now.getTime() - lastLogin.getTime()) / (1000 * 60 * 60 * 24)) : undefined,
							full_name: `${admin.first_name} ${admin.last_name}`
						};
					});
		}
	} catch (error) {
		console.error('Failed to load faculty admins:', error);
		facultyAdmins = [];
	}

	// Calculate comprehensive statistics
	const stats: FacultyAdminStats = {
		total_admins: facultyAdmins.length,
		active_admins: facultyAdmins.filter(admin => admin.is_active).length,
		inactive_admins: facultyAdmins.filter(admin => !admin.is_active).length,
		recent_logins: facultyAdmins.filter(admin => {
			return admin.days_since_last_login !== undefined && admin.days_since_last_login <= 7;
		}).length,
		total_faculties: isSuperAdmin ? faculties.length : 1,
		faculty_breakdown: faculties.map(faculty => {
			const facultyAdmins2 = facultyAdmins.filter(admin => admin.faculty_id === faculty.id);
			return {
				faculty_id: faculty.id,
				faculty_name: faculty.name,
				admin_count: facultyAdmins2.length,
				active_count: facultyAdmins2.filter(admin => admin.is_active).length
			};
		}),
		permission_breakdown: Object.values(ADMIN_PERMISSIONS).map(permission => ({
			permission,
			count: facultyAdmins.filter(admin => admin.permissions.includes(permission)).length
		}))
	};

	const form = await superValidate(zod(adminCreateSchema));

	return {
		user,
		facultyAdmins,
		faculties,
		stats,
		form,
		isSuperAdmin,
		userFacultyId,
		currentFaculty: userFacultyId ? faculties.find(f => f.id === userFacultyId) : null
	};
};

// Helper functions
function formatDateTime(date: Date): string {
	return date.toLocaleDateString('th-TH', {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	});
}


export const actions: Actions = {
	create: async (event) => {
		const { request } = event;
		const form = await superValidate(request, zod(adminCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			// Authorization check - use proxy
            const authResponse = await api.get(event, `/api/admin/auth/me`);

            if (!authResponse.success) {
                form.errors._errors = ['ไม่สามารถยืนยันตัวตนได้'];
                return fail(401, { form });
            }
            const authResult = authResponse.data as any;
            const userLevel = authResult.user?.admin_role?.admin_level;
            const userFacultyId = authResult.user?.admin_role?.faculty_id;

			// Check authorization based on admin levels
			if (form.data.admin_level === AdminLevel.FacultyAdmin) {
				// Only SuperAdmin can create FacultyAdmin
				if (userLevel !== AdminLevel.SuperAdmin) {
					form.errors._errors = ['เฉพาะซุปเปอร์แอดมินเท่านั้นที่สามารถสร้างแอดมินคณะได้'];
					return fail(403, { form });
				}
			} else if (form.data.admin_level === AdminLevel.RegularAdmin) {
				// Both SuperAdmin and FacultyAdmin can create RegularAdmin
				if (userLevel !== AdminLevel.SuperAdmin && userLevel !== AdminLevel.FacultyAdmin) {
					form.errors._errors = ['ไม่มีสิทธิ์ในการสร้างแอดมินประเภทนี้'];
					return fail(403, { form });
				}
				// FacultyAdmin can only create RegularAdmin in their own faculty
				if (userLevel === AdminLevel.FacultyAdmin && form.data.faculty_id !== userFacultyId) {
					form.errors._errors = ['แอดมินคณะสามารถสร้างแอดมินได้เฉพาะในคณะของตนเองเท่านั้น'];
					return fail(403, { form });
				}
			}

			// Define default permissions based on admin level
			const getDefaultPermissions = (adminLevel: string) => {
				if (adminLevel === AdminLevel.FacultyAdmin) {
					return [
						'ViewDashboard',
						'ManageActivities',
						'ManageUsers',
						'ManageFacultyUsers'
					];
				} else if (adminLevel === AdminLevel.RegularAdmin) {
					return [
						'ViewDashboard',
						'ManageActivities'
					];
				}
				return ['ViewDashboard'];
			};

			// Transform form data to match backend expectations
			const adminPrefix = form.data.admin_level === AdminLevel.FacultyAdmin ? 'FA' : 'RA'; // RA for RegularAdmin
			const defaultPassword = form.data.admin_level === AdminLevel.FacultyAdmin ? 'FacAdmin123!' : 'RegAdmin123!';
			
			const transformedData = {
				student_id: `${adminPrefix}${Date.now()}`, // Generate admin ID with appropriate prefix
				email: form.data.email,
				password: form.data.password || defaultPassword,
				first_name: form.data.first_name,
				last_name: form.data.last_name,
				department_id: null,
				admin_level: form.data.admin_level, // Use the provided admin level
				faculty_id: form.data.faculty_id && form.data.faculty_id !== '' ? form.data.faculty_id : null,
				permissions: getDefaultPermissions(form.data.admin_level)
			};

			console.log('Creating faculty admin with data:', {
				admin_level: transformedData.admin_level,
				faculty_id: transformedData.faculty_id,
				permissions: transformedData.permissions
			});

			// Use faculty-specific admin creation endpoint
			const endpoint = transformedData.faculty_id 
				? `/api/faculties/${transformedData.faculty_id}/admins`
				: `/api/admin/create`;

				const response = await api.post(event, endpoint, transformedData);

				if (response.success) {
					return { form, success: true, message: response.message || 'สร้างแอดมินคณะสำเร็จ' };
				} else {
					form.errors._errors = [response.error || 'เกิดข้อผิดพลาดในการสร้างแอดมินคณะ'];
					return fail(400, { form });
				}
		} catch (error) {
			console.error('Create faculty admin error:', error);
			
			if (error instanceof TypeError && error.message.includes('fetch')) {
				form.errors._errors = ['เกิดข้อผิดพลาดในการเชื่อมต่อกับเซิร์ฟเวอร์'];
			} else if (error instanceof Error) {
				form.errors._errors = [`เกิดข้อผิดพลาด: ${error.message}`];
			} else {
				form.errors._errors = ['เกิดข้อผิดพลาดไม่ทราบสาเหตุในการสร้างแอดมินคณะ'];
			}
			return fail(500, { form });
		}
	},

	toggleStatus: async (event) => {
		const { request } = event;
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string;
		const isActive = formData.get('isActive') === 'true';

		if (!adminId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		try {
			// Verify authorization
            const authResponse = await api.get(event, `/api/admin/auth/me`);

            if (!authResponse.success) {
                return fail(401, { error: 'ไม่สามารถยืนยันตัวตนได้' });
            }
            const authResult = authResponse.data as any;
            const userLevel = authResult.user?.admin_role?.admin_level;

			// Only SuperAdmin can toggle status, or FacultyAdmin for their own faculty
			if (userLevel !== AdminLevel.SuperAdmin) {
				// For FacultyAdmin, check if they're managing their own faculty
				// This would require additional validation against the target admin's faculty
				return fail(403, { error: 'ไม่มีสิทธิ์ในการเปลี่ยนสถานะแอดมินนี้' });
			}

            const response = await api.put(event, `/api/admin/roles/${adminId}/toggle-status`, { is_active: isActive });

				if (response.success) {
					return { success: true, message: response.message || `${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมินคณะสำเร็จ`, data: response.data };
				} else {
					return fail(400, { error: response.error || `เกิดข้อผิดพลาดในการ${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมินคณะ` });
				}
		} catch (error) {
			console.error('Toggle faculty admin status error:', error);
			return fail(500, { error: 'เกิดข้อผิดพลาดในการเปลี่ยนสถานะแอดมินคณะ' });
		}
	},

	update: async (event) => {
		const { request } = event;
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string;
		const userId = formData.get('userId') as string;
		const updateDataString = formData.get('updateData') as string;

		if (!adminId && !userId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		if (!updateDataString) {
			return fail(400, { error: 'ไม่พบข้อมูลที่ต้องการอัพเดต' });
		}

		let updateData;
		try {
			updateData = JSON.parse(updateDataString);
		} catch (parseError) {
			console.error('JSON parse error:', parseError);
			return fail(400, { error: 'ข้อมูลที่ส่งมาไม่อยู่ในรูปแบบที่ถูกต้อง' });
		}

		try {
			const targetUserId = userId || adminId;

			// Verify authorization
            const authResponse = await api.get(event, `/api/admin/auth/me`);

            if (!authResponse.success) {
                return fail(401, { error: 'ไม่สามารถยืนยันตัวตนได้' });
            }
            const authResult = authResponse.data as any;
            const userLevel = authResult.user?.admin_role?.admin_level;

			// Only SuperAdmin can update faculty admin info
			if (userLevel !== AdminLevel.SuperAdmin) {
				return fail(403, { error: 'ไม่มีสิทธิ์ในการแก้ไขข้อมูลแอดมินคณะ' });
			}

			const requiredFields = ['first_name', 'last_name', 'email'];
			const missingFields = requiredFields.filter(field => !updateData[field]);
			
			if (missingFields.length > 0) {
				return fail(400, { 
					error: `ข้อมูลไม่ครบถ้วน: ${missingFields.join(', ')}` 
				});
			}

			const preparedData = {
				first_name: updateData.first_name,
				last_name: updateData.last_name,
				email: updateData.email,
				...(updateData.department_id !== undefined && { department_id: updateData.department_id || null })
			};

            const response = await api.put(event, `/api/users/${targetUserId}`, preparedData);
            if (!response.success) {
                return fail(400, { error: response.error || 'เกิดข้อผิดพลาดในการอัพเดตข้อมูลแอดมินคณะ' });
            }
            return { success: true, message: 'อัพเดตข้อมูลแอดมินคณะสำเร็จ', data: response.data };
		} catch (error) {
			console.error('Update faculty admin error:', error);
			return fail(500, { error: 'เกิดข้อผิดพลาดในการอัพเดตข้อมูลแอดมินคณะ' });
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
			// Verify authorization - only SuperAdmin can delete faculty admins
            const authResponse = await api.get(event, `/api/admin/auth/me`);

            if (!authResponse.success) {
                return fail(401, { error: 'ไม่สามารถยืนยันตัวตนได้' });
            }

            const authResult = authResponse.data as any;
            if (authResult.user?.admin_role?.admin_level !== AdminLevel.SuperAdmin) {
                return fail(403, { error: 'เฉพาะซุปเปอร์แอดมินเท่านั้นที่สามารถลบแอดมินคณะได้' });
            }
            
            const response = await api.delete(event, `/api/users/${adminId}`);

            const result = response;

            if (!response.success) {
                return fail(400, { error: response.error || 'เกิดข้อผิดพลาดในการลบแอดมินคณะ' });
            }

            if (response.success) {
                return { 
                    success: true, 
                    message: 'ลบแอดมินคณะสำเร็จ' 
                };
            } else {
                return fail(400, { 
                    error: response.error || 'เกิดข้อผิดพลาดในการลบแอดมินคณะ' 
                });
            }
		} catch (error) {
			console.error('Delete faculty admin error:', error);
			return fail(500, { error: 'เกิดข้อผิดพลาดในการลบแอดมินคณะ' });
		}
	}
};
