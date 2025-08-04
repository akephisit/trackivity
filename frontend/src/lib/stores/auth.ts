import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import type { User } from '$lib/api/client';

interface AuthState {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;
}

const initialState: AuthState = {
	user: null,
	isAuthenticated: false,
	isLoading: true,
};

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>(initialState);

	return {
		subscribe,
		
		setUser: (user: User | null) => {
			update(state => ({
				...state,
				user,
				isAuthenticated: user !== null,
				isLoading: false,
			}));
		},

		setLoading: (isLoading: boolean) => {
			update(state => ({ ...state, isLoading }));
		},

		logout: () => {
			set({
				user: null,
				isAuthenticated: false,
				isLoading: false,
			});
		},

		reset: () => {
			set(initialState);
		}
	};
}

export const authStore = createAuthStore();