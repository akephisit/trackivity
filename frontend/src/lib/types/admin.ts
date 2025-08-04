export enum AdminLevel {
	SuperAdmin = 'SuperAdmin',
	FacultyAdmin = 'FacultyAdmin',
	RegularAdmin = 'RegularAdmin'
}

export interface User {
	id: number;
	email: string;
	name: string;
	created_at: string;
	updated_at: string;
}

export interface Faculty {
	id: number;
	name: string;
	code: string;
	created_at: string;
	updated_at: string;
}

export interface AdminRole {
	id: number;
	user_id: number;
	admin_level: AdminLevel;
	faculty_id?: number;
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
	total_admins: number;
	total_faculties: number;
	recent_activities: number;
}

export interface ApiResponse<T = any> {
	success: boolean;
	data?: T;
	error?: string;
	message?: string;
}