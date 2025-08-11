// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
import type { SessionUser, ApiError } from '$lib/types';

declare global {
	namespace App {
		interface Error {
			message: string;
			code?: string;
			details?: Record<string, any>;
		}
		
		interface Locals {
			user: SessionUser | null;
			session_id: string | null;
		}
		
		interface PageData {
			user?: SessionUser | null;
		}
		
		interface PageState {
			user?: SessionUser | null;
		}
		// interface Platform {}
	}
}

export {};
