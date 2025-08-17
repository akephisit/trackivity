export type ActivityType = 'Academic' | 'Sports' | 'Cultural' | 'Social' | 'Other';
export type ActivityStatus = 'รอดำเนินการ' | 'กำลังดำเนินการ' | 'เสร็จสิ้น';

export interface Activity {
	id: string;
	activity_name: string;
	description: string;
	start_date: string; // ISO date
	end_date: string; // ISO date
	start_time: string; // HH:MM format
	end_time: string; // HH:MM format
	activity_type: ActivityType;
	location: string;
	max_participants?: number;
	hours?: number; // total hours credited for activity
	organizer: string;
	faculty_id?: string;
	created_by: string; // admin user id
	created_at: string;
	updated_at: string;
	// For display purposes
	name?: string; // legacy field
	require_score?: boolean; // legacy field
	organizerType?: 'คณะ' | 'มหาวิทยาลัย'; // legacy field
	participantCount?: number; // legacy field
	status?: ActivityStatus; // legacy field
	createdAt?: string; // legacy field
	updatedAt?: string; // legacy field
}

export interface ActivityCreateData {
	activity_name: string;
	description: string | null;
	start_date: string;
	end_date: string;
	start_time: string;
	end_time: string;
	activity_type: ActivityType;
	location: string;
	max_participants?: number;
	hours?: number;
	organizer: string;
	eligible_faculties: string;
	academic_year: string;
}

export interface ActivityFormData extends ActivityCreateData {
	// Extends ActivityCreateData for form handling
}

export interface ActivityApiResponse {
	success: boolean;
	data?: Activity;
	error?: string;
	message?: string;
}
