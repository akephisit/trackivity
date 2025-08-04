export interface Activity {
	id: number;
	name: string;
	organizer: string;
	organizerType: 'คณะ' | 'มหาวิทยาลัย';
	participantCount: number;
	status: 'รอดำเนินการ' | 'กำลังดำเนินการ' | 'เสร็จสิ้น';
	createdAt: string;
	updatedAt: string;
}

export interface ActivityFormData {
	name: string;
	organizer: string;
	organizerType: Activity['organizerType'];
	participantCount: number;
	status: Activity['status'];
}