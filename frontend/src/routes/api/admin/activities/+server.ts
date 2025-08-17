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
			'academic_year',
			'hours'
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

		// Validate hours
		const hours = parseInt(body.hours);
		if (isNaN(hours) || hours <= 0) {
			throw error(400, 'ชั่วโมงกิจกรรมต้องเป็นจำนวนเต็มมากกว่า 0');
		}

		// สร้าง payload ให้ตรงกับ backend /api/admin/activities
		const payload = {
			activity_name: body.activity_name.trim(),
			description: body.description ? String(body.description).trim() : '',
			start_date: body.start_date,
			end_date: body.end_date,
			start_time: body.start_time,
			end_time: body.end_time,
			// include time-only aliases for backend compatibility
			start_time_only: body.start_time,
			end_time_only: body.end_time,
			activity_type: body.activity_type,
			location: body.location.trim(),
			max_participants: body.max_participants ?? null,
			organizer: body.organizer.trim(),
			eligible_faculties: String(body.eligible_faculties)
				.split(',')
				.map((s) => s.trim())
				.filter((s) => s.length > 0),
			academic_year: body.academic_year,
			hours
		};

		// เรียก backend API เพื่อสร้างกิจกรรม (admin endpoint)
		const response = await fetch(`${PUBLIC_API_URL}/api/admin/activities`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
				'Cookie': `session_id=${sessionId}`,
				'X-Session-ID': sessionId
			},
			body: JSON.stringify(payload)
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
			} else if (response.status === 422) {
				throw error(422, errorMessage);
			} else {
				throw error(response.status, errorMessage);
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
