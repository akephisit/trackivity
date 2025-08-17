export type ActivityType = 'Academic' | 'Sports' | 'Cultural' | 'Social' | 'Other';
export type ActivityStatus = 'draft' | 'published' | 'ongoing' | 'completed' | 'cancelled';
export type ParticipationStatus = 'registered' | 'checked_in' | 'checked_out' | 'completed';

// Updated to match backend API response structure
export interface Activity {
	id: string;
	title: string;
	description: string;
	location: string;
	start_time: string; // Full ISO datetime from backend
	end_time: string; // Full ISO datetime from backend
	max_participants?: number;
	current_participants: number;
	status: ActivityStatus;
	faculty_id?: string;
	faculty_name?: string;
	created_by: string;
	created_by_name: string;
	created_at: string;
	updated_at: string;
	is_registered: boolean;
	user_participation_status?: ParticipationStatus;
	// Legacy fields for backward compatibility
	activity_name?: string;
	start_date?: string;
	end_date?: string;
	start_time_only?: string;
	end_time_only?: string;
	activity_type?: ActivityType;
	hours?: number;
	organizer?: string;
	academic_year?: string;
	name?: string;
	require_score?: boolean;
	organizerType?: 'คณะ' | 'มหาวิทยาลัย';
	participantCount?: number;
	createdAt?: string;
	updatedAt?: string;
}

// For creating new activities
export interface ActivityCreateData {
	title: string;
	description: string;
	location: string;
	start_time: string; // ISO datetime
	end_time: string; // ISO datetime
	max_participants?: number;
	faculty_id?: string;
	department_id?: string;
}

// For updating activities
export interface ActivityUpdateData {
	title?: string;
	description?: string;
	location?: string;
	start_time?: string;
	end_time?: string;
	max_participants?: number;
	status?: ActivityStatus;
	faculty_id?: string;
	department_id?: string;
}

// Participation data
export interface Participation {
	id: string;
	user_id: string;
	user_name: string;
	student_id: string;
	email: string;
	department_name?: string;
	status: ParticipationStatus;
	registered_at: string;
	checked_in_at?: string;
	checked_out_at?: string;
	notes?: string;
}

// Legacy interfaces for backward compatibility
export interface ActivityFormData {
	activity_name: string;
	description: string | null;
	start_date: string;
	end_date: string;
	start_time: string;
	end_time: string;
	activity_type: ActivityType;
	location: string;
	max_participants?: number;
	hours: number;
	organizer: string;
	eligible_faculties: string;
	academic_year: string;
}

export interface ActivityApiResponse {
	status: string;
	data?: Activity | Activity[];
	error?: string;
	message?: string;
}
