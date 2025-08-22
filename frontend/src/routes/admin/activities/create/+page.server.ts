import { requireFacultyAdmin } from '$lib/server/auth';
import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';
import type { ActivityCreateData } from '$lib/types/activity';

// Validation schema for activity creation
const activityCreateSchema = z.object({
	activity_name: z.string().min(1, 'กรุณากรอกชื่อกิจกรรม').max(255, 'ชื่อกิจกรรมต้องไม่เกิน 255 ตัวอักษร'),
	description: z.string().max(2000, 'รายละเอียดต้องไม่เกิน 2000 ตัวอักษร').optional().or(z.literal('')),
	start_date: z.string().min(1, 'กรุณาเลือกวันที่เริ่ม').refine((date) => {
		const d = new Date(date);
		return !isNaN(d.getTime());
	}, 'วันที่เริ่มไม่ถูกต้อง'),
	end_date: z.string().min(1, 'กรุณาเลือกวันที่สิ้นสุด').refine((date) => {
		const d = new Date(date);
		return !isNaN(d.getTime());
	}, 'วันที่สิ้นสุดไม่ถูกต้อง'),
	start_time: z.string().min(1, 'กรุณากรอกเวลาเริ่ม').regex(/^([01]?[0-9]|2[0-3]):[0-5][0-9]$/, 'รูปแบบเวลาไม่ถูกต้อง (ต้องเป็น HH:MM)'),
	end_time: z.string().min(1, 'กรุณากรอกเวลาสิ้นสุด').regex(/^([01]?[0-9]|2[0-3]):[0-5][0-9]$/, 'รูปแบบเวลาไม่ถูกต้อง (ต้องเป็น HH:MM)'),
	activity_type: z.enum(['Academic', 'Sports', 'Cultural', 'Social', 'Other'], {
		errorMap: () => ({ message: 'กรุณาเลือกประเภทกิจกรรม' })
	}),
	location: z.string().min(1, 'กรุณากรอกสถานที่').max(500, 'สถานที่ต้องไม่เกิน 500 ตัวอักษร'),
	max_participants: z.string().optional(),
	hours: z
		.string()
		.min(1, 'กรุณากรอกจำนวนชั่วโมง')
		.regex(/^\d+$/, 'ชั่วโมงต้องเป็นจำนวนเต็ม')
		.refine((v) => parseInt(v) > 0, 'ชั่วโมงต้องมากกว่า 0'),
	organizer: z.string().min(1, 'กรุณากรอกหน่วยงานที่จัดกิจกรรม').max(255, 'ชื่อหน่วยงานต้องไม่เกิน 255 ตัวอักษร'),
	eligible_faculties: z.string().min(1, 'กรุณาเลือกคณะที่สามารถเข้าร่วมได้').refine(value => {
		const faculties = value.split(',').filter(f => f.trim() !== '');
		return faculties.length > 0;
	}, 'กรุณาเลือกอย่างน้อย 1 คณะ'),
	academic_year: z.string().min(1, 'กรุณาเลือกปีการศึกษา')
}).refine(data => {
	const startDate = new Date(data.start_date);
	const endDate = new Date(data.end_date);
	return endDate >= startDate;
}, {
	message: 'วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น',
	path: ['end_date']
}).refine(data => {
	// If same date, check that end time is after start time
	if (data.start_date === data.end_date) {
		const [startHour, startMin] = data.start_time.split(':').map(Number);
		const [endHour, endMin] = data.end_time.split(':').map(Number);
		const startMinutes = startHour * 60 + startMin;
		const endMinutes = endHour * 60 + endMin;
		return endMinutes > startMinutes;
	}
	return true;
}, {
	message: 'เวลาสิ้นสุดต้องมากกว่าเวลาเริ่มต้น',
	path: ['end_time']
});

export const load: PageServerLoad = async (event) => {
	// ตรวจสอบสิทธิ์ - เฉพาะ FacultyAdmin หรือ SuperAdmin
	const user = await requireFacultyAdmin(event);
	
	// สร้าง empty form
	const form = await superValidate(zod(activityCreateSchema));
	
	// Set default values
	form.data = {
		activity_name: '',
		description: '',
		start_date: '',
		end_date: '',
		start_time: '09:00',
		end_time: '17:00',
		activity_type: 'Academic',
		location: '',
		max_participants: '',
		hours: '1',
		organizer: '',
		eligible_faculties: '',
		academic_year: ''
	};

	// ดึงข้อมูลคณะจากฐานข้อมูล
	let faculties: any[] = [];
	try {
		const response = await event.fetch('/api/faculties');
		if (response.ok) {
			const apiData = await response.json();
			
			// แก้ไขการ parse ให้ถูกต้อง - ตาม log structure: { data: { faculties: [...] } }
			if (apiData.data && apiData.data.faculties && Array.isArray(apiData.data.faculties)) {
				faculties = apiData.data.faculties;
			} else if (apiData.data && Array.isArray(apiData.data)) {
				faculties = apiData.data;
			} else if (Array.isArray(apiData)) {
				faculties = apiData;
			} else if (apiData.faculties && Array.isArray(apiData.faculties)) {
				faculties = apiData.faculties;
			} else {
				faculties = [];
			}
		} else {
			console.error('Faculties API error:', response.status, response.statusText);
		}
	} catch (error) {
		console.error('Failed to fetch faculties:', error);
		// ใช้ default faculties หากไม่สามารถดึงข้อมูลได้
		faculties = [];
	}

	return {
		form,
		user,
		admin_role: user.admin_role,
		faculties
	};
};

export const actions: Actions = {
	default: async (event) => {
		// ตรวจสอบสิทธิ์อีกครั้ง
		await requireFacultyAdmin(event);
		
		// Validate form data
		const form = await superValidate(event, zod(activityCreateSchema));
		
		if (!form.valid) {
			return fail(400, {
				form,
				error: 'ข้อมูลไม่ถูกต้อง กรุณาตรวจสอบอีกครั้ง'
			});
		}

		try {
			// แปลง eligible_faculties จาก string เป็น array of UUIDs
			const eligibleFacultiesArray = form.data.eligible_faculties 
				? form.data.eligible_faculties.split(',').filter(f => f.trim() !== '') 
				: [];

			// เตรียมข้อมูลสำหรับส่งไป API
			const activityData: any = {
				activity_name: form.data.activity_name,
				description: form.data.description && form.data.description.trim() !== '' ? form.data.description : '',
				start_date: form.data.start_date,
				end_date: form.data.end_date,
				start_time: form.data.start_time,
				end_time: form.data.end_time,
				activity_type: form.data.activity_type,
				location: form.data.location,
				max_participants: form.data.max_participants ? parseInt(form.data.max_participants) : null,
				hours: form.data.hours ? parseInt(form.data.hours) : 1,
				organizer: form.data.organizer,
				eligible_faculties: eligibleFacultiesArray,
				academic_year: form.data.academic_year
			};

			// เรียก API ผ่าน internal route
			const response = await event.fetch('/api/admin/activities', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(activityData)
			});

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({}));
				const errorMessage = errorData.error || errorData.message || 'เกิดข้อผิดพลาดในการสร้างกิจกรรม';
				
				return fail(response.status, {
					form,
					error: errorMessage
				});
			}

			// อ่าน response data (ignore content if not JSON)
			try { await response.json(); } catch {}

			// ส่ง success ให้ client จัดการ toast และนำทางเอง (ไม่ใช้ query param)
			return { form, success: true } as const;

		} catch (error) {
			// ส่งต่อ redirect object ของ SvelteKit โดยไม่ log เป็น error
			if (error && typeof error === 'object' && 'status' in error && 'location' in error) {
				throw error as any;
			}

			console.error('Error creating activity:', error);
			return fail(500, { form, error: 'เกิดข้อผิดพลาดภายในเซิร์ฟเวอร์' });
		}
	}
};
