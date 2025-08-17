import { requireFacultyAdmin } from '$lib/server/auth';
import { error, fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import type { Activity } from '$lib/types/activity';
import { AdminLevel } from '$lib/types/admin';
import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
	// ตรวจสอบสิทธิ์ - เฉพาะ FacultyAdmin หรือ SuperAdmin
	const user = await requireFacultyAdmin(event);
	const adminLevel = user.admin_role?.admin_level;
	const facultyId = user.admin_role?.faculty_id;

	try {
		// กำหนด API endpoint ตามระดับแอดมิน
		let apiEndpoint: string;
		let params: Record<string, string> = {};
		
		if (adminLevel === AdminLevel.SuperAdmin) {
			// SuperAdmin ดูกิจกรรมทั้งหมด
			apiEndpoint = `/api/admin/activities`;
		} else if (adminLevel === AdminLevel.FacultyAdmin) {
			// FacultyAdmin ดูเฉพาะกิจกรรมในคณะของตัวเอง
			if (!facultyId) {
				throw error(403, 'Faculty admin ต้องมี faculty_id');
			}
			apiEndpoint = `/api/admin/activities`;
			params.faculty_id = facultyId;
		} else {
			throw error(403, 'ไม่มีสิทธิ์เข้าถึงข้อมูลกิจกรรม');
		}

		// เรียก API เพื่อดึงข้อมูลกิจกรรม
		const response = await api.get(event, apiEndpoint, Object.keys(params).length > 0 ? params : undefined);

		let activities: Activity[] = [];
		
    if (response.success) {
        const rawActivities = response.data?.activities || response.data || [];
			
			activities = rawActivities.map((activity: any) => ({
				id: activity.id,
				activity_name: activity.activity_name || activity.name || activity.title,
				description: activity.description,
				start_date: activity.start_date,
				end_date: activity.end_date,
				start_time: activity.start_time_only || activity.start_time,
				end_time: activity.end_time_only || activity.end_time,
				activity_type: activity.activity_type,
				location: activity.location,
				max_participants: activity.max_participants,
				hours: activity.hours,
				require_score: activity.require_score,
				faculty_id: activity.faculty_id,
				created_by: activity.created_by,
				created_at: activity.created_at,
				updated_at: activity.updated_at,
				// Legacy fields for compatibility
				name: activity.activity_name || activity.name || activity.title,
				organizer: activity.organizer || 'ระบบ',
				organizerType: activity.organizerType || 'คณะ',
				participantCount: activity.participant_count || 0,
				status: activity.status || 'รอดำเนินการ'
			}));
		} else {
			console.error('Failed to load activities:', response.error);
			// ไม่ throw error แต่ให้ส่งค่า array ว่างไป
		}

		return {
			activities,
			user,
			adminLevel,
			facultyId,
			canCreateActivity: adminLevel === AdminLevel.SuperAdmin || adminLevel === AdminLevel.FacultyAdmin
		};

	} catch (err) {
		console.error('Error loading activities:', err);
		
		// ถ้าเป็น error ที่ throw มาแล้ว ให้ส่งต่อไป
		if (err instanceof Error && 'status' in err) {
			throw err;
		}
		
		// สำหรับ error อื่นๆ ให้ส่งค่าเริ่มต้นแทนการ throw
		return {
			activities: [],
			user,
			adminLevel,
			facultyId,
			canCreateActivity: adminLevel === AdminLevel.SuperAdmin || adminLevel === AdminLevel.FacultyAdmin
		};
	}
};

export const actions: Actions = {
    delete: async (event) => {
        // Ensure only FacultyAdmin/SuperAdmin can delete
        await requireFacultyAdmin(event);

        try {
            const formData = await event.request.formData();
            const activityId = formData.get('activityId');
            if (!activityId || typeof activityId !== 'string') {
                return fail(400, { error: 'ไม่พบรหัสกิจกรรม' });
            }

            const response = await api.delete(event, `/api/activities/${activityId}`);

            if (!response.success) {
                return fail(500, { error: response.error || 'ลบกิจกรรมไม่สำเร็จ' });
            }

            // Success
            return { success: true };
        } catch (err) {
            console.error('Delete activity error:', err);
            return fail(500, { error: 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์' });
        }
    }
};
