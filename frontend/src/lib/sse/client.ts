/**
 * SSE (Server-Sent Events) Client for Real-time Updates
 * รองรับ session-based authentication และ automatic reconnection
 */

import { browser } from '$app/environment';
import { writable, type Writable } from 'svelte/store';
import type { SSEEvent, SSEEventType, SSEConfig, SessionUser } from '$lib/types';

// ===== CONFIGURATION =====
const DEFAULT_CONFIG: SSEConfig = {
  auto_reconnect: true,
  reconnect_interval: 5000, // 5 seconds
  max_reconnect_attempts: 10,
  heartbeat_interval: 30000 // 30 seconds
};

// ===== CONNECTION STATUS =====
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error' | 'reconnecting';

// ===== SSE CLIENT CLASS =====
export class SSEClient {
  private eventSource: EventSource | null = null;
  private config: SSEConfig;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private isManualClose = false;

  // Svelte stores for reactive updates
  public connectionStatus: Writable<ConnectionStatus> = writable('disconnected');
  public events: Writable<SSEEvent[]> = writable([]);
  public lastEvent: Writable<SSEEvent | null> = writable(null);
  public errorMessage: Writable<string | null> = writable(null);

  // Event listeners
  private eventListeners: Map<SSEEventType, Set<(event: SSEEvent) => void>> = new Map();
  private globalListeners: Set<(event: SSEEvent) => void> = new Set();

  constructor(config: Partial<SSEConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };

    if (browser) {
      // Auto-connect on page visibility change
      document.addEventListener('visibilitychange', () => {
        if (!document.hidden && this.getConnectionStatus() === 'disconnected') {
          this.connect();
        }
      });

      // Cleanup on page unload
      window.addEventListener('beforeunload', () => {
        this.disconnect();
      });
    }
  }

  // ===== CONNECTION MANAGEMENT =====
  public connect(user?: SessionUser): void {
    if (!browser) return;

    // Check if we have a session before attempting to connect
    const sessionId = this.getSessionId();
    if (!sessionId) {
      console.log('[SSE] No session ID found - skipping SSE connection');
      this.connectionStatus.set('disconnected');
      return;
    }

    this.isManualClose = false;
    this.connectionStatus.set('connecting');
    this.errorMessage.set(null);

    try {
      const url = this.buildSSEUrl();
      console.log('[SSE] Attempting to connect to:', url);
      
      this.eventSource = new EventSource(url, {
        withCredentials: true
      });

      this.setupEventListeners();
      this.startHeartbeat();

    } catch (error) {
      console.error('[SSE] Failed to create EventSource:', error);
      this.handleError('Failed to establish connection');
    }
  }

  public disconnect(): void {
    this.isManualClose = true;
    this.cleanup();
    this.connectionStatus.set('disconnected');
  }

  public reconnect(): void {
    this.disconnect();
    setTimeout(() => this.connect(), 1000);
  }

  // ===== EVENT SUBSCRIPTION =====
  public on<T = any>(eventType: SSEEventType, listener: (event: SSEEvent<T>) => void): () => void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    
    this.eventListeners.get(eventType)!.add(listener);

    // Return unsubscribe function
    return () => {
      const listeners = this.eventListeners.get(eventType);
      if (listeners) {
        listeners.delete(listener);
        if (listeners.size === 0) {
          this.eventListeners.delete(eventType);
        }
      }
    };
  }

  public onAny(listener: (event: SSEEvent) => void): () => void {
    this.globalListeners.add(listener);
    
    return () => {
      this.globalListeners.delete(listener);
    };
  }

  public off(eventType: SSEEventType, listener?: (event: SSEEvent) => void): void {
    if (!listener) {
      this.eventListeners.delete(eventType);
      return;
    }

    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.delete(listener);
      if (listeners.size === 0) {
        this.eventListeners.delete(eventType);
      }
    }
  }

  public offAll(): void {
    this.eventListeners.clear();
    this.globalListeners.clear();
  }

  // ===== GETTERS =====
  public getConnectionStatus(): ConnectionStatus {
    let status: ConnectionStatus = 'disconnected';
    this.connectionStatus.subscribe(s => status = s)();
    return status;
  }

  public isConnected(): boolean {
    return this.getConnectionStatus() === 'connected';
  }

  // ===== PRIVATE METHODS =====
  private buildSSEUrl(): string {
    const baseUrl = browser 
      ? (window.location.origin.includes('localhost') ? 'http://localhost:3000' : window.location.origin)
      : 'http://localhost:3000';
    
    const sessionId = this.getSessionId();
    if (!sessionId) {
      throw new Error('No session ID available for SSE connection');
    }

    // Backend expects session_id as path parameter, not query parameter
    // Route: /api/sse/{session_id}
    return `${baseUrl}/api/sse/${sessionId}`;
  }

  private getSessionId(): string | null {
    if (!browser) return null;
    
    // Try cookie first
    const cookieMatch = document.cookie.match(/session_id=([^;]+)/);
    if (cookieMatch) return cookieMatch[1];
    
    // Try localStorage for mobile
    return localStorage.getItem('session_id');
  }

  private getDeviceInfo(): { device_type: 'web' | 'mobile' | 'tablet' } {
    if (!browser) return { device_type: 'web' };

    const userAgent = navigator.userAgent;
    const isMobile = /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(userAgent);
    const isTablet = /iPad/i.test(userAgent);

    return {
      device_type: isTablet ? 'tablet' : isMobile ? 'mobile' : 'web'
    };
  }

  private setupEventListeners(): void {
    if (!this.eventSource) return;

    this.eventSource.onopen = () => {
      console.log('[SSE] Connection established');
      this.connectionStatus.set('connected');
      this.reconnectAttempts = 0;
      this.errorMessage.set(null);
    };

    this.eventSource.onerror = (error) => {
      console.error('[SSE] Connection error:', error);
      console.log('[SSE] EventSource readyState:', this.eventSource?.readyState);
      console.log('[SSE] EventSource state constants:', {
        CONNECTING: EventSource.CONNECTING,
        OPEN: EventSource.OPEN,
        CLOSED: EventSource.CLOSED
      });
      this.handleConnectionError();
    };

    this.eventSource.onmessage = (event) => {
      try {
        const sseEvent: SSEEvent = JSON.parse(event.data);
        this.handleEvent(sseEvent);
      } catch (error) {
        console.error('Failed to parse SSE event:', error);
      }
    };

    // Listen for specific event types
    const eventTypes: SSEEventType[] = [
      'session_updated',
      'session_expired',
      'permission_changed',
      'activity_created',
      'activity_updated',
      'activity_deleted',
      'qr_refresh',
      'participation_recorded',
      'notification',
      'system_alert'
    ];

    eventTypes.forEach(eventType => {
      this.eventSource!.addEventListener(eventType, (event: any) => {
        try {
          const sseEvent: SSEEvent = {
            event_type: eventType,
            data: JSON.parse(event.data),
            timestamp: new Date().toISOString(),
            user_id: undefined,
            session_id: this.getSessionId() || undefined
          };
          this.handleEvent(sseEvent);
        } catch (error) {
          console.error(`Failed to parse ${eventType} event:`, error);
        }
      });
    });
  }

  private handleEvent(event: SSEEvent): void {
    // Update stores
    this.lastEvent.set(event);
    this.events.update(events => [...events.slice(-99), event]); // Keep last 100 events

    // Call specific event listeners
    const listeners = this.eventListeners.get(event.event_type);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(event);
        } catch (error) {
          console.error(`Error in ${event.event_type} listener:`, error);
        }
      });
    }

    // Call global listeners
    this.globalListeners.forEach(listener => {
      try {
        listener(event);
      } catch (error) {
        console.error('Error in global SSE listener:', error);
      }
    });

    // Handle specific event types
    this.handleSpecialEvents(event);
  }

  private handleSpecialEvents(event: SSEEvent): void {
    switch (event.event_type) {
      case 'session_expired':
        // Redirect to login
        if (browser) {
          window.location.href = '/login?expired=true';
        }
        break;
      
      case 'permission_changed':
        // Trigger page reload to update permissions
        if (browser) {
          window.location.reload();
        }
        break;

      case 'qr_refresh':
        // QR codes might need refreshing
        this.dispatchCustomEvent('qr-refresh', event.data);
        break;

      case 'notification':
        // Show browser notification if permission granted
        this.showNotification(event.data);
        break;
    }
  }

  private dispatchCustomEvent(name: string, detail: any): void {
    if (browser) {
      window.dispatchEvent(new CustomEvent(name, { detail }));
    }
  }

  private showNotification(data: any): void {
    if (!browser || !('Notification' in window)) return;

    if (Notification.permission === 'granted') {
      new Notification(data.title || 'Trackivity', {
        body: data.message,
        icon: '/favicon.ico',
        tag: data.type || 'general'
      });
    }
  }

  private handleConnectionError(): void {
    if (this.isManualClose) return;

    this.connectionStatus.set('error');
    
    if (this.config.auto_reconnect && this.reconnectAttempts < this.config.max_reconnect_attempts) {
      this.attemptReconnect();
    } else {
      this.handleError('Connection failed and max reconnect attempts reached');
    }
  }

  private attemptReconnect(): void {
    if (this.isManualClose) return;

    this.reconnectAttempts++;
    this.connectionStatus.set('reconnecting');
    
    const delay = Math.min(
      this.config.reconnect_interval * Math.pow(2, this.reconnectAttempts - 1),
      30000 // Max 30 seconds
    );

    this.reconnectTimer = setTimeout(() => {
      console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.config.max_reconnect_attempts})...`);
      this.cleanup();
      this.connect();
    }, delay);
  }

  private startHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
    }

    this.heartbeatTimer = setInterval(() => {
      if (this.eventSource && this.eventSource.readyState === EventSource.OPEN) {
        // Send heartbeat via custom event (backend should respond)
        this.dispatchCustomEvent('sse-heartbeat', { timestamp: Date.now() });
      }
    }, this.config.heartbeat_interval);
  }

  private cleanup(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private handleError(message: string): void {
    console.error('SSE Error:', message);
    this.errorMessage.set(message);
    this.connectionStatus.set('error');
  }
}

// ===== SINGLETON INSTANCE =====
export const sseClient = new SSEClient();

// ===== UTILITY FUNCTIONS =====
export function createSSEClient(config?: Partial<SSEConfig>): SSEClient {
  return new SSEClient(config);
}

// ===== COMPOSABLE FOR SVELTE COMPONENTS =====
export function useSSE(eventType?: SSEEventType) {
  const events = writable<SSEEvent[]>([]);
  const lastEvent = writable<SSEEvent | null>(null);
  const connectionStatus = sseClient.connectionStatus;

  let unsubscribe: (() => void) | null = null;

  const subscribe = () => {
    if (eventType) {
      unsubscribe = sseClient.on(eventType, (event) => {
        lastEvent.set(event);
        events.update(list => [...list.slice(-49), event]); // Keep last 50 events
      });
    } else {
      unsubscribe = sseClient.onAny((event) => {
        lastEvent.set(event);
        events.update(list => [...list.slice(-49), event]); // Keep last 50 events
      });
    }
  };

  const destroy = () => {
    if (unsubscribe) {
      unsubscribe();
      unsubscribe = null;
    }
  };

  return {
    events,
    lastEvent,
    connectionStatus,
    subscribe,
    destroy,
    connect: () => sseClient.connect(),
    disconnect: () => sseClient.disconnect(),
    reconnect: () => sseClient.reconnect()
  };
}