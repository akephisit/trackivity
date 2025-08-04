import type { Activity } from '../types/activity.js';

export const mockActivities: Activity[] = [
	{
		id: 1,
		name: 'งานวันคุ้มครองผู้บริโภค',
		organizer: 'คณะเศรษฐศาสตร์',
		organizerType: 'คณะ',
		participantCount: 150,
		status: 'เสร็จสิ้น',
		createdAt: '2024-01-15T10:00:00Z',
		updatedAt: '2024-01-20T16:30:00Z'
	},
	{
		id: 2,
		name: 'การประชุมสัมมนาวิชาการระดับชาติ',
		organizer: 'คณะวิศวกรรมศาสตร์',
		organizerType: 'คณะ',
		participantCount: 300,
		status: 'กำลังดำเนินการ',
		createdAt: '2024-02-01T09:00:00Z',
		updatedAt: '2024-02-15T14:20:00Z'
	},
	{
		id: 3,
		name: 'งานปฐมนิเทศนักศึกษาใหม่',
		organizer: 'มหาวิทยาลัยเทคโนโลยีพระจอมเกล้าธนบุรี',
		organizerType: 'มหาวิทยาลัย',
		participantCount: 2500,
		status: 'รอดำเนินการ',
		createdAt: '2024-02-20T08:30:00Z',
		updatedAt: '2024-02-20T08:30:00Z'
	},
	{
		id: 4,
		name: 'การแข่งขันกีฬาระหว่างคณะ',
		organizer: 'กองกิจการนักศึกษา',
		organizerType: 'มหาวิทยาลัย',
		participantCount: 800,
		status: 'กำลังดำเนินการ',
		createdAt: '2024-01-25T07:45:00Z',
		updatedAt: '2024-02-10T11:15:00Z'
	},
	{
		id: 5,
		name: 'เวทีแลกเปลี่ยนเรียนรู้นวัตกรรม',
		organizer: 'คณะครุศาสตร์',
		organizerType: 'คณะ',
		participantCount: 120,
		status: 'เสร็จสิ้น',
		createdAt: '2024-01-10T13:20:00Z',
		updatedAt: '2024-01-18T17:00:00Z'
	},
	{
		id: 6,
		name: 'งานแสดงผลงานนักศึกษา',
		organizer: 'คณะศิลปกรรมศาสตร์',
		organizerType: 'คณะ',
		participantCount: 200,
		status: 'รอดำเนินการ',
		createdAt: '2024-02-18T10:30:00Z',
		updatedAt: '2024-02-18T10:30:00Z'
	},
	{
		id: 7,
		name: 'การบรรยายพิเศษด้านเทคโนโลยี',
		organizer: 'คณะเทคโนโลยีสารสนเทศ',
		organizerType: 'คณะ',
		participantCount: 180,
		status: 'กำลังดำเนินการ',
		createdAt: '2024-02-05T12:00:00Z',
		updatedAt: '2024-02-12T09:45:00Z'
	},
	{
		id: 8,
		name: 'งานมหกรรมวิทยาศาสตร์และเทคโนโลยี',
		organizer: 'มหาวิทยาลัยเทคโนโลยีพระจอมเกล้าธนบุรี',
		organizerType: 'มหาวิทยาลัย',
		participantCount: 1200,
		status: 'รอดำเนินการ',
		createdAt: '2024-02-22T15:30:00Z',
		updatedAt: '2024-02-22T15:30:00Z'
	}
];