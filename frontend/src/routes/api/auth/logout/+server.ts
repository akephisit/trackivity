import { json, redirect } from '@sveltejs/kit';
import { logout } from '$lib/server/auth';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async (event) => {
	try {
		await logout(event);
		
		return json({ 
			success: true, 
			message: 'ออกจากระบบสำเร็จ' 
		});
	} catch (error) {
		console.error('Logout error:', error);
		
		// ลบ session cookie ในกรณีที่เกิดข้อผิดพลาด
		event.cookies.delete('session_id', { path: '/' });
		
		return json({ 
			success: true, 
			message: 'ออกจากระบบสำเร็จ' 
		});
	}
};