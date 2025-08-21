import type { RequestEvent } from '@sveltejs/kit';
import { error } from '@sveltejs/kit';

// API Response interface
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  error?: string;
}

// API Client configuration
interface ApiClientConfig {
  baseUrl?: string;
  timeout?: number;
}

// Optional behavior for error handling per request
interface RequestBehaviorOptions {
  // If true (default), HTTP errors will throw SvelteKit errors for 4xx/5xx.
  // If false, returns { success: false, error } instead of throwing.
  throwOnHttpError?: boolean;
}

// API Client class for SSR
export class ApiClient {
  private baseUrl: string;
  private timeout: number;

  constructor(config: ApiClientConfig = {}) {
    const rawUrl = config.baseUrl || process.env.PUBLIC_API_URL || 'http://localhost:3000';
    // Remove trailing slash to prevent double slashes
    this.baseUrl = rawUrl.replace(/\/+$/, '');
    this.timeout = config.timeout || 10000;
  }

  /**
   * Create authenticated headers from SvelteKit event
   */
  private createHeaders(event: RequestEvent, additionalHeaders: Record<string, string> = {}): Record<string, string> {
    const sessionId = event.cookies.get('session_id');
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...additionalHeaders
    };

    if (sessionId) {
      headers['Cookie'] = `session_id=${sessionId}`;
      headers['X-Session-ID'] = sessionId;
    }

    return headers;
  }

  /**
   * Make HTTP request with consistent error handling
   */
  private async makeRequest<T>(
    event: RequestEvent,
    endpoint: string,
    options: RequestInit = {},
    behavior: RequestBehaviorOptions = {}
  ): Promise<ApiResponse<T>> {
    const { throwOnHttpError = true } = behavior;
    const url = `${this.baseUrl}${endpoint}`;
    const headers = this.createHeaders(event, options.headers as Record<string, string>);

    console.log('=== API CLIENT DEBUG ===');
    console.log('baseUrl:', this.baseUrl);
    console.log('endpoint:', endpoint);
    console.log('final URL:', url);

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      const response = await event.fetch(url, {
        ...options,
        headers,
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      // Handle different content types
      const contentType = response.headers.get('content-type') || '';
      let data: any;

      if (contentType.includes('application/json')) {
        data = await response.json().catch(() => ({}));
      } else {
        const text = await response.text();
        data = text ? { message: text } : {};
      }

      if (!response.ok) {
        // Standard error handling
        const errorMessage = data?.message || data?.error || `HTTP ${response.status}`;
        if (throwOnHttpError) {
          // Convert HTTP errors to SvelteKit errors
          if (response.status === 401) {
            throw error(401, 'ไม่มีการ authentication');
          } else if (response.status === 403) {
            throw error(403, 'ไม่มีสิทธิ์เข้าถึงข้อมูลนี้');
          } else if (response.status === 404) {
            throw error(404, 'ไม่พบข้อมูลที่ระบุ');
          } else if (response.status >= 500) {
            throw error(500, 'เกิดข้อผิดพลาดของเซิร์ฟเวอร์');
          }
        }

        // Non-throwing path
        return { success: false, error: errorMessage, data: undefined };
      }

      // Normalize to unified shape { success, data, error }
      if (typeof data === 'object' && data !== null) {
        if ('success' in data) {
          const d: any = data;
          if (d.success === true) {
            // Ensure payload is under data for consumers
            return {
              success: true,
              data: d.data ?? d,
              message: d.message
            } as ApiResponse<T>;
          } else {
            return {
              success: false,
              error: d.error || d.message || 'Request failed',
              data: d.data
            } as ApiResponse<T>;
          }
        }
        // Legacy/non-standard success shape
        return { success: true, data: (data as any).data ?? data, message: (data as any).message } as ApiResponse<T>;
      }

      return { success: true, data: data as T } as ApiResponse<T>;

    } catch (err) {
      if (err && typeof err === 'object' && 'status' in err) {
        throw err; // Re-throw SvelteKit errors
      }

      const errorMessage = err instanceof Error ? err.message : String(err);
      console.error(`API request failed for ${endpoint}:`, {
        error: err,
        message: errorMessage,
        url: `${this.baseUrl}${endpoint}`,
        timestamp: new Date().toISOString()
      });
      
      // Return more detailed error information
      const detailedError = `เกิดข้อผิดพลาดในการเชื่อมต่อ: ${errorMessage}`;
      
      if ((options as any)?.throwOnHttpError === false || behavior.throwOnHttpError === false) {
        return { success: false, error: detailedError } as ApiResponse<T>;
      }
      throw error(500, detailedError);
    }
  }

  /**
   * GET request
   */
  async get<T = any>(
    event: RequestEvent,
    endpoint: string,
    params?: Record<string, string>,
    behavior?: RequestBehaviorOptions
  ): Promise<ApiResponse<T>> {
    const url = params ? `${endpoint}?${new URLSearchParams(params).toString()}` : endpoint;
    return this.makeRequest<T>(event, url, { method: 'GET' }, behavior);
  }

  /**
   * POST request
   */
  async post<T = any>(
    event: RequestEvent,
    endpoint: string,
    body?: any,
    behavior?: RequestBehaviorOptions
  ): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined
    }, behavior);
  }

  /**
   * PUT request
   */
  async put<T = any>(
    event: RequestEvent,
    endpoint: string,
    body?: any,
    behavior?: RequestBehaviorOptions
  ): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined
    }, behavior);
  }

  /**
   * PATCH request
   */
  async patch<T = any>(
    event: RequestEvent,
    endpoint: string,
    body?: any,
    behavior?: RequestBehaviorOptions
  ): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, {
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined
    }, behavior);
  }

  /**
   * DELETE request
   */
  async delete<T = any>(
    event: RequestEvent,
    endpoint: string,
    behavior?: RequestBehaviorOptions
  ): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, { method: 'DELETE' }, behavior);
  }

  /**
   * Upload file with multipart/form-data
   */
  async upload<T = any>(
    event: RequestEvent,
    endpoint: string,
    formData: FormData,
    behavior?: RequestBehaviorOptions
  ): Promise<ApiResponse<T>> {
    const sessionId = event.cookies.get('session_id');
    const headers: Record<string, string> = {};

    if (sessionId) {
      headers['Cookie'] = `session_id=${sessionId}`;
      headers['X-Session-ID'] = sessionId;
    }

    return this.makeRequest<T>(event, endpoint, {
      method: 'POST',
      headers,
      body: formData
    }, behavior);
  }
}

// Default instance
export const apiClient = new ApiClient();

// Convenience functions
export const api = {
  get: <T = any>(
    event: RequestEvent,
    endpoint: string,
    params?: Record<string, string>,
    behavior?: RequestBehaviorOptions
  ) => apiClient.get<T>(event, endpoint, params, behavior),
  
  post: <T = any>(
    event: RequestEvent,
    endpoint: string,
    body?: any,
    behavior?: RequestBehaviorOptions
  ) => apiClient.post<T>(event, endpoint, body, behavior),
  
  put: <T = any>(
    event: RequestEvent,
    endpoint: string,
    body?: any,
    behavior?: RequestBehaviorOptions
  ) => apiClient.put<T>(event, endpoint, body, behavior),
  
  patch: <T = any>(
    event: RequestEvent,
    endpoint: string,
    body?: any,
    behavior?: RequestBehaviorOptions
  ) => apiClient.patch<T>(event, endpoint, body, behavior),
  
  delete: <T = any>(
    event: RequestEvent,
    endpoint: string,
    behavior?: RequestBehaviorOptions
  ) => apiClient.delete<T>(event, endpoint, behavior),
  
  upload: <T = any>(event: RequestEvent, endpoint: string, formData: FormData) => 
    apiClient.upload<T>(event, endpoint, formData)
};
