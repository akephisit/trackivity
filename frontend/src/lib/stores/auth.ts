import { writable } from 'svelte/store';
import type { User } from '$lib/api/client';

interface AuthState {
	user: User | null;
	isAuthenticated: boolean;
}

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		isAuthenticated: false
	});

	return {
		subscribe,
		login: (user: User) => {
			set({
				user,
				isAuthenticated: true
			});
		},
		logout: () => {
			set({
				user: null,
				isAuthenticated: false
			});
		},
		updateUser: (user: User) => {
			update(state => ({
				...state,
				user
			}));
		}
	};
}

export const authStore = createAuthStore();