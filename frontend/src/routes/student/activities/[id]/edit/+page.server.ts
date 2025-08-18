import type { PageServerLoad, Actions } from './$types';
import { error, fail, redirect } from '@sveltejs/kit';
import type { Activity, ActivityUpdateData } from '$lib/types/activity';
import { api } from '$lib/server/api-client';
import { convertStatusForBackend, convertStatusFromBackend } from '$lib/utils/activity';

export const load: PageServerLoad = async (event) => {
  const { params, depends } = event;
  depends('student:activity-edit');

  const { id } = params;

  if (!id) {
    throw error(404, 'Activity ID is required');
  }

  try {
    // Fetch activity details
    const activityRes = await api.get(event, `/api/activities/${id}`);
    if (!activityRes.success || !activityRes.data) {
      throw error(500, 'ข้อมูลกิจกรรมไม่ถูกต้อง');
    }
    const rawActivity = activityRes.data as any;
    const activity: Activity = {
      ...rawActivity,
      status: convertStatusFromBackend(rawActivity.status ?? 'Draft')
    };

    // Fetch faculties for dropdown
    let faculties: any[] = [];
    try {
      const facultiesRes = await api.get(event, '/api/faculties');
      if (facultiesRes.success) {
        faculties = facultiesRes.data || [];
      }
    } catch (e) {
      console.warn('Could not fetch faculties:', e);
    }

    return {
      activity,
      faculties
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
    const { params, request } = event;
    const { id } = params;
    
    if (!id) {
      return fail(400, { error: 'Activity ID is required' });
    }

    try {
      const formData = await request.formData();
      
      const statusValue = formData.get('status')?.toString();
      const updateData: ActivityUpdateData = {
        title: formData.get('title')?.toString(),
        description: formData.get('description')?.toString(),
        location: formData.get('location')?.toString(),
        start_time: formData.get('start_time')?.toString(),
        end_time: formData.get('end_time')?.toString(),
        max_participants: formData.get('max_participants') ? 
          parseInt(formData.get('max_participants')?.toString() || '0') : undefined,
        status: statusValue ? convertStatusForBackend(statusValue as any) as any : undefined,
        faculty_id: formData.get('faculty_id')?.toString() || undefined
      };

      // Remove undefined values
      Object.keys(updateData).forEach(key => {
        if (updateData[key as keyof ActivityUpdateData] === undefined) {
          delete updateData[key as keyof ActivityUpdateData];
        }
      });

      // Validate required fields
      if (!updateData.title?.trim()) {
        return fail(400, { 
          error: 'ชื่อกิจกรรมเป็นสิ่งจำเป็น',
          formData: Object.fromEntries(formData)
        });
      }

      if (!updateData.location?.trim()) {
        return fail(400, { 
          error: 'สถานที่เป็นสิ่งจำเป็น',
          formData: Object.fromEntries(formData)
        });
      }

      if (!updateData.start_time || !updateData.end_time) {
        return fail(400, { 
          error: 'วันที่และเวลาเป็นสิ่งจำเป็น',
          formData: Object.fromEntries(formData)
        });
      }

      // Validate time range
      if (new Date(updateData.start_time) >= new Date(updateData.end_time)) {
        return fail(400, { 
          error: 'เวลาเริ่มต้องน้อยกว่าเวลาสิ้นสุด',
          formData: Object.fromEntries(formData)
        });
      }

      // Update activity
      const response = await api.put(event, `/api/activities/${id}`, updateData);
      if (!response.success) {
        return fail(400, { 
          error: response.error || 'เกิดข้อผิดพลาดในการอัปเดตกิจกรรม',
          formData: Object.fromEntries(formData)
        });
      }

      // Redirect to activity details page
      throw redirect(302, `/student/activities/${id}`);

    } catch (e) {
      if (e instanceof Error && 'status' in e && (e as any).status === 302) {
        throw e; // Re-throw redirect
      }
      
      console.error('Error updating activity:', e);
      return fail(500, { 
        error: 'เกิดข้อผิดพลาดในการอัปเดตกิจกรรม',
        formData: Object.fromEntries(await request.formData())
      });
    }
  }
};
