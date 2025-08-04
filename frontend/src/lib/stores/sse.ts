import { writable, derived, get } from 'svelte/store';
import { browser } from '$app/environment';
import { authStore } from './auth';
import { goto } from '$app/navigation';

// Types for SSE messages
export interface SseMessage {
    event_type: string;
    data: any;
    timestamp: string;
    target_permissions?: string[];
    target_user_id?: string;
    target_faculty_id?: string;
}

export interface NotificationMessage {
    title: string;
    message: string;
    notification_type: 'info' | 'warning' | 'error' | 'success';
    action_url?: string;
    expires_at?: string;
    timestamp?: string;
}

export interface SessionUpdateMessage {
    session_id: string;
    action: string; // "force_logout" | "permission_changed" | "extended"
    reason?: string;
    new_expires_at?: string;
}

export interface ActivityUpdateMessage {
    activity_id: string;
    title: string;
    update_type: string; // "created" | "updated" | "cancelled" | "started"
    message: string;
}

// SSE store state
interface SseState {
    connected: boolean;
    connecting: boolean;
    error: string | null;
    lastMessage: SseMessage | null;
    notifications: NotificationMessage[];
    connectionCount: number;
}

const initialState: SseState = {
    connected: false,
    connecting: false,
    error: null,
    lastMessage: null,
    notifications: [],
    connectionCount: 0
};

// Create writable store
export const sseStore = writable<SseState>(initialState);

// Derived stores
export const isConnected = derived(sseStore, $sse => $sse.connected);
export const notifications = derived(sseStore, $sse => $sse.notifications);
export const unreadCount = derived(notifications, $notifications => 
    $notifications.filter(n => !n.expires_at || new Date(n.expires_at) > new Date()).length
);

// SSE Service class
class SseService {
    private eventSource: EventSource | null = null;
    private maxReconnectAttempts = 5;
    private reconnectAttempt = 0;
    private reconnectTimeout: NodeJS.Timeout | null = null;

    // Connect to SSE endpoint
    connect(sessionId: string): void {
        if (!browser || this.eventSource?.readyState === EventSource.OPEN) {
            return;
        }

        this.cleanup();
        
        sseStore.update(state => ({
            ...state,
            connecting: true,
            error: null
        }));

        try {
            this.eventSource = new EventSource(`/api/sse/${sessionId}`, {
                withCredentials: true
            });

            this.setupEventListeners();
            
        } catch (error) {
            console.error('Failed to create SSE connection:', error);
            sseStore.update(state => ({
                ...state,
                connecting: false,
                error: error instanceof Error ? error.message : 'Connection failed'
            }));
        }
    }

    private setupEventListeners(): void {
        if (!this.eventSource) return;

        this.eventSource.onopen = () => {
            console.log('SSE connection established');
            this.reconnectAttempt = 0;
            
            sseStore.update(state => ({
                ...state,
                connected: true,
                connecting: false,
                error: null
            }));
        };

        this.eventSource.onerror = (event) => {
            console.error('SSE connection error:', event);
            
            sseStore.update(state => ({
                ...state,
                connected: false,
                connecting: false,
                error: 'Connection error occurred'
            }));

            // Attempt to reconnect
            this.handleReconnect();
        };

        this.eventSource.onmessage = (event) => {
            this.handleMessage(event);
        };

        // Handle specific event types
        this.eventSource.addEventListener('notification', (event) => {
            this.handleNotification(event);
        });

        this.eventSource.addEventListener('session_update', (event) => {
            this.handleSessionUpdate(event);
        });

        this.eventSource.addEventListener('activity_update', (event) => {
            this.handleActivityUpdate(event);
        });

        this.eventSource.addEventListener('error', (event) => {
            console.error('SSE error event:', event);
        });
    }

    private handleMessage(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            
            sseStore.update(state => ({
                ...state,
                lastMessage: message
            }));

            console.log('SSE message received:', message);
        } catch (error) {
            console.error('Failed to parse SSE message:', error);
        }
    }

    private handleNotification(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const notification: NotificationMessage = message.data;

            // Add notification to store
            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50) // Keep last 50
            }));

            // Show browser notification if permission granted
            this.showBrowserNotification(notification);

            // Play notification sound
            this.playNotificationSound(notification.notification_type);

        } catch (error) {
            console.error('Failed to handle notification:', error);
        }
    }

    private handleSessionUpdate(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const sessionUpdate: SessionUpdateMessage = message.data;

            console.log('Session update received:', sessionUpdate);

            switch (sessionUpdate.action) {
                case 'force_logout':
                    this.handleForceLogout(sessionUpdate);
                    break;
                case 'permission_changed':
                    this.handlePermissionChange(sessionUpdate);
                    break;
                case 'extended':
                    this.handleSessionExtended(sessionUpdate);
                    break;
            }
        } catch (error) {
            console.error('Failed to handle session update:', error);
        }
    }

    private handleActivityUpdate(event: MessageEvent): void {
        try {
            const message: SseMessage = JSON.parse(event.data);
            const activityUpdate: ActivityUpdateMessage = message.data;

            // Create notification for activity update
            const notification: NotificationMessage = {
                title: `Activity ${activityUpdate.update_type}`,
                message: `${activityUpdate.title}: ${activityUpdate.message}`,
                notification_type: activityUpdate.update_type === 'cancelled' ? 'warning' : 'info',
                action_url: `/activities/${activityUpdate.activity_id}`
            };

            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications].slice(0, 50)
            }));

        } catch (error) {
            console.error('Failed to handle activity update:', error);
        }
    }

    private handleForceLogout(sessionUpdate: SessionUpdateMessage): void {
        const notification: NotificationMessage = {
            title: 'Session Terminated',
            message: sessionUpdate.reason || 'Your session has been terminated by an administrator.',
            notification_type: 'error'
        };

        // Show notification
        sseStore.update(state => ({
            ...state,
            notifications: [notification, ...state.notifications]
        }));

        // Disconnect SSE
        this.disconnect();

        // Clear auth and redirect to login
        authStore.set({
            user: null,
            session_id: null,
            expires_at: null,
            loading: false,
            error: 'Session terminated by administrator'
        });

        // Clear localStorage
        if (browser) {
            localStorage.removeItem('session_id');
            localStorage.removeItem('user');
            localStorage.removeItem('expires_at');
        }

        // Redirect to login with message
        goto('/login?message=session_terminated');
    }

    private handlePermissionChange(sessionUpdate: SessionUpdateMessage): void {
        const notification: NotificationMessage = {
            title: 'Permissions Updated',
            message: sessionUpdate.reason || 'Your account permissions have been updated.',
            notification_type: 'info'
        };

        sseStore.update(state => ({
            ...state,
            notifications: [notification, ...state.notifications]
        }));

        // Refresh user data to get updated permissions
        this.refreshUserData();
    }

    private handleSessionExtended(sessionUpdate: SessionUpdateMessage): void {
        if (sessionUpdate.new_expires_at) {
            // Update auth store with new expiry
            authStore.update(state => ({
                ...state,
                expires_at: sessionUpdate.new_expires_at!
            }));

            if (browser) {
                localStorage.setItem('expires_at', sessionUpdate.new_expires_at);
            }

            const notification: NotificationMessage = {
                title: 'Session Extended',
                message: 'Your session has been extended.',
                notification_type: 'success'
            };

            sseStore.update(state => ({
                ...state,
                notifications: [notification, ...state.notifications]
            }));
        }
    }

    private async refreshUserData(): Promise<void> {
        try {
            const response = await fetch('/api/auth/me', {
                credentials: 'include'
            });

            if (response.ok) {
                const user = await response.json();
                authStore.update(state => ({
                    ...state,
                    user
                }));

                if (browser) {
                    localStorage.setItem('user', JSON.stringify(user));
                }
            }
        } catch (error) {
            console.error('Failed to refresh user data:', error);
        }
    }

    private showBrowserNotification(notification: NotificationMessage): void {
        if (!browser || Notification.permission !== 'granted') {
            return;
        }

        new Notification(notification.title, {
            body: notification.message,
            icon: '/favicon.ico',
            tag: 'trackivity-notification'
        });
    }

    private playNotificationSound(type: string): void {
        if (!browser) return;

        try {
            const audio = new Audio();
            
            switch (type) {
                case 'error':
                    audio.src = '/sounds/error.mp3';
                    break;
                case 'warning':
                    audio.src = '/sounds/warning.mp3';
                    break;
                case 'success':
                    audio.src = '/sounds/success.mp3';
                    break;
                default:
                    audio.src = '/sounds/notification.mp3';
            }

            audio.volume = 0.3;
            audio.play().catch(() => {
                // Ignore autoplay errors
            });
        } catch (error) {
            // Ignore audio errors
        }
    }

    private handleReconnect(): void {
        if (this.reconnectAttempt >= this.maxReconnectAttempts) {
            console.error('Max reconnection attempts reached');
            sseStore.update(state => ({
                ...state,
                error: 'Failed to reconnect after multiple attempts'
            }));
            return;
        }

        this.reconnectAttempt++;
        const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempt), 30000);

        console.log(`Attempting to reconnect in ${delay}ms (attempt ${this.reconnectAttempt})`);

        this.reconnectTimeout = setTimeout(() => {
            const auth = get(authStore);
            if (auth.session_id) {
                this.connect(auth.session_id);
            }
        }, delay);
    }

    disconnect(): void {
        if (this.reconnectTimeout) {
            clearTimeout(this.reconnectTimeout);
            this.reconnectTimeout = null;
        }

        this.cleanup();

        sseStore.update(state => ({
            ...state,
            connected: false,
            connecting: false
        }));
    }

    private cleanup(): void {
        if (this.eventSource) {
            this.eventSource.close();
            this.eventSource = null;
        }
    }

    // Clear all notifications
    clearNotifications(): void {
        sseStore.update(state => ({
            ...state,
            notifications: []
        }));
    }

    // Remove specific notification
    removeNotification(index: number): void {
        sseStore.update(state => ({
            ...state,
            notifications: state.notifications.filter((_, i) => i !== index)
        }));
    }

    // Mark notification as read (remove if expired)
    markAsRead(index: number): void {
        sseStore.update(state => {
            const notification = state.notifications[index];
            if (notification && notification.expires_at && new Date(notification.expires_at) <= new Date()) {
                return {
                    ...state,
                    notifications: state.notifications.filter((_, i) => i !== index)
                };
            }
            return state;
        });
    }

    // Request notification permission
    async requestNotificationPermission(): Promise<boolean> {
        if (!browser || !('Notification' in window)) {
            return false;
        }

        if (Notification.permission === 'granted') {
            return true;
        }

        if (Notification.permission === 'denied') {
            return false;
        }

        const permission = await Notification.requestPermission();
        return permission === 'granted';
    }
}

// Create singleton instance
export const sseService = new SseService();

// Auto-connect when authenticated
if (browser) {
    authStore.subscribe(auth => {
        if (auth.session_id && !get(sseStore).connected) {
            sseService.connect(auth.session_id);
        } else if (!auth.session_id && get(sseStore).connected) {
            sseService.disconnect();
        }
    });

    // Request notification permission on first load
    sseService.requestNotificationPermission();
}

// Cleanup on page unload
if (browser) {
    window.addEventListener('beforeunload', () => {
        sseService.disconnect();
    });
}