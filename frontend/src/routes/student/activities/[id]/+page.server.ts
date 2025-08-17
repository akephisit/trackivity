import type { PageServerLoad } from './$types';
import { error } from '@sveltejs/kit';
import type { Activity, Participation } from '$lib/types/activity';

export const load: PageServerLoad = async ({ params, fetch, depends }) => {
  depends('student:activity-details');

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
      throw error(500, 'ไม่สามารถโหลดข้อมูลกิจกรรมได้');
    }

    const activityData = await activityRes.json();
    
    if (activityData.status !== 'success' || !activityData.data) {
      throw error(500, 'ข้อมูลกิจกรรมไม่ถูกต้อง');
    }

    const activity: Activity = activityData.data;

    // Fetch activity participations (only if user has permission)
    let participations: Participation[] = [];
    try {
      const participationsRes = await fetch(`/api/activities/${id}/participations`);
      if (participationsRes.ok) {
        const participationsData = await participationsRes.json();
        if (participationsData.status === 'success' && participationsData.data?.participations) {
          participations = participationsData.data.participations;
        }
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