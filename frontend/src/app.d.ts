// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		interface Error {
			message: string;
			code?: string;
		}
		
		interface Locals {
			user: SessionUser | null;
			session_id: string | null;
		}
		
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

// Type definitions for session management
interface SessionUser {
	user_id: string;
	student_id: string;
	email: string;
	first_name: string;
	last_name: string;
	department_id?: string;
	admin_role?: AdminRole;
	session_id: string;
	permissions: string[];
	faculty_id?: string;
}

interface AdminRole {
	id: string;
	admin_level: 'SuperAdmin' | 'FacultyAdmin' | 'RegularAdmin';
	faculty_id?: string;
	permissions: string[];
}

export {};
