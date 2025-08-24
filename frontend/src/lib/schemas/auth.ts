import { z } from 'zod';
import { AdminLevel } from '$lib/types/admin';

export const loginSchema = z.object({
	student_id: z
		.string()
		.min(1, 'กรุณาใส่รหัสนักศึกษา')
		.regex(/^[0-9]+$/, 'รหัสนักศึกษาต้องเป็นตัวเลขเท่านั้น')
		.min(8, 'รหัสนักศึกษาต้องมีอย่างน้อย 8 หลัก'),
	password: z
		.string()
		.min(1, 'กรุณาใส่รหัสผ่าน')
		.min(6, 'รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร'),
	remember_me: z
		.boolean()
		.optional()
		.default(false)
});

// Admin login schema (uses email instead of student_id)
export const adminLoginSchema = z.object({
	email: z
		.string()
		.min(1, 'กรุณาใส่อีเมล')
		.email('รูปแบบอีเมลไม่ถูกต้อง'),
	password: z
		.string()
		.min(1, 'กรุณาใส่รหัสผ่าน')
		.min(6, 'รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร'),
	remember_me: z
		.boolean()
		.optional()
		.default(false)
});

// Define Thai prefix options (all 9 options for admin)
export const PrefixOptions = [
	{ value: 'Mr', label: 'นาย' },
	{ value: 'Mrs', label: 'นาง' },
	{ value: 'Miss', label: 'นางสาว' },
	{ value: 'Dr', label: 'ดร.' },
	{ value: 'Professor', label: 'ศาสตราจารย์' },
	{ value: 'AssociateProfessor', label: 'รองศาสตราจารย์' },
	{ value: 'AssistantProfessor', label: 'ผู้ช่วยศาสตราจารย์' },
	{ value: 'Lecturer', label: 'อาจารย์' },
	{ value: 'Generic', label: 'คุณ' }
] as const;

// Basic prefix options for student registration (only 3 basic ones)
export const BasicPrefixOptions = [
	{ value: 'Mr', label: 'นาย' },
	{ value: 'Mrs', label: 'นาง' },
	{ value: 'Miss', label: 'นางสาว' }
] as const;

// Student registration schema with prefix support
export const registerSchema = z.object({
	prefix: z
		.string()
		.min(1, 'กรุณาเลือกคำนำหน้า')
		.refine(val => BasicPrefixOptions.some(option => option.value === val), {
			message: 'กรุณาเลือกคำนำหน้าที่ถูกต้อง'
		}),
	first_name: z
		.string()
		.min(1, 'กรุณาใส่ชื่อจริง')
		.min(2, 'ชื่อต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(50, 'ชื่อต้องไม่เกิน 50 ตัวอักษร'),
	last_name: z
		.string()
		.min(1, 'กรุณาใส่นามสกุล')
		.min(2, 'นามสกุลต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(50, 'นามสกุลต้องไม่เกิน 50 ตัวอักษร'),
	student_id: z
		.string()
		.min(1, 'กรุณาใส่รหัสนักเรียน')
		.regex(/^[0-9]+$/, 'รหัสนักเรียนต้องเป็นตัวเลขเท่านั้น')
		.min(8, 'รหัสนักเรียนต้องมีอย่างน้อย 8 หลัก'),
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
	faculty_id: z
		.string()
		.min(1, 'กรุณาเลือกคณะ'),
	department_id: z
		.string()
		.min(1, 'กรุณาเลือกสาขาวิชา')
}).refine(data => data.password === data.confirmPassword, {
	message: 'รหัสผ่านไม่ตรงกัน',
	path: ['confirmPassword']
});

export const adminCreateSchema = z.object({
	prefix: z
		.string()
		.min(1, 'กรุณาเลือกคำนำหน้า')
		.refine(val => PrefixOptions.some(option => option.value === val), {
			message: 'กรุณาเลือกคำนำหน้าที่ถูกต้อง'
		}),
	first_name: z
		.string()
		.min(1, 'กรุณาใส่ชื่อจริง')
		.min(2, 'ชื่อต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(50, 'ชื่อต้องไม่เกิน 50 ตัวอักษร'),
	last_name: z
		.string()
		.min(1, 'กรุณาใส่นามสกุล')
		.min(2, 'นามสกุลต้องมีอย่างน้อย 2 ตัวอักษร')
		.max(50, 'นามสกุลต้องไม่เกิน 50 ตัวอักษร'),
	email: z
		.string()
		.min(1, 'กรุณาใส่อีเมล')
		.email('รูปแบบอีเมลไม่ถูกต้อง'),
	password: z
		.string()
		.min(1, 'กรุณาใส่รหัสผ่าน')
		.min(6, 'รหัสผ่านต้องมีอย่างน้อย 6 ตัวอักษร')
		.regex(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)/, 'รหัสผ่านต้องมีตัวพิมพ์เล็ก พิมพ์ใหญ่ และตัวเลข'),
	admin_level: z
		.nativeEnum(AdminLevel, {
			message: 'กรุณาเลือกระดับแอดมิน'
		}),
	faculty_id: z
		.string()
		.optional()
		.refine(val => !val || val === '' || /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(val), {
			message: 'รูปแบบ Faculty ID ไม่ถูกต้อง'
		}),
	permissions: z
		.array(z.string())
		.default([])
}).refine(data => {
	// FacultyAdmin ต้องมี faculty_id
	if (data.admin_level === AdminLevel.FacultyAdmin && (!data.faculty_id || data.faculty_id === '')) {
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
export type AdminLoginFormData = z.infer<typeof adminLoginSchema>;
export type RegisterFormData = z.infer<typeof registerSchema>;
export type AdminCreateFormData = z.infer<typeof adminCreateSchema>;
export type AdminUpdateFormData = z.infer<typeof adminUpdateSchema>;