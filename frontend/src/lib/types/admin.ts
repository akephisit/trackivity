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
	employee_id?: string;
	department_id?: string;
	faculty_id?: string;
    status: 'active' | 'inactive' | 'suspended' | 'online' | 'offline' | 'disabled';
	role: 'student' | 'faculty' | 'staff' | 'admin' | 'super_admin' | 'faculty_admin' | 'regular_admin';
	phone?: string;
	avatar?: string;
	last_login?: string;
	email_verified_at?: string;
	created_at: string;
	updated_at: string;
	department?: Department;
	faculty?: Faculty;
}

export interface Faculty {
	id: string; // UUID string
	name: string;
	code: string;
	description?: string;
	status: boolean;
	created_at: string;
	updated_at: string;
}

export interface Department {
	id: string; // UUID string
	name: string;
	code: string;
	description?: string;
	faculty_id: string; // UUID string
	head_name?: string;
	head_email?: string;
	students_count?: number;
	admins_count?: number;
	status: boolean;
	created_at: string;
	updated_at: string;
	faculty?: Faculty;
	department_admins?: ExtendedAdminRole[];
}

export interface AdminRole {
	id: string; // UUID string
	user_id: string; // UUID string
	admin_level: AdminLevel;
	faculty_id?: string; // UUID string
	permissions: string[];
	is_enabled: boolean; // Whether admin account is enabled (can login)
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
	// System-wide stats (SuperAdmin)
	total_users: number;
	total_activities: number;
	total_participations: number;
	active_sessions: number;
	ongoing_activities: number;
	user_registrations_today: number;
	recent_activities?: any[];
	popular_activities?: any[];
	
	// Faculty-specific stats (FacultyAdmin)
	faculty_users?: number;
	departments_count?: number;
	active_users?: number;
	new_users_this_month?: number;
	department_breakdown?: Array<{
		id: string;
		name: string;
		user_count: number;
		active_users?: number;
	}>;
}

export interface ApiResponse<T = any> {
	success: boolean;
	data?: T;
	error?: string;
	message?: string;
}

// User Management Types
export interface UserFilter {
	search?: string;
	faculty_id?: string;
	department_id?: string;
	status?: 'active' | 'inactive' | 'suspended' | 'online' | 'offline' | 'disabled' | 'all';
	role?: 'student' | 'faculty' | 'staff' | 'admin' | 'super_admin' | 'faculty_admin' | 'regular_admin' | 'all';
	created_after?: string;
	created_before?: string;
}

export interface UserPagination {
	page: number;
	limit: number;
	total: number;
	pages: number;
}

export interface UserListResponse {
	users: User[];
	pagination: UserPagination;
	filters: UserFilter;
	total_count: number;
}

export interface UserStats {
	total_users: number;
	active_users: number;
	inactive_users: number;
	students: number;
	faculty: number;
	staff: number;
	recent_registrations: number;
	faculty_breakdown?: Array<{
		faculty_id: string;
		faculty_name: string;
		user_count: number;
	}>;
}

export interface UserActivity {
	id: string;
	user_id: string;
	action: 'login' | 'logout' | 'profile_update' | 'status_change' | 'role_change';
	description: string;
	ip_address?: string;
	user_agent?: string;
	created_at: string;
}

export interface UserUpdateRequest {
	first_name?: string;
	last_name?: string;
	email?: string;
	phone?: string;
	status?: 'active' | 'inactive' | 'suspended';
	role?: 'student' | 'faculty' | 'staff' | 'admin';
	department_id?: string;
	faculty_id?: string;
}

export interface BulkUserOperation {
	user_ids: string[];
	operation: 'activate' | 'deactivate' | 'suspend' | 'delete' | 'change_role' | 'transfer_faculty';
	params?: {
		status?: 'active' | 'inactive' | 'suspended';
		role?: 'student' | 'faculty' | 'staff' | 'admin';
		faculty_id?: string;
		department_id?: string;
	};
}

export interface UserExportOptions {
	format: 'csv' | 'xlsx' | 'pdf';
	filters?: UserFilter;
	fields?: string[];
	include_stats?: boolean;
}

// Faculty Admin Management Types
export interface FacultyAdminFilter {
	search?: string;
	faculty_id?: string;
	status?: 'active' | 'inactive' | 'suspended' | 'all';
	admin_level?: AdminLevel;
	last_login_after?: string;
	last_login_before?: string;
	created_after?: string;
	created_before?: string;
	has_permissions?: string[];
}

export interface FacultyAdminPagination {
	page: number;
	limit: number;
	total: number;
	pages: number;
}

export interface FacultyAdminListResponse {
	admins: AdminRole[];
	pagination: FacultyAdminPagination;
	filters: FacultyAdminFilter;
}

export interface FacultyAdminStats {
	total_admins: number;
	active_admins: number;
	inactive_admins: number;
	recent_logins: number; // admins logged in within last 7 days
	total_faculties: number;
	faculty_breakdown?: Array<{
		faculty_id: string;
		faculty_name: string;
		admin_count: number;
		active_count: number;
	}>;
	permission_breakdown?: Array<{
		permission: string;
		count: number;
	}>;
}

export interface FacultyAdminActivity {
	id: string;
	admin_id: string;
	action: 'login' | 'logout' | 'permission_change' | 'status_change' | 'profile_update' | 'password_change';
	description: string;
	ip_address?: string;
	user_agent?: string;
	created_at: string;
	admin?: AdminRole;
}

export interface FacultyAdminUpdateRequest {
	first_name?: string;
	last_name?: string;
	email?: string;
	status?: 'active' | 'inactive' | 'suspended';
	faculty_id?: string;
	permissions?: string[];
	admin_level?: AdminLevel;
}

export interface BulkAdminOperation {
	admin_ids: string[];
	operation: 'activate' | 'deactivate' | 'suspend' | 'delete' | 'change_faculty' | 'update_permissions';
	params?: {
		status?: 'active' | 'inactive' | 'suspended';
		faculty_id?: string;
		permissions?: string[];
		admin_level?: AdminLevel;
	};
}

export interface FacultyAdminExportOptions {
	format: 'csv' | 'xlsx' | 'pdf';
	filters?: FacultyAdminFilter;
	fields?: string[];
	include_stats?: boolean;
	include_permissions?: boolean;
}

// Enhanced AdminRole interface with additional properties
export interface ExtendedAdminRole extends AdminRole {
	is_active?: boolean;  // Whether admin has active login session (from sessions)
	is_enabled: boolean;  // Whether admin account is enabled (from admin_roles)
	last_login_formatted?: string;
	created_at_formatted?: string;
	permission_count?: number;
	days_since_last_login?: number;
	full_name?: string;
	assigned_departments?: Department[];
	department_count?: number;
}

// Faculty Admin Dashboard Statistics
export interface FacultyAdminDashboardStats extends AdminDashboardStats {
	// Faculty admin specific stats
	faculty_admin_count?: number;
	department_admins?: number;
	permission_distribution?: Array<{
		permission: string;
		admin_count: number;
		percentage: number;
	}>;
	recent_activities?: FacultyAdminActivity[];
	login_frequency?: Array<{
		date: string;
		login_count: number;
	}>;
}

// Permission definitions
export const ADMIN_PERMISSIONS = {
	VIEW_DASHBOARD: 'ViewDashboard',
	MANAGE_ACTIVITIES: 'ManageActivities', 
	MANAGE_USERS: 'ManageUsers',
	MANAGE_FACULTY_USERS: 'ManageFacultyUsers',
	MANAGE_DEPARTMENTS: 'ManageDepartments',
	VIEW_REPORTS: 'ViewReports',
	MANAGE_FACULTY_SETTINGS: 'ManageFacultySettings',
	EXPORT_DATA: 'ExportData'
} as const;

export type AdminPermission = typeof ADMIN_PERMISSIONS[keyof typeof ADMIN_PERMISSIONS];

// Status definitions
export const ADMIN_STATUS = {
	ACTIVE: 'active',
	INACTIVE: 'inactive', 
	SUSPENDED: 'suspended'
} as const;

export type AdminStatus = typeof ADMIN_STATUS[keyof typeof ADMIN_STATUS];
