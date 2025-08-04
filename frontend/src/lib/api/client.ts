import { browser } from '$app/environment';

export interface User {
	id: string;
	student_id: string;
	email: string;
	first_name: string;
	last_name: string;
	department_id?: string;
	created_at?: string;
	updated_at?: string;
}

export interface LoginRequest {
	student_id: string;
	password: string;
}

export interface RegisterRequest {
	student_id: string;
	email: string;
	password: string;
	first_name: string;
	last_name: string;
	department_id?: string;
}

export interface LoginResponse {
	user: User;
	session_id: string;
}

export interface RegisterResponse {
	user: User;
	message: string;
}

export interface ApiErrorResponse {
	message: string;
	status: number;
}

class ApiClient {
	private baseUrl: string;

	constructor(baseUrl: string = '/api') {
		this.baseUrl = baseUrl;
	}

	private async request<T>(
		endpoint: string,
		options: RequestInit = {}
	): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;
		
		const config: RequestInit = {
			credentials: 'include',
			headers: {
				'Content-Type': 'application/json',
				...options.headers,
			},
			...options,
		};

		try {
			const response = await fetch(url, config);
			
			if (!response.ok) {
				throw new ApiError(`HTTP error! status: ${response.status}`, response.status);
			}

			// Check if response has content
			const contentLength = response.headers.get('content-length');
			if (contentLength === '0' || response.status === 204) {
				return {} as T;
			}

			return await response.json();
		} catch (error) {
			if (error instanceof SyntaxError) {
				throw new ApiError('Invalid JSON response', 500);
			}
			throw error;
		}
	}

	// Authentication methods
	async login(credentials: LoginRequest): Promise<LoginResponse> {
		return this.request<LoginResponse>('/auth/login', {
			method: 'POST',
			body: JSON.stringify(credentials),
		});
	}

	async register(userData: RegisterRequest): Promise<RegisterResponse> {
		return this.request<RegisterResponse>('/auth/register', {
			method: 'POST',
			body: JSON.stringify(userData),
		});
	}

	async logout(): Promise<void> {
		return this.request<void>('/auth/logout', {
			method: 'POST',
		});
	}

	async me(): Promise<User> {
		return this.request<User>('/auth/me');
	}

	// Health check
	async health(): Promise<string> {
		return this.request<string>('/health');
	}
}

// Custom error class
export class ApiError extends Error {
	status: number;

	constructor(message: string, status: number) {
		super(message);
		this.status = status;
		this.name = 'ApiError';
	}
}

// Export singleton instance
export const apiClient = new ApiClient();