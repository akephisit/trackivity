import type { PageServerLoad, Actions } from './$types';
import { error, redirect } from '@sveltejs/kit';
import type { Activity, Participation, ActivityStatus } from '$lib/types/activity';
import { requireFacultyAdmin } from '$lib/server/auth';
import { api } from '$lib/server/api-client';
import { convertStatusForBackend, convertStatusFromBackend } from '$lib/utils/activity';

export const load: PageServerLoad = async (event) => {
  const { params, fetch, depends } = event;
  depends('admin:activity-details');

  const { id } = params;

  if (!id) {
    throw error(404, 'Activity ID is required');
  }

  // Check admin authorization (FacultyAdmin or SuperAdmin)
  const user = await requireFacultyAdmin(event);

  try {
    // Try admin endpoint first; if 404, fallback to public endpoint
    let activityResponse = await api.get(event, `/api/admin/activities/${id}`);
    
    if (!activityResponse.success) {
      // Try fallback to public endpoint
      activityResponse = await api.get(event, `/api/activities/${id}`);
    }
    
    // Extract activity data from response
    const rawActivity = activityResponse.data as any;
    if (!rawActivity) {
      throw error(500, 'ข้อมูลกิจกรรมไม่ถูกต้อง');
    }

    // Build ISO start/end if only date+time provided
    const startIso = rawActivity.start_time ?? (rawActivity.start_date && rawActivity.start_time_only ? new Date(`${rawActivity.start_date}T${rawActivity.start_time_only}`).toISOString() : undefined);
    const endIso = rawActivity.end_time ?? (rawActivity.end_date && rawActivity.end_time_only ? new Date(`${rawActivity.end_date}T${rawActivity.end_time_only}`).toISOString() : undefined);

    // Map to Activity type with extended fields
    const activity: Activity = {
      id: rawActivity.id,
      title: rawActivity.title ?? rawActivity.activity_name ?? rawActivity.name,
      description: rawActivity.description ?? '',
      location: rawActivity.location ?? '',
      start_time: startIso ?? rawActivity.start_date,
      end_time: endIso ?? rawActivity.end_date,
      max_participants: rawActivity.max_participants ?? undefined,
      current_participants: rawActivity.current_participants ?? 0,
      status: convertStatusFromBackend(rawActivity.status ?? 'Draft'),
      faculty_id: rawActivity.faculty_id ?? undefined,
      faculty_name: rawActivity.faculty_name ?? undefined,
      created_by: rawActivity.created_by,
      created_by_name: rawActivity.created_by_name ?? 'ระบบ',
      created_at: rawActivity.created_at,
      updated_at: rawActivity.updated_at,
      is_registered: rawActivity.is_registered ?? false,
      user_participation_status: rawActivity.user_participation_status ?? undefined,
      // Extended admin fields (optional)
      activity_type: rawActivity.activity_type ?? undefined,
      hours: rawActivity.hours ?? undefined,
      organizer: rawActivity.organizer ?? undefined,
      academic_year: rawActivity.academic_year ?? undefined,
      start_date: rawActivity.start_date ?? undefined,
      end_date: rawActivity.end_date ?? undefined,
      start_time_only: rawActivity.start_time_only ?? undefined,
      end_time_only: rawActivity.end_time_only ?? undefined
    };

    // Fetch activity participations with admin privileges
    let participations: Participation[] = [];
    let participationStats = { total: 0, registered: 0, checked_in: 0, checked_out: 0, completed: 0 };
    
    try {
      const participationsResponse = await api.get(event, `/api/activities/${id}/participations`);
      
      if (participationsResponse.success && participationsResponse.data?.participations) {
        participations = participationsResponse.data.participations;
        
        // Calculate participation statistics
        participationStats = {
          total: participations.length,
          registered: participations.filter(p => p.status === 'registered').length,
          checked_in: participations.filter(p => p.status === 'checked_in').length,
          checked_out: participations.filter(p => p.status === 'checked_out').length,
          completed: participations.filter(p => p.status === 'completed').length
        };
      }
    } catch (e) {
      console.warn('Could not fetch participations:', e);
    }

    // Fetch faculties list for editing
    let faculties: any[] = [];
    try {
      const facultiesResponse = await api.get(event, `/api/admin/faculties`);
      
      if (facultiesResponse.success && facultiesResponse.data) {
        faculties = facultiesResponse.data;
      }
    } catch (e) {
      console.warn('Could not fetch faculties:', e);
    }

    return {
      activity,
      participations,
      participationStats,
      faculties,
      user
    };
  } catch (e) {
    if (e instanceof Error && 'status' in e) {
      throw e; // Re-throw SvelteKit errors
    }
    
    console.error('Error loading activity details:', e);
    throw error(500, 'เกิดข้อผิดพลาดในการโหลดข้อมูล');
  }
};

export const actions: Actions = {
  // Update activity status
  updateStatus: async (event) => {
    const { request, params } = event;
    await requireFacultyAdmin(event);

    const { id } = params;
    const formData = await request.formData();
    const status = formData.get('status') as ActivityStatus;

    if (!status) {
      return {
        error: 'กรุณาระบุสถานะกิจกรรม'
      };
    }

    try {
      const response = await api.put(event, `/api/activities/${id}`, { 
        status: convertStatusForBackend(status)
      });

      if (!response.success) {
        return {
          error: response.error || 'ไม่สามารถอัปเดตสถานะกิจกรรมได้'
        };
      }

      return {
        success: true,
        message: 'อัปเดตสถานะกิจกรรมสำเร็จ'
      };
    } catch (e) {
      console.error('Error updating activity status:', e);
      return {
        error: 'เกิดข้อผิดพลาดในการอัปเดตสถานะ'
      };
    }
  },

  // Update participant status
  updateParticipant: async (event) => {
    const { request, params } = event;
    await requireFacultyAdmin(event);

    const { id } = params;
    const formData = await request.formData();
    const participationId = formData.get('participationId') as string;
    const status = formData.get('participantStatus') as string;
    const notes = formData.get('notes') as string;

    if (!participationId || !status) {
      return {
        error: 'ข้อมูลไม่ครบถ้วน'
      };
    }

    try {
      const response = await api.patch(event, `/api/admin/activities/${id}/participations/${participationId}`, {
        status,
        notes
      });

      if (!response.success) {
        return {
          error: response.error || 'ไม่สามารถอัปเดตสถานะผู้เข้าร่วมได้'
        };
      }

      return {
        success: true,
        message: 'อัปเดตสถานะผู้เข้าร่วมสำเร็จ'
      };
    } catch (e) {
      console.error('Error updating participant status:', e);
      return {
        error: 'เกิดข้อผิดพลาดในการอัปเดตสถานะผู้เข้าร่วม'
      };
    }
  },

  // Remove participant
  removeParticipant: async (event) => {
    const { request, params } = event;
    await requireFacultyAdmin(event);

    const { id } = params;
    const formData = await request.formData();
    const participationId = formData.get('participationId') as string;

    if (!participationId) {
      return {
        error: 'ไม่พบรหัสผู้เข้าร่วม'
      };
    }

    try {
      const response = await api.delete(event, `/api/admin/activities/${id}/participations/${participationId}`);

      if (!response.success) {
        return {
          error: response.error || 'ไม่สามารถลบผู้เข้าร่วมได้'
        };
      }

      return {
        success: true,
        message: 'ลบผู้เข้าร่วมสำเร็จ'
      };
    } catch (e) {
      console.error('Error removing participant:', e);
      return {
        error: 'เกิดข้อผิดพลาดในการลบผู้เข้าร่วม'
      };
    }
  },

  // Delete activity
  deleteActivity: async (event) => {
    const { params } = event;
    await requireFacultyAdmin(event);

    const { id } = params;

    try {
      const response = await api.delete(event, `/api/activities/${id}`);

      if (!response.success) {
        return {
          error: response.error || 'ไม่สามารถลบกิจกรรมได้'
        };
      }

      throw redirect(302, '/admin/activities');
    } catch (e) {
      if (typeof e === 'object' && e && 'status' in (e as any) && (e as any).status === 302) {
        throw e as any; // Re-throw redirect for SvelteKit to handle
      }
      
      console.error('Error deleting activity:', e);
      return {
        error: 'เกิดข้อผิดพลาดในการลบกิจกรรม'
      };
    }
  }
};
