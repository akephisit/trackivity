import type { PageServerLoad, Actions } from './$types';
import { error, fail, redirect } from '@sveltejs/kit';
import type { Activity, ActivityUpdateData } from '$lib/types/activity';

export const load: PageServerLoad = async ({ params, fetch, depends }) => {
  depends('student:activity-edit');

  const { id } = params;

  if (!id) {
    throw error(404, 'Activity ID is required');
  }

  try {
    // Fetch activity details
    const activityRes = await fetch(`/api/activities/${id}`);
    
    if (!activityRes.ok) {
      if (activityRes.status === 404) {
        throw error(404, 'กิจกรรมไม่พบ');
      }
      if (activityRes.status === 403) {
        throw error(403, 'คุณไม่มีสิทธิ์แก้ไขกิจกรรมนี้');
      }
      throw error(500, 'ไม่สามารถโหลดข้อมูลกิจกรรมได้');
    }

    const activityData = await activityRes.json();
    
    if (activityData.status !== 'success' || !activityData.data) {
      throw error(500, 'ข้อมูลกิจกรรมไม่ถูกต้อง');
    }

    const activity: Activity = activityData.data;

    // Fetch faculties for dropdown
    let faculties: any[] = [];
    try {
      const facultiesRes = await fetch('/api/faculties');
      if (facultiesRes.ok) {
        const facultiesData = await facultiesRes.json();
        faculties = facultiesData.data || facultiesData || [];
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
  update: async ({ params, request, fetch }) => {
    const { id } = params;
    
    if (!id) {
      return fail(400, { error: 'Activity ID is required' });
    }

    try {
      const formData = await request.formData();
      
      const updateData: ActivityUpdateData = {
        title: formData.get('title')?.toString(),
        description: formData.get('description')?.toString(),
        location: formData.get('location')?.toString(),
        start_time: formData.get('start_time')?.toString(),
        end_time: formData.get('end_time')?.toString(),
        max_participants: formData.get('max_participants') ? 
          parseInt(formData.get('max_participants')?.toString() || '0') : undefined,
        status: formData.get('status')?.toString() as any,
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
      const response = await fetch(`/api/activities/${id}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(updateData)
      });

      if (!response.ok) {
        const errorData = await response.json();
        return fail(response.status, { 
          error: errorData.message || 'เกิดข้อผิดพลาดในการอัปเดตกิจกรรม',
          formData: Object.fromEntries(formData)
        });
      }

      const result = await response.json();
      
      if (result.status !== 'success') {
        return fail(400, { 
          error: result.message || 'เกิดข้อผิดพลาดในการอัปเดตกิจกรรม',
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