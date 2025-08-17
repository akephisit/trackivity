import type { PageServerLoad } from './$types';
import { api } from '$lib/server/api-client';

export const load: PageServerLoad = async (event) => {
  const { depends } = event;
  depends('student:dashboard');

  // Recent activities for dashboard
  let recentActivities: any[] = [];
  try {
    const res = await api.get(event, `/api/activities`, { per_page: '5', active_only: 'true' });
    if (res.success) {
      recentActivities = res.data ?? [];
    }
  } catch (_) {}

  // Placeholder until real endpoints exist
  const participationHistory: any[] = [];
  const stats = {
    totalParticipations: 0,
    thisMonthParticipations: 0,
    upcomingActivities: Array.isArray(recentActivities) ? recentActivities.length : 0
  };

  return {
    recentActivities,
    participationHistory,
    stats
  };
};
