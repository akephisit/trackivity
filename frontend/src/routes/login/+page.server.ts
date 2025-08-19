import { fail, redirect } from '@sveltejs/kit';
import { superValidate } from 'sveltekit-superforms';
import { zod } from 'sveltekit-superforms/adapters';
import { loginSchema } from '$lib/schemas/auth';
import { api } from '$lib/server/api-client';
import type { Actions, PageServerLoad } from './$types';
 

export const load: PageServerLoad = async (event) => {
    const { cookies, url } = event;
    // ตรวจสอบว่ามี session อยู่แล้วหรือไม่
    const sessionId = cookies.get('session_id');
    if (sessionId) {
        try {
            const response = await api.get(event, `/api/auth/me`);
            if (response.success) {
                const payload = response.data;
                const hasUser = Boolean((payload as any)?.user ?? (payload as any)?.data);
                if (hasUser) {
                    const redirectTo = url.searchParams.get('redirectTo') || '/';
                    throw redirect(303, redirectTo);
                }
            }
        } catch (error) {
            cookies.delete('session_id', { path: '/' });
        }
    }

    const form = await superValidate(zod(loginSchema));
    return { form };
};

export const actions: Actions = {
    default: async (event) => {
        const { request, cookies, url } = event;
        const form = await superValidate(request, zod(loginSchema));

        if (!form.valid) {
            return fail(400, { form });
        }

        try {
            const result = await api.post(event, `/api/auth/login`, {
                student_id: form.data.student_id,
                password: form.data.password,
                remember_me: form.data.remember_me
            });

            if (!result.success) {
                form.errors.student_id = [result.error || 'อีเมลหรือรหัสผ่านไม่ถูกต้อง'];
                return fail(400, { form });
            }

            const payload = result.data as any;
            if (payload?.success && payload?.session) {
                cookies.set('session_id', payload.session.session_id, {
                    path: '/',
                    httpOnly: true,
                    secure: process.env.NODE_ENV === 'production',
                    sameSite: 'lax',
                    maxAge: form.data.remember_me ? 30 * 24 * 60 * 60 : 24 * 60 * 60
                });

                const redirectTo = url.searchParams.get('redirectTo') || '/';
                throw redirect(303, redirectTo);
            } else {
                form.errors.student_id = [payload?.message || 'เกิดข้อผิดพลาดในการเข้าสู่ระบบ'];
                return fail(400, { form });
            }
        } catch (error) {
            if (error && typeof error === 'object' && 'status' in (error as any) && 'location' in (error as any)) {
                throw error as any;
            }
            console.error('Login error:', error);
            form.errors.student_id = ['เกิดข้อผิดพลาดในการเชื่อมต่อ กรุณาลองใหม่อีกครั้ง'];
            return fail(500, { form });
        }
    }
};
