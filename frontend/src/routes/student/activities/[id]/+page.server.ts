import type { PageServerLoad } from './$types';
import { error } from '@sveltejs/kit';
import type { Activity, Participation } from '$lib/types/activity';

import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
  const { params, depends } = event;
  depends('student:activity-details');

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
    const activity: Activity = activityRes.data as any;

    // Fetch activity participations (only if user has permission)
    let participations: Participation[] = [];
    try {
      const participationsRes = await api.get(event, `/api/activities/${id}/participations`);
      if (participationsRes.success && participationsRes.data?.participations) {
        participations = participationsRes.data.participations;
      }
    } catch (e) {
      // Ignore participation fetch errors - user might not have permission
      console.warn('Could not fetch participations:', e);
    }

    return {
      activity,
      participations
    };
  } catch (e) {
    if (e instanceof Error && 'status' in e) {
      throw e; // Re-throw SvelteKit errors
    }
    
    console.error('Error loading activity details:', e);
    throw error(500, 'เกิดข้อผิดพลาดในการโหลดข้อมูล');
  }
};
