export enum AdminLevel {
	SuperAdmin = 'SuperAdmin',
	FacultyAdmin = 'FacultyAdmin', 
	RegularAdmin = 'RegularAdmin'
}

// For API compatibility
export type AdminLevelAPI = 'super_admin' | 'faculty_admin' | 'regular_admin';

export interface User {
	id: string; // UUID string
	email: string;
	first_name: string;
	last_name: string;
	student_id?: string;
	department_id?: string;
	created_at: string;
	updated_at: string;
}

export interface Faculty {
	id: string; // UUID string
	name: string;
	code: string;
	created_at: string;
	updated_at: string;
}

export interface AdminRole {
	id: string; // UUID string
	user_id: string; // UUID string
	admin_level: AdminLevel;
	faculty_id?: string; // UUID string
	permissions: string[];
	created_at: string;
	updated_at: string;
	user?: User;
	faculty?: Faculty;
}

export interface LoginCredentials {
	email: string;
	password: string;
}

export interface RegisterData {
	email: string;
	password: string;
	name: string;
	admin_level?: AdminLevel;
	faculty_id?: number;
}

export interface AuthSession {
	user: User;
	admin_role?: AdminRole;
	session_id: string;
	expires_at: string;
}

export interface AdminDashboardStats {
	total_users: number;
	total_activities: number;
	total_participations: number;
	active_sessions: number;
	ongoing_activities: number;
	user_registrations_today: number;
	recent_activities?: any[];
	popular_activities?: any[];
}

export interface ApiResponse<T = any> {
	success: boolean;
	data?: T;
	error?: string;
	message?: string;
}