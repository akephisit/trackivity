import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import type { UserPrefix } from "$lib/types/activity";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// Utility function to convert user prefix to Thai text
export function prefixToThai(prefix: UserPrefix): string {
	const prefixMap: Record<UserPrefix, string> = {
		'Mr': 'นาย',
		'Mrs': 'นาง',
		'Miss': 'นางสาว',
		'Dr': 'ดร.',
		'Professor': 'ศาสตราจารย์',
		'AssociateProfessor': 'รองศาสตราจารย์',
		'AssistantProfessor': 'ผู้ช่วยศาสตราจารย์',
		'Lecturer': 'อาจารย์',
		'Generic': 'คุณ'
	};
	return prefixMap[prefix] || prefix;
}

// Utility function to format full name with Thai prefix
export function formatFullNameWithPrefix(prefix: UserPrefix, firstName: string, lastName: string): string {
	return `${prefixToThai(prefix)}${firstName} ${lastName}`;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
