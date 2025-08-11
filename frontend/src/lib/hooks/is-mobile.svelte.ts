import { MediaQuery } from "svelte/reactivity";
import { browser } from '$app/environment';

const DEFAULT_MOBILE_BREAKPOINT = 768;

export class IsMobile extends MediaQuery {
	constructor(breakpoint: number = DEFAULT_MOBILE_BREAKPOINT) {
		super(`(max-width: ${breakpoint - 1}px)`);
	}
}

// Create global instance
export const isMobile = new IsMobile();

// Additional mobile utilities
export function isMobileDevice(): boolean {
	if (!browser) return false;
	return isMobile.current || /Android|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
}

export function isTabletDevice(): boolean {
	if (!browser) return false;
	const width = window.innerWidth;
	return width >= 768 && width < 1024;
}

export function isDesktopDevice(): boolean {
	if (!browser) return true;
	return window.innerWidth >= 1024;
}

export function hasTouch(): boolean {
	if (!browser) return false;
	return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
}
