import type { PageServerLoad, Actions } from './$types';
import { error, redirect } from '@sveltejs/kit';
import type { Activity, ActivityUpdateData } from '$lib/types/activity';
import { requireFacultyAdmin } from '$lib/server/auth';
import { PUBLIC_API_URL } from '$env/static/public';

export const load: PageServerLoad = async (event) => {
  const { params, fetch, depends } = event;
  depends('admin:activity-edit');

  const { id } = params;

  if (!id) {
    throw error(404, 'Activity ID is required');
  }

  // Check admin authorization (FacultyAdmin or SuperAdmin)
  const user = await requireFacultyAdmin(event);
  const sessionId = event.cookies.get('session_id');
  if (!sessionId) {
    throw error(401, 'ไม่มีการ authentication');
  }

  try {
    // Fetch activity details from public activities endpoint
    const activityRes = await fetch(`${PUBLIC_API_URL}/api/activities/${id}`, {
      headers: {
        'Cookie': `session_id=${sessionId}`,
        'X-Session-ID': sessionId
      }
    });
    
    if (!activityRes.ok) {
      if (activityRes.status === 404) {
        throw error(404, 'ไม่พบกิจกรรมที่ระบุ');
      }
      if (activityRes.status === 403) {
        throw error(403, 'ไม่มีสิทธิ์เข้าถึงข้อมูลนี้');
      }
      throw error(500, 'ไม่สามารถโหลดข้อมูลกิจกรรมได้');
    }

    const activityData = await activityRes.json();

    const rawActivity = activityData?.data ?? activityData;
    if (!rawActivity) {
      throw error(500, 'ข้อมูลกิจกรรมไม่ถูกต้อง');
    }

    const activity: Activity = {
      id: rawActivity.id,
      title: rawActivity.title ?? rawActivity.activity_name ?? rawActivity.name,
      description: rawActivity.description ?? '',
      location: rawActivity.location ?? '',
      start_time: rawActivity.start_time ?? rawActivity.start_date,
      end_time: rawActivity.end_time ?? rawActivity.end_date,
      max_participants: rawActivity.max_participants ?? undefined,
      current_participants: rawActivity.current_participants ?? 0,
      status: rawActivity.status ?? 'draft',
      faculty_id: rawActivity.faculty_id ?? undefined,
      faculty_name: rawActivity.faculty_name ?? undefined,
      created_by: rawActivity.created_by,
      created_by_name: rawActivity.created_by_name ?? 'ระบบ',
      created_at: rawActivity.created_at,
      updated_at: rawActivity.updated_at,
      is_registered: rawActivity.is_registered ?? false,
      user_participation_status: rawActivity.user_participation_status ?? undefined
    };

    // Fetch faculties list
    let faculties: any[] = [];
    try {
      const facultiesRes = await fetch(`${PUBLIC_API_URL}/api/admin/faculties`, {
        headers: {
          'Cookie': `session_id=${sessionId}`,
          'X-Session-ID': sessionId
        }
      });
      
      if (facultiesRes.ok) {
        const facultiesData = await facultiesRes.json();
        if (facultiesData.status === 'success' && facultiesData.data) {
          faculties = facultiesData.data;
        }
      }
    } catch (e) {
      console.warn('Could not fetch faculties:', e);
    }

    // Fetch departments list
    let departments: any[] = [];
    try {
      const departmentsRes = await fetch(`${PUBLIC_API_URL}/api/admin/departments`, {
        headers: {
          'Cookie': `session_id=${sessionId}`,
          'X-Session-ID': sessionId
        }
      });
      
      if (departmentsRes.ok) {
        const departmentsData = await departmentsRes.json();
        if (departmentsData.status === 'success' && departmentsData.data) {
          departments = departmentsData.data;
        }
      }
    } catch (e) {
      console.warn('Could not fetch departments:', e);
    }

    return {
      activity,
      faculties,
      departments,
      user
    };
  } catch (e) {
    if (e instanceof Error && 'status' in e) {
      throw e; // Re-throw SvelteKit errors
    }
    
    console.error('Error loading activity for edit:', e);
    throw error(500, 'เกิดข้อผิดพลาดในการโหลดข้อมูล');
  }
};

export const actions: Actions = {
  update: async (event) => {
    const { request, params, fetch } = event;
    await requireFacultyAdmin(event);
    const sessionId = event.cookies.get('session_id');
    if (!sessionId) {
      throw error(401, 'ไม่มีการ authentication');
    }

    const { id } = params;
    const formData = await request.formData();

    // Extract and validate form data
    const title = formData.get('title') as string;
    const description = formData.get('description') as string;
    const location = formData.get('location') as string;
    const start_time = formData.get('start_time') as string;
    const end_time = formData.get('end_time') as string;
    const max_participants = formData.get('max_participants') as string;
    const status = formData.get('status') as string;
    const faculty_id = formData.get('faculty_id') as string;
    const department_id = formData.get('department_id') as string;

    // Validation
    if (!title || !location || !start_time || !end_time) {
      return {
        error: 'กรุณากรอกข้อมูลที่จำเป็นให้ครบถ้วน',
        formData: Object.fromEntries(formData)
      };
    }

    // Validate dates
    const startDate = new Date(start_time);
    const endDate = new Date(end_time);
    
    if (startDate >= endDate) {
      return {
        error: 'วันที่และเวลาสิ้นสุดต้องหลังจากวันที่และเวลาเริ่มต้น',
        formData: Object.fromEntries(formData)
      };
    }

    // Prepare update data
    const updateData: ActivityUpdateData = {
      title,
      description: description || undefined,
      location,
      start_time,
      end_time,
      status: status as any,
      faculty_id: faculty_id || undefined,
      department_id: department_id || undefined
    };

    // Add max_participants if provided
    if (max_participants && max_participants.trim() !== '') {
      const maxParticipantsNum = parseInt(max_participants);
      if (isNaN(maxParticipantsNum) || maxParticipantsNum < 1) {
        return {
          error: 'จำนวนผู้เข้าร่วมสูงสุดต้องเป็นตัวเลขที่มากกว่า 0',
          formData: Object.fromEntries(formData)
        };
      }
      updateData.max_participants = maxParticipantsNum;
    } else {
      updateData.max_participants = undefined;
    }

    try {
      const response = await fetch(`${PUBLIC_API_URL}/api/activities/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Cookie': `session_id=${sessionId}`,
          'X-Session-ID': sessionId
        },
        body: JSON.stringify(updateData)
      });

      if (!response.ok) {
        const errorData = await response.json();
        return {
          error: errorData.message || 'ไม่สามารถแก้ไขกิจกรรมได้',
          formData: Object.fromEntries(formData)
        };
      }

      const result = await response.json();
      
      if (result.status === 'success') {
        // Redirect to activity detail page
        throw redirect(302, `/admin/activities/${id}`);
      } else {
        return {
          error: result.message || 'ไม่สามารถแก้ไขกิจกรรมได้',
          formData: Object.fromEntries(formData)
        };
      }
    } catch (e) {
      if (e instanceof Error && 'status' in e && e.status === 302) {
        throw e; // Re-throw redirect
      }
      
      console.error('Error updating activity:', e);
      return {
        error: 'เกิดข้อผิดพลาดในการแก้ไขกิจกรรม',
        formData: Object.fromEntries(formData)
      };
    }
  }
};
