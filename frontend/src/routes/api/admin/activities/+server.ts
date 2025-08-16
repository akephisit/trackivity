import { requireFacultyAdmin } from '$lib/server/auth';
import { json, error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import type { ActivityApiResponse } from '$lib/types/activity';
import { PUBLIC_API_URL } from '$env/static/public';

/**
 * POST /api/admin/activities
 * สร้างกิจกรรมใหม่โดย Faculty Admin หรือ Super Admin
 */
export const POST: RequestHandler = async (event) => {
	try {
		// ตรวจสอบการ authentication - เฉพาะ FacultyAdmin หรือ SuperAdmin
		await requireFacultyAdmin(event);
		const sessionId = event.cookies.get('session_id');

		if (!sessionId) {
			throw error(401, 'ไม่มีการ authentication');
		}

		// รับข้อมูลจาก request body
		const body = await event.request.json();
		
		// Validate required fields
		const requiredFields = [
			'activity_name',
			'start_date',
			'end_date',
			'start_time',
			'end_time',
			'activity_type',
			'location',
			'organizer',
			'eligible_faculties',
			'academic_year'
		];

		for (const field of requiredFields) {
			if (!body[field] || (typeof body[field] === 'string' && body[field].trim() === '')) {
				throw error(400, `กรุณากรอก ${field}`);
			}
		}

		// Validate activity type
		const validActivityTypes = ['Academic', 'Sports', 'Cultural', 'Social', 'Other'];
		if (!validActivityTypes.includes(body.activity_type)) {
			throw error(400, 'ประเภทกิจกรรมไม่ถูกต้อง');
		}

		// Validate dates and times
		const startDate = new Date(body.start_date);
		const endDate = new Date(body.end_date);
		
		if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) {
			throw error(400, 'วันที่ไม่ถูกต้อง');
		}

		if (endDate < startDate) {
			throw error(400, 'วันที่สิ้นสุดต้องไม่น้อยกว่าวันที่เริ่มต้น');
		}

		// Validate time format (HH:MM)
		const timeRegex = /^([01]?[0-9]|2[0-3]):[0-5][0-9]$/;
		if (!timeRegex.test(body.start_time) || !timeRegex.test(body.end_time)) {
			throw error(400, 'รูปแบบเวลาไม่ถูกต้อง (ต้องเป็น HH:MM)');
		}

		// Validate max_participants if provided
		if (body.max_participants !== undefined && body.max_participants !== null) {
			const maxParticipants = parseInt(body.max_participants);
			if (isNaN(maxParticipants) || maxParticipants < 1) {
				throw error(400, 'จำนวนผู้เข้าร่วมสูงสุดต้องเป็นตัวเลขที่มากกว่า 0');
			}
			body.max_participants = maxParticipants;
		}

		// แปลงข้อมูลให้ตรงกับ backend CreateActivityRequest
		const startDateTime = new Date(`${body.start_date}T${body.start_time}:00`);
		const endDateTime = new Date(`${body.end_date}T${body.end_time}:00`);
		
		const activityData: any = {
			title: body.activity_name.trim(),
			description: body.description ? body.description.trim() : "",
			location: body.location.trim(),
			start_time: startDateTime.toISOString(),
			end_time: endDateTime.toISOString(),
			max_participants: body.max_participants || null,
			faculty_id: null, // จะต้องใส่ faculty_id ที่ถูกต้อง
			department_id: null
		};

		// เรียก backend API เพื่อสร้างกิจกรรม
		const response = await fetch(`${PUBLIC_API_URL}/api/activities`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				'Cookie': `session_id=${sessionId}`,
				'X-Session-ID': sessionId
			},
			body: JSON.stringify(activityData)
		});

		if (!response.ok) {
			const errorData = await response.json().catch(() => ({}));
			const errorMessage = (errorData as any).error || (errorData as any).message || 'เกิดข้อผิดพลาดในการสร้างกิจกรรม';
			
			if (response.status === 401) {
				throw error(401, 'ไม่มีสิทธิ์ในการสร้างกิจกรรม');
			} else if (response.status === 403) {
				throw error(403, 'ไม่ได้รับอนุญาตให้สร้างกิจกรรม');
			} else if (response.status === 400) {
				throw error(400, errorMessage);
			} else {
				throw error(500, errorMessage);
			}
		}

		const result = await response.json();
		
		return json({
			success: true,
			data: result.data || result,
			message: 'สร้างกิจกรรมสำเร็จ'
		} as ActivityApiResponse);

	} catch (err) {
		console.error('Error creating activity:', err);
		
		// ถ้าเป็น error ที่ throw มาแล้ว ให้ส่งต่อไป
		if (err instanceof Error && 'status' in err) {
			throw err;
		}
		
		// สำหรับ error อื่นๆ
		throw error(500, 'เกิดข้อผิดพลาดภายในเซิร์ฟเวอร์');
	}
};