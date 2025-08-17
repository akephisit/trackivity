import type { PageServerLoad, Actions } from './$types';
import { error, redirect } from '@sveltejs/kit';
import type { Activity, Participation, ActivityStatus } from '$lib/types/activity';

export const load: PageServerLoad = async ({ params, fetch, depends, locals }) => {
  depends('admin:activity-details');

  const { id } = params;

  if (!id) {
    throw error(404, 'Activity ID is required');
  }

  // Check admin authorization
  if (!locals.user || (locals.user as any).role !== 'admin') {
    throw redirect(302, '/admin/login');
  }

  try {
    // Fetch activity details with admin privileges
    const activityRes = await fetch(`/api/admin/activities/${id}`, {
      headers: {
        'Authorization': `Bearer ${(locals.user as any).token}`
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
    
    if (activityData.status !== 'success' || !activityData.data) {
      throw error(500, 'ข้อมูลกิจกรรมไม่ถูกต้อง');
    }

    const activity: Activity = activityData.data;

    // Fetch activity participations with admin privileges
    let participations: Participation[] = [];
    let participationStats = { total: 0, registered: 0, checked_in: 0, checked_out: 0, completed: 0 };
    
    try {
      const participationsRes = await fetch(`/api/admin/activities/${id}/participations`, {
        headers: {
          'Authorization': `Bearer ${(locals.user as any).token}`
        }
      });
      
      if (participationsRes.ok) {
        const participationsData = await participationsRes.json();
        if (participationsData.status === 'success' && participationsData.data?.participations) {
          participations = participationsData.data.participations;
          
          // Calculate participation statistics
          participationStats = {
            total: participations.length,
            registered: participations.filter(p => p.status === 'registered').length,
            checked_in: participations.filter(p => p.status === 'checked_in').length,
            checked_out: participations.filter(p => p.status === 'checked_out').length,
            completed: participations.filter(p => p.status === 'completed').length
          };
        }
      }
    } catch (e) {
      console.warn('Could not fetch participations:', e);
    }

    // Fetch faculties list for editing
    let faculties: any[] = [];
    try {
      const facultiesRes = await fetch('/api/admin/faculties', {
        headers: {
          'Authorization': `Bearer ${(locals.user as any).token}`
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

    return {
      activity,
      participations,
      participationStats,
      faculties,
      user: locals.user
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
  updateStatus: async ({ request, params, locals, fetch }) => {
    if (!locals.user || (locals.user as any).role !== 'admin') {
      throw error(403, 'ไม่มีสิทธิ์ในการดำเนินการนี้');
    }

    const { id } = params;
    const formData = await request.formData();
    const status = formData.get('status') as ActivityStatus;

    if (!status) {
      return {
        error: 'กรุณาระบุสถานะกิจกรรม'
      };
    }

    try {
      const response = await fetch(`/api/admin/activities/${id}`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${(locals.user as any).token}`
        },
        body: JSON.stringify({ status })
      });

      if (!response.ok) {
        const errorData = await response.json();
        return {
          error: errorData.message || 'ไม่สามารถอัปเดตสถานะกิจกรรมได้'
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
  updateParticipant: async ({ request, params, locals, fetch }) => {
    if (!locals.user || (locals.user as any).role !== 'admin') {
      throw error(403, 'ไม่มีสิทธิ์ในการดำเนินการนี้');
    }

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
      const response = await fetch(`/api/admin/activities/${id}/participations/${participationId}`, {
        method: 'PATCH',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${(locals.user as any).token}`
        },
        body: JSON.stringify({ status, notes })
      });

      if (!response.ok) {
        const errorData = await response.json();
        return {
          error: errorData.message || 'ไม่สามารถอัปเดตสถานะผู้เข้าร่วมได้'
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
  removeParticipant: async ({ request, params, locals, fetch }) => {
    if (!locals.user || (locals.user as any).role !== 'admin') {
      throw error(403, 'ไม่มีสิทธิ์ในการดำเนินการนี้');
    }

    const { id } = params;
    const formData = await request.formData();
    const participationId = formData.get('participationId') as string;

    if (!participationId) {
      return {
        error: 'ไม่พบรหัสผู้เข้าร่วม'
      };
    }

    try {
      const response = await fetch(`/api/admin/activities/${id}/participations/${participationId}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${(locals.user as any).token}`
        }
      });

      if (!response.ok) {
        const errorData = await response.json();
        return {
          error: errorData.message || 'ไม่สามารถลบผู้เข้าร่วมได้'
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
  deleteActivity: async ({ params, locals, fetch }) => {
    if (!locals.user || (locals.user as any).role !== 'admin') {
      throw error(403, 'ไม่มีสิทธิ์ในการดำเนินการนี้');
    }

    const { id } = params;

    try {
      const response = await fetch(`/api/admin/activities/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${(locals.user as any).token}`
        }
      });

      if (!response.ok) {
        const errorData = await response.json();
        return {
          error: errorData.message || 'ไม่สามารถลบกิจกรรมได้'
        };
      }

      throw redirect(302, '/admin/activities');
    } catch (e) {
      if (e instanceof Error && 'status' in e && e.status === 302) {
        throw e; // Re-throw redirect
      }
      
      console.error('Error deleting activity:', e);
      return {
        error: 'เกิดข้อผิดพลาดในการลบกิจกรรม'
      };
    }
  }
};