import type { ColumnDef } from "@tanstack/table-core";
import type { Activity } from "$lib/types/activity.js";
import { renderComponent, renderSnippet } from "$lib/components/ui/data-table/index.js";
import { Badge } from "$lib/components/ui/badge/index.js";
import { Button } from "$lib/components/ui/button/index.js";
import { createRawSnippet } from "svelte";

export const activityColumns: ColumnDef<Activity>[] = [
	{
		accessorKey: "name",
		header: "ชื่อกิจกรรม",
		cell: ({ row }) => {
			const activity = row.original;
			return renderSnippet(
				createRawSnippet(() => ({
					render: () => `
						<div class="font-medium">
							${activity.name}
						</div>
					`
				}))
			);
		}
	},
	{
		accessorKey: "organizer",
		header: "หน่วยงานที่จัด",
		cell: ({ row }) => {
			const activity = row.original;
			return renderSnippet(
				createRawSnippet(() => ({
					render: () => `
						<div class="max-w-[200px]">
							<div class="font-medium text-sm">${activity.organizer}</div>
							<div class="text-xs text-muted-foreground">${activity.organizerType}</div>
						</div>
					`
				}))
			);
		}
	},
	{
		accessorKey: "participantCount",
		header: () => renderSnippet(
			createRawSnippet(() => ({
				render: () => '<div class="text-center">จำนวนคน</div>'
			}))
		),
		cell: ({ row }) => {
			const count = row.getValue("participantCount") as number;
			return renderSnippet(
				createRawSnippet(() => ({
					render: () => `
						<div class="text-center font-medium">
							${count.toLocaleString('th-TH')}
						</div>
					`
				}))
			);
		}
	},
	{
		accessorKey: "status",
		header: "สถานะ",
		cell: ({ row }) => {
			const status = row.getValue("status") as Activity['status'];
			const variant = status === 'เสร็จสิ้น' ? 'default' : 
			              status === 'กำลังดำเนินการ' ? 'secondary' : 'outline';
			
			return renderSnippet(
				createRawSnippet(() => ({
					render: () => `
						<div class="flex items-center gap-2">
							<div class="w-2 h-2 rounded-full ${
								status === 'เสร็จสิ้น' ? 'bg-green-500' :
								status === 'กำลังดำเนินการ' ? 'bg-yellow-500' : 'bg-gray-400'
							}"></div>
							<span class="text-sm font-medium">${status}</span>
						</div>
					`
				}))
			);
		}
	},
	{
		accessorKey: "createdAt",
		header: "วันที่สร้าง",
		cell: ({ row }) => {
			const date = new Date(row.getValue("createdAt") as string);
			return renderSnippet(
				createRawSnippet(() => ({
					render: () => `
						<div class="text-sm text-muted-foreground">
							${date.toLocaleDateString('th-TH', {
								year: 'numeric',
								month: 'short',
								day: 'numeric'
							})}
						</div>
					`
				}))
			);
		}
	},
	{
		id: "actions",
		header: "",
		cell: ({ row }) => {
			return renderSnippet(
				createRawSnippet(() => ({
					render: () => `
						<div class="flex items-center gap-2">
							<button class="px-3 py-1 text-xs bg-primary text-primary-foreground rounded-md hover:bg-primary/80 transition-colors">
								ดูรายละเอียด
							</button>
							<button class="px-3 py-1 text-xs border border-border rounded-md hover:bg-accent transition-colors">
								แก้ไข
							</button>
						</div>
					`
				}))
			);
		}
	}
];