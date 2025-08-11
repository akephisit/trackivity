import { getAuthUser } from '$lib/server/auth';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	// Try to get authenticated user (optional - won't redirect if not logged in)
	const user = await getAuthUser(event);
	
	return {
		user
	};
};