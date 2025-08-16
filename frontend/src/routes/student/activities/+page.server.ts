import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, depends }) => {
  depends('student:activities');

  try {
    const params = new URLSearchParams({ per_page: '50' });
    const res = await fetch(`/api/activities?${params.toString()}`);
    if (!res.ok) {
      return { activities: [] };
    }

    const payload = await res.json();
    // Support both { data: [...] } and raw array
    const activities = (payload?.data ?? payload) || [];
    return { activities };
  } catch (e) {
    return { activities: [] };
  }
};

