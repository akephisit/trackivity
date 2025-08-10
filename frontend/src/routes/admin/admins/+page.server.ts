import { requireSuperAdmin } from '$lib/server/auth';
import { fail } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { adminCreateSchema } from '$lib/schemas/auth';
import type { PageServerLoad, Actions } from './$types';
import type { AdminRole, Faculty } from '$lib/types/admin';
import { PUBLIC_API_URL } from '$env/static/public';

const API_BASE_URL = PUBLIC_API_URL || 'http://localhost:3000';

export const load: PageServerLoad = async (event) => {
	const user = await requireSuperAdmin(event);
	const sessionId = event.cookies.get('session_id');

	// โหลดรายการคณะก่อน
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
			if (result.status === 'success' && result.data) {
				// API ส่งข้อมูลในรูปแบบ { users: [], ... } ไม่ใช่ array โดยตรง
				const adminUsers = result.data.users || [];
				
				// Helper function to convert API AdminLevel to Frontend AdminLevel
				const mapAdminLevel = (apiLevel: string) => {
					switch (apiLevel) {
						case 'SuperAdmin':
							return 'SuperAdmin';
						case 'FacultyAdmin':
							return 'FacultyAdmin';
						case 'RegularAdmin':
							return 'RegularAdmin';
						default:
							return 'RegularAdmin';
					}
				};

				// แปลงข้อมูลจาก AdminUserInfo ให้เป็น AdminRole format ที่ frontend ใช้
				admins = adminUsers
					.filter((user: any) => user.admin_role) // เฉพาะ user ที่มี admin role
					.map((user: any) => ({
						id: user.admin_role.id,
						user_id: user.id,
						admin_level: mapAdminLevel(user.admin_role.admin_level),
						faculty_id: user.admin_role.faculty_id,
						permissions: user.admin_role.permissions || [],
						created_at: user.admin_role.created_at,
						updated_at: user.admin_role.updated_at,
						// เพิ่มข้อมูล user เข้าไปด้วยเพื่อให้ UI แสดงได้
						user: {
							id: user.id,
							student_id: user.student_id,
							email: user.email,
							first_name: user.first_name,
							last_name: user.last_name,
							department_id: user.department_id,
							created_at: user.created_at
						},
						// เพิ่ม faculty ข้อมูลถ้ามี
						faculty: user.admin_role.faculty_id ? 
							faculties.find(f => f.id === user.admin_role.faculty_id) : undefined
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
	create: async ({ request, cookies }) => {
		const form = await superValidate(request, zod(adminCreateSchema));

		if (!form.valid) {
			return fail(400, { form });
		}

		try {
			const sessionId = cookies.get('session_id');
			
			// Define default permissions based on admin level
			const getDefaultPermissions = (adminLevel: string, facultyId?: string) => {
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
				first_name: form.data.name.split(' ')[0] || form.data.name,
				last_name: form.data.name.split(' ').slice(1).join(' ') || 'Admin',
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
			
			// ใช้ user_id จากข้อมูล admin ที่ส่งมาจาก frontend
			const response = await fetch(`${API_BASE_URL}/api/users/${adminId}`, {
				method: 'DELETE',
				headers: {
					'Cookie': `session_id=${sessionId}`
				}
			});

			// ตรวจสอบว่า response มี content หรือไม่
			const contentType = response.headers.get('content-type');
			let result;
			
			if (contentType && contentType.includes('application/json')) {
				result = await response.json();
			} else {
				// ถ้าไม่มี JSON response ให้สร้าง default result
				result = {
					status: response.ok ? 'success' : 'error',
					message: response.ok ? 'User deleted successfully' : 'Failed to delete user'
				};
			}

			if (!response.ok) {
				// จัดการ specific error cases
				if (response.status === 404) {
					return fail(404, { error: 'ไม่พบแอดมินที่ต้องการลบ' });
				} else if (response.status === 403) {
					return fail(403, { error: 'ไม่มีสิทธิ์ในการลบแอดมินนี้' });
				}
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการลบแอดมิน' 
				});
			}

			// ตรวจสอบ result structure
			if (result.status === 'success' || response.ok) {
				return { 
					success: true, 
					message: 'ลบแอดมินสำเร็จ' 
				};
			} else {
				return fail(400, { 
					error: result.message || 'เกิดข้อผิดพลาดในการลบแอดมิน' 
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

	toggleStatus: async ({ request, cookies }) => {
		const formData = await request.formData();
		const adminId = formData.get('adminId') as string; // admin role id
		const isActive = formData.get('isActive') === 'true';

		if (!adminId) {
			return fail(400, { error: 'ไม่พบ ID ของแอดมิน' });
		}

		try {
			const sessionId = cookies.get('session_id');
			
			// ใช้ admin toggle status endpoint ที่เพิ่งสร้างขึ้น
			const response = await fetch(`${API_BASE_URL}/api/admin/roles/${adminId}/toggle-status`, {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify({
					is_active: isActive
				})
			});

			const contentType = response.headers.get('content-type');
			let result;
			
			if (contentType && contentType.includes('application/json')) {
				try {
					result = await response.json();
				} catch (parseError) {
					console.error('Failed to parse JSON response:', parseError);
					return fail(500, { 
						error: 'เกิดข้อผิดพลาดในการประมวลผลข้อมูลจากเซิร์ฟเวอร์' 
					});
				}
			} else {
				const responseText = await response.text();
				console.error('Non-JSON response received:', responseText);
				return fail(500, { 
					error: 'เซิร์ฟเวอร์ตอบกลับข้อมูลในรูปแบบที่ไม่ถูกต้อง' 
				});
			}

			if (!response.ok) {
				if (response.status === 404) {
					return fail(404, { error: 'ไม่พบแอดมินที่ต้องการเปลี่ยนสถานะ' });
				} else if (response.status === 403) {
					return fail(403, { error: 'ไม่มีสิทธิ์ในการเปลี่ยนสถานะแอดมินนี้' });
				}
				return fail(response.status, { 
					error: result?.message || `เกิดข้อผิดพลาดในการ${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมิน` 
				});
			}

			if (result.status === 'success') {
				return { 
					success: true, 
					message: result.message || `${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมินสำเร็จ`,
					data: result.data
				};
			} else {
				return fail(400, { 
					error: result.message || `เกิดข้อผิดพลาดในการ${isActive ? 'เปิดใช้งาน' : 'ปิดใช้งาน'}แอดมิน` 
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

	update: async ({ request, cookies }) => {
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
			const sessionId = cookies.get('session_id');
			
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
			const response = await fetch(`${API_BASE_URL}/api/users/${targetUserId}`, {
				method: 'PUT',
				headers: {
					'Content-Type': 'application/json',
					'Cookie': `session_id=${sessionId}`
				},
				body: JSON.stringify(preparedData)
			});

			// ตรวจสอบว่า response มี content หรือไม่
			const contentType = response.headers.get('content-type');
			let result;
			
			if (contentType && contentType.includes('application/json')) {
				try {
					result = await response.json();
				} catch (parseError) {
					console.error('Response JSON parse error:', parseError);
					return fail(500, { error: 'เกิดข้อผิดพลาดในการประมวลผลข้อมูลจากเซิร์ฟเวอร์' });
				}
			} else {
				// ถ้าไม่มี JSON response
				const responseText = await response.text();
				if (!responseText.trim()) {
					return fail(500, { 
						error: 'เซิร์ฟเวอร์ตอบกลับข้อมูลไม่ครบถ้วน กรุณาลองใหม่อีกครั้ง' 
					});
				}
				
				result = {
					status: response.ok ? 'success' : 'error',
					message: response.ok ? 'User updated successfully' : responseText
				};
			}

			if (!response.ok) {
				// Handle specific error cases
				if (response.status === 404) {
					return fail(404, { error: 'ไม่พบแอดมินที่ต้องการอัพเดต' });
				} else if (response.status === 409) {
					return fail(409, { error: 'อีเมลนี้มีผู้ใช้แล้ว' });
				} else if (response.status === 403) {
					return fail(403, { error: 'ไม่มีสิทธิ์ในการแก้ไขข้อมูลแอดมินนี้' });
				}
				
				return fail(response.status, { 
					error: result.message || 'เกิดข้อผิดพลาดในการอัพเดตข้อมูลแอดมิน' 
				});
			}

			// ตรวจสอบ result structure
			if (result.status === 'success' || response.ok) {
				return { 
					success: true, 
					message: 'อัพเดตข้อมูลแอดมินสำเร็จ',
					data: result.data || result
				};
			} else {
				return fail(400, { 
					error: result.message || 'เกิดข้อผิดพลาดในการอัพเดตข้อมูลแอดมิน' 
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