import { z } from 'zod';
import { AdminLevel } from '$lib/types/admin';

export const loginSchema = z.object({
	email: z
		.string()
		.min(1, 'กรุณาใส่อีเมล')
		.email('รูปแบบอีเมลไม่ถูกต้อง'),
	password: z
		.string()
		.min(1, 'กรุณาใส่รหัสผ่าน')
		.min(6, 'รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร')
});

export const registerSchema = z.object({
	email: z
		.string()
		.min(1, 'กรุณาใส่อีเมล')
		.email('รูปแบบอีเมลไม่ถูกต้อง'),
	password: z
		.string()
		.min(6, 'รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร')
		.regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/, 'รหัสผ่านต้องมีตัวพิมพ์เล็ก พิมพ์ใหญ่ และตัวเลข'),
	confirmPassword: z
		.string()
		.min(1, 'กรุณายืนยันรหัสผ่าน'),
	name: z
		.string()
		.min(1, 'กรุณาใส่ชื่อ')
		.min(2, 'ชื่อต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(100, 'ชื่อต้องไม่เกิน 100 ตัวอักษร'),
	admin_level: z
		.nativeEnum(AdminLevel)
		.optional(),
	faculty_id: z
		.number()
		.positive('กรุณาเลือกคณะ')
		.optional()
}).refine(data => data.password === data.confirmPassword, {
	message: 'รหัสผ่านไม่ตรงกัน',
	path: ['confirmPassword']
});

export const adminCreateSchema = z.object({
	email: z
		.string()
		.min(1, 'กรุณาใส่อีเมล')
		.email('รูปแบบอีเมลไม่ถูกต้อง'),
	name: z
		.string()
		.min(1, 'กรุณาใส่ชื่อ')
		.min(2, 'ชื่อต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(100, 'ชื่อต้องไม่เกิน 100 ตัวอักษร'),
	admin_level: z
		.nativeEnum(AdminLevel, {
			message: 'กรุณาเลือกระดับแอดมิน'
		}),
	faculty_id: z
		.number()
		.positive('กรุณาเลือกคณะ')
		.optional(),
	permissions: z
		.array(z.string())
		.default([])
}).refine(data => {
	// FacultyAdmin ต้องมี faculty_id
	if (data.admin_level === AdminLevel.FacultyAdmin && !data.faculty_id) {
		return false;
	}
	return true;
}, {
	message: 'แอดมินระดับคณะต้องระบุคณะ',
	path: ['faculty_id']
});

export const adminUpdateSchema = z.object({
	id: z.number().positive('ID ไม่ถูกต้อง'),
	email: z
		.string()
		.min(1, 'กรุณาใส่อีเมล')
		.email('รูปแบบอีเมลไม่ถูกต้อง')
		.optional(),
	name: z
		.string()
		.min(2, 'ชื่อต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(100, 'ชื่อต้องไม่เกิน 100 ตัวอักษร')
		.optional(),
	admin_level: z
		.nativeEnum(AdminLevel)
		.optional(),
	faculty_id: z
		.number()
		.positive('กรุณาเลือกคณะ')
		.optional(),
	permissions: z
		.array(z.string())
		.optional()
});

export type LoginFormData = z.infer<typeof loginSchema>;
export type RegisterFormData = z.infer<typeof registerSchema>;
export type AdminCreateFormData = z.infer<typeof adminCreateSchema>;
export type AdminUpdateFormData = z.infer<typeof adminUpdateSchema>;