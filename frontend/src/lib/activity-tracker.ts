/**
 * Activity Tracker for Session Management
 * จับการดำเนินการของผู้ใช้เพื่อรักษา session ไว้ (mouse, keyboard, page navigation)
 */

import { browser } from '$app/environment';
import { apiClient } from '$lib/api/client';

class ActivityTracker {
  private lastActivity: number = Date.now();
  private refreshTimer: number | null = null;
  private isTracking: boolean = false;
  private readonly REFRESH_INTERVAL = 30 * 60 * 1000; // 30 minutes
  private readonly ACTIVITY_THRESHOLD = 5 * 60 * 1000; // 5 minutes since last activity

  constructor() {
    if (browser) {
      this.init();
    }
  }

  private init(): void {
    // Track mouse movement
    document.addEventListener('mousemove', this.updateActivity.bind(this), { passive: true });
    
    // Track keyboard activity
    document.addEventListener('keypress', this.updateActivity.bind(this), { passive: true });
    
    // Track page navigation
    document.addEventListener('visibilitychange', this.updateActivity.bind(this), { passive: true });
    
    // Track page focus/blur
    window.addEventListener('focus', this.updateActivity.bind(this), { passive: true });
    
    // Track scroll activity
    document.addEventListener('scroll', this.updateActivity.bind(this), { passive: true });
    
    // Track click events
    document.addEventListener('click', this.updateActivity.bind(this), { passive: true });
  }

  private updateActivity(): void {
    this.lastActivity = Date.now();
    
    if (!this.isTracking) {
      this.startTracking();
    }
  }

  public startTracking(): void {
    if (this.isTracking) return;
    
    this.isTracking = true;
    this.scheduleRefresh();
  }

  public stopTracking(): void {
    this.isTracking = false;
    
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
  }

  private scheduleRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
    }

    this.refreshTimer = window.setTimeout(() => {
      this.checkAndRefresh();
    }, this.REFRESH_INTERVAL);
  }

  private async checkAndRefresh(): Promise<void> {
    if (!this.isTracking) return;

    const timeSinceActivity = Date.now() - this.lastActivity;
    
    // ถ้าไม่มีการใช้งานมากกว่า 5 นาที ให้หยุด refresh
    if (timeSinceActivity > this.ACTIVITY_THRESHOLD) {
      console.log('[ActivityTracker] User inactive, stopping session refresh');
      this.stopTracking();
      return;
    }

    try {
      // Refresh session ถ้ามีการใช้งานอยู่
      await apiClient.refreshSession();
      console.log('[ActivityTracker] Session refreshed successfully');
    } catch (error) {
      console.warn('[ActivityTracker] Failed to refresh session:', error);
      // หยุด tracking ถ้า refresh ไม่สำเร็จ (เช่น session หมดอายุแล้ว)
      this.stopTracking();
    }

    // Schedule next refresh
    this.scheduleRefresh();
  }

  public getLastActivity(): number {
    return this.lastActivity;
  }

  public isUserActive(): boolean {
    const timeSinceActivity = Date.now() - this.lastActivity;
    return timeSinceActivity <= this.ACTIVITY_THRESHOLD;
  }
}

// Export singleton instance
export const activityTracker = new ActivityTracker();