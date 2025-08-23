import type { PageServerLoad } from './$types';
import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
  event.depends('student:activities');

  try {
    const params = { limit: '50' };
    const response = await api.get(event, '/api/activities', params);
    
    if (!response.success) {
      console.warn('Activities API returned error:', response.error);
      return { activities: [] };
    }

    // Backend returns nested structure: { status: "success", data: { activities: [...] } }
    const activities = response.data?.activities || response.data || [];
    return { activities };
  } catch (e) {
    console.error('Error loading activities:', e);
    return { activities: [] };
  }
};

