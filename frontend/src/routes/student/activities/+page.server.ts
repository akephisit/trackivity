import type { PageServerLoad } from './$types';
import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
  event.depends('student:activities');

  try {
    const params = { per_page: '50' };
    const response = await api.get(event, '/api/activities', params);
    
    if (response.status === 'error') {
      return { activities: [] };
    }

    // Support both { data: [...] } and raw array
    const activities = response.data || [];
    return { activities };
  } catch (e) {
    return { activities: [] };
  }
};

