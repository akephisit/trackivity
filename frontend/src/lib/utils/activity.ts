import type { ActivityStatus } from '$lib/types/activity';

/**
 * แปลงค่า ActivityStatus จากรูปแบบ frontend (lowercase) เป็น backend (capitalized)
 * Frontend: 'draft' | 'published' | 'ongoing' | 'completed' | 'cancelled'
 * Backend: 'Draft' | 'Published' | 'Ongoing' | 'Completed' | 'Cancelled'
 */
export function convertStatusForBackend(status: ActivityStatus): string {
	const statusMapping: Record<ActivityStatus, string> = {
		draft: 'Draft',
		published: 'Published',
		ongoing: 'Ongoing',
		completed: 'Completed',
		cancelled: 'Cancelled'
	};

	return statusMapping[status] || status;
}

/**
 * แปลงค่า ActivityStatus จากรูปแบบ backend (capitalized) เป็น frontend (lowercase)
 * Backend: 'Draft' | 'Published' | 'Ongoing' | 'Completed' | 'Cancelled'
 * Frontend: 'draft' | 'published' | 'ongoing' | 'completed' | 'cancelled'
 */
export function convertStatusFromBackend(status: string): ActivityStatus {
	const statusMapping: Record<string, ActivityStatus> = {
		Draft: 'draft',
		Published: 'published',
		Ongoing: 'ongoing',
		Completed: 'completed',
		Cancelled: 'cancelled'
	};

	return statusMapping[status] || (status.toLowerCase() as ActivityStatus);
}

/**
 * ตรวจสอบว่า status string เป็นค่าที่ valid หรือไม่
 */
export function isValidActivityStatus(status: string): status is ActivityStatus {
	const validStatuses: ActivityStatus[] = ['draft', 'published', 'ongoing', 'completed', 'cancelled'];
	return validStatuses.includes(status as ActivityStatus);
}