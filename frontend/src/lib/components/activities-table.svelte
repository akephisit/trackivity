<script lang="ts">
	import {
		getCoreRowModel,
		getFilteredRowModel,
		getPaginationRowModel,
		getSortedRowModel,
		type ColumnFiltersState,
		type PaginationState,
		type SortingState
	} from "@tanstack/table-core";
	
	import { createSvelteTable, FlexRender } from "$lib/components/ui/data-table/index.js";
	import * as Table from "$lib/components/ui/table/index.js";
	import * as Card from "$lib/components/ui/card/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Input } from "$lib/components/ui/input/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { activityColumns } from "./activities-table-columns.js";
	import type { Activity } from "$lib/types/activity.js";
	
	import ChevronLeftIcon from "@tabler/icons-svelte/icons/chevron-left";
	import ChevronRightIcon from "@tabler/icons-svelte/icons/chevron-right";
	import ChevronsLeftIcon from "@tabler/icons-svelte/icons/chevrons-left";
	import ChevronsRightIcon from "@tabler/icons-svelte/icons/chevrons-right";
	import PlusIcon from "@tabler/icons-svelte/icons/plus";
	import SearchIcon from "@tabler/icons-svelte/icons/search";

	let { activities }: { activities: Activity[] } = $props();
	
	let pagination = $state<PaginationState>({ pageIndex: 0, pageSize: 10 });
	let sorting = $state<SortingState>([]);
	let columnFilters = $state<ColumnFiltersState>([]);
	let globalFilter = $state("");

	const table = createSvelteTable({
		get data() { return activities; },
		columns: activityColumns,
		state: {
			get pagination() { return pagination; },
			get sorting() { return sorting; },
			get columnFilters() { return columnFilters; },
			get globalFilter() { return globalFilter; }
		},
		getCoreRowModel: getCoreRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		getSortedRowModel: getSortedRowModel(),
		getFilteredRowModel: getFilteredRowModel(),
		onPaginationChange: (updater) => {
			pagination = typeof updater === "function" ? updater(pagination) : updater;
		},
		onSortingChange: (updater) => {
			sorting = typeof updater === "function" ? updater(sorting) : updater;
		},
		onColumnFiltersChange: (updater) => {
			columnFilters = typeof updater === "function" ? updater(columnFilters) : updater;
		},
		onGlobalFilterChange: (updater) => {
			globalFilter = typeof updater === "function" ? updater(globalFilter) : updater;
		}
	});

	// สถิติสรุป
	const stats = $derived({
		total: activities.length,
		completed: activities.filter(a => a.status === 'เสร็จสิ้น').length,
		inProgress: activities.filter(a => a.status === 'กำลังดำเนินการ').length,
		pending: activities.filter(a => a.status === 'รอดำเนินการ').length,
		totalParticipants: activities.reduce((sum, a) => sum + a.participantCount, 0)
	});
</script>

<div class="space-y-6">
	<!-- สถิติสรุป -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
		<Card.Root>
			<Card.Content class="p-4">
				<div class="text-2xl font-bold text-primary">{stats.total}</div>
				<div class="text-sm text-muted-foreground">กิจกรรมทั้งหมด</div>
			</Card.Content>
		</Card.Root>
		
		<Card.Root>
			<Card.Content class="p-4">
				<div class="text-2xl font-bold text-green-600">{stats.completed}</div>
				<div class="text-sm text-muted-foreground">เสร็จสิ้นแล้ว</div>
			</Card.Content>
		</Card.Root>
		
		<Card.Root>
			<Card.Content class="p-4">
				<div class="text-2xl font-bold text-yellow-600">{stats.inProgress}</div>
				<div class="text-sm text-muted-foreground">กำลังดำเนินการ</div>
			</Card.Content>
		</Card.Root>
		
		<Card.Root>
			<Card.Content class="p-4">
				<div class="text-2xl font-bold text-gray-600">{stats.pending}</div>
				<div class="text-sm text-muted-foreground">รอดำเนินการ</div>
			</Card.Content>
		</Card.Root>
		
		<Card.Root>
			<Card.Content class="p-4">
				<div class="text-2xl font-bold text-blue-600">{stats.totalParticipants.toLocaleString('th-TH')}</div>
				<div class="text-sm text-muted-foreground">ผู้เข้าร่วมทั้งหมด</div>
			</Card.Content>
		</Card.Root>
	</div>

	<!-- ตารางกิจกรรม -->
	<Card.Root>
		<Card.Header>
			<div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
				<div>
					<Card.Title>รายการกิจกรรม</Card.Title>
					<Card.Description>จัดการและติดตามกิจกรรมทั้งหมดของมหาวิทยาลัย</Card.Description>
				</div>
				<Button class="flex items-center gap-2">
					<PlusIcon class="w-4 h-4" />
					เพิ่มกิจกรรมใหม่
				</Button>
			</div>
		</Card.Header>
		
		<Card.Content class="space-y-4">
			<!-- แถบค้นหาและฟิลเตอร์ -->
			<div class="flex flex-col sm:flex-row gap-4">
				<div class="relative flex-1">
					<SearchIcon class="absolute left-3 w-4 h-4 text-muted-foreground top-1/2 transform -translate-y-1/2" />
					<Input
						placeholder="ค้นหากิจกรรม..."
						bind:value={globalFilter}
						class="pl-10"
					/>
				</div>
				<div class="flex gap-2">
					<Badge variant="outline" class="cursor-pointer hover:bg-accent">
						ทั้งหมด {stats.total}
					</Badge>
					<Badge variant="outline" class="cursor-pointer hover:bg-accent">
						เสร็จสิ้น {stats.completed}
					</Badge>
					<Badge variant="outline" class="cursor-pointer hover:bg-accent">
						กำลังดำเนินการ {stats.inProgress}
					</Badge>
				</div>
			</div>

			<!-- ตาราง -->
			<div class="rounded-md border">
				<Table.Root>
					<Table.Header>
						{#each table.getHeaderGroups() as headerGroup}
							<Table.Row>
								{#each headerGroup.headers as header}
									<Table.Head class="font-semibold">
										{#if !header.isPlaceholder}
											<FlexRender
												content={header.column.columnDef.header}
												context={header.getContext()}
											/>
										{/if}
									</Table.Head>
								{/each}
							</Table.Row>
						{/each}
					</Table.Header>
					<Table.Body>
						{#each table.getRowModel().rows as row}
							<Table.Row class="hover:bg-muted/50">
								{#each row.getVisibleCells() as cell}
									<Table.Cell class="py-4">
										<FlexRender
											content={cell.column.columnDef.cell}
											context={cell.getContext()}
										/>
									</Table.Cell>
								{/each}
							</Table.Row>
						{:else}
							<Table.Row>
								<Table.Cell colspan={activityColumns.length} class="h-24 text-center text-muted-foreground">
									ไม่พบข้อมูลกิจกรรม
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</div>

			<!-- การแบ่งหน้า -->
			<div class="flex flex-col sm:flex-row items-center justify-between gap-4">
				<div class="text-sm text-muted-foreground">
					แสดง {table.getRowModel().rows.length} จาก {activities.length} รายการ
				</div>
				
				<div class="flex items-center gap-2">
					<Button
						variant="outline"
						size="sm"
						onclick={() => table.setPageIndex(0)}
						disabled={!table.getCanPreviousPage()}
					>
						<ChevronsLeftIcon class="w-4 h-4" />
					</Button>
					
					<Button
						variant="outline"
						size="sm"
						onclick={() => table.previousPage()}
						disabled={!table.getCanPreviousPage()}
					>
						<ChevronLeftIcon class="w-4 h-4" />
					</Button>
					
					<div class="flex items-center gap-1 text-sm">
						<span>หน้า</span>
						<strong>
							{table.getState().pagination.pageIndex + 1} จาก {table.getPageCount()}
						</strong>
					</div>
					
					<Button
						variant="outline"
						size="sm"
						onclick={() => table.nextPage()}
						disabled={!table.getCanNextPage()}
					>
						<ChevronRightIcon class="w-4 h-4" />
					</Button>
					
					<Button
						variant="outline"
						size="sm"
						onclick={() => table.setPageIndex(table.getPageCount() - 1)}
						disabled={!table.getCanNextPage()}
					>
						<ChevronsRightIcon class="w-4 h-4" />
					</Button>
				</div>
			</div>
		</Card.Content>
	</Card.Root>
</div>