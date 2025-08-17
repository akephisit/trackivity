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

// API Client class for SSR
export class ApiClient {
  private baseUrl: string;
  private timeout: number;

  constructor(config: ApiClientConfig = {}) {
    this.baseUrl = config.baseUrl || process.env.PUBLIC_API_URL || 'http://localhost:3000';
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
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers = this.createHeaders(event, options.headers as Record<string, string>);

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

        return {
          success: false,
          error: errorMessage,
          data: undefined
        };
      }

      // Normalize response format to { success, data }
      // If backend already returns { success }, honor it; else wrap
      if (typeof data === 'object' && data !== null) {
        if ('success' in data) {
          return data as ApiResponse<T>;
        }
        return { success: true, data: (data as any).data ?? data, message: (data as any).message } as ApiResponse<T>;
      }

      return { success: true, data: data as T } as ApiResponse<T>;

    } catch (err) {
      if (err && typeof err === 'object' && 'status' in err) {
        throw err; // Re-throw SvelteKit errors
      }

      console.error(`API request failed for ${endpoint}:`, err);
      throw error(500, 'เกิดข้อผิดพลาดในการเชื่อมต่อเซิร์ฟเวอร์');
    }
  }

  /**
   * GET request
   */
  async get<T = any>(event: RequestEvent, endpoint: string, params?: Record<string, string>): Promise<ApiResponse<T>> {
    const url = params ? `${endpoint}?${new URLSearchParams(params).toString()}` : endpoint;
    return this.makeRequest<T>(event, url, { method: 'GET' });
  }

  /**
   * POST request
   */
  async post<T = any>(event: RequestEvent, endpoint: string, body?: any): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined
    });
  }

  /**
   * PUT request
   */
  async put<T = any>(event: RequestEvent, endpoint: string, body?: any): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined
    });
  }

  /**
   * PATCH request
   */
  async patch<T = any>(event: RequestEvent, endpoint: string, body?: any): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, {
      method: 'PATCH',
      body: body ? JSON.stringify(body) : undefined
    });
  }

  /**
   * DELETE request
   */
  async delete<T = any>(event: RequestEvent, endpoint: string): Promise<ApiResponse<T>> {
    return this.makeRequest<T>(event, endpoint, { method: 'DELETE' });
  }

  /**
   * Upload file with multipart/form-data
   */
  async upload<T = any>(event: RequestEvent, endpoint: string, formData: FormData): Promise<ApiResponse<T>> {
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
    });
  }
}

// Default instance
export const apiClient = new ApiClient();

// Convenience functions
export const api = {
  get: <T = any>(event: RequestEvent, endpoint: string, params?: Record<string, string>) => 
    apiClient.get<T>(event, endpoint, params),
  
  post: <T = any>(event: RequestEvent, endpoint: string, body?: any) => 
    apiClient.post<T>(event, endpoint, body),
  
  put: <T = any>(event: RequestEvent, endpoint: string, body?: any) => 
    apiClient.put<T>(event, endpoint, body),
  
  patch: <T = any>(event: RequestEvent, endpoint: string, body?: any) => 
    apiClient.patch<T>(event, endpoint, body),
  
  delete: <T = any>(event: RequestEvent, endpoint: string) => 
    apiClient.delete<T>(event, endpoint),
  
  upload: <T = any>(event: RequestEvent, endpoint: string, formData: FormData) => 
    apiClient.upload<T>(event, endpoint, formData)
};
