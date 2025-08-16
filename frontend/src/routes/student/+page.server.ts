import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, depends }) => {
  depends('student:dashboard');

  // Recent activities for dashboard
  let recentActivities: any[] = [];
  try {
    const params = new URLSearchParams({ per_page: '5', active_only: 'true' });
    const res = await fetch(`/api/activities?${params.toString()}`);
    if (res.ok) {
      const payload = await res.json();
      recentActivities = (payload?.data ?? payload) || [];
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

