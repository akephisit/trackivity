use chrono::{DateTime, Utc};
use crate::middleware::session::SessionState;
use crate::models::activity::ActivityStatus;
use sqlx::Row;
use tokio::time::{interval, Duration};
use tracing::{info, error, debug};

pub struct ActivityStatusUpdater {
    session_state: SessionState,
}

impl ActivityStatusUpdater {
    pub fn new(session_state: SessionState) -> Self {
        Self { session_state }
    }

    /// เริ่มต้น background task สำหรับอัพเดตสถานะกิจกรรมอัตโนมัติ
    pub async fn start_background_task(self) {
        let mut interval = interval(Duration::from_secs(60)); // อัพเดตทุก 1 นาที
        
        info!("Starting activity status updater background task");
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.update_activity_statuses().await {
                error!("Failed to update activity statuses: {}", e);
            }
        }
    }

    /// อัพเดตสถานะกิจกรรมทั้งหมดตามวันที่และเวลา
    pub async fn update_activity_statuses(&self) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        
        // อัพเดตกิจกรรมที่ยังเป็น 'draft' หรือ 'published' ให้เป็น 'ongoing'
        // เมื่อเวลาปัจจุบันเข้าสู่ช่วงเวลากิจกรรม
        let ongoing_update_result = sqlx::query(
            r#"
            UPDATE activities 
            SET status = 'ongoing', updated_at = NOW()
            WHERE status IN ('draft', 'published')
            AND (
                (start_date::timestamp + start_time_only) AT TIME ZONE 'UTC' <= $1
                AND (end_date::timestamp + end_time_only) AT TIME ZONE 'UTC' > $1
            )
            "#
        )
        .bind(now)
        .execute(&self.session_state.db_pool)
        .await?;

        if ongoing_update_result.rows_affected() > 0 {
            debug!("Updated {} activities to 'ongoing' status", ongoing_update_result.rows_affected());
        }

        // อัพเดตกิจกรรมที่เป็น 'ongoing' ให้เป็น 'completed'
        // เมื่อเวลาปัจจุบันเลยช่วงเวลากิจกรรมแล้ว
        let completed_update_result = sqlx::query(
            r#"
            UPDATE activities 
            SET status = 'completed', updated_at = NOW()
            WHERE status = 'ongoing'
            AND (end_date::timestamp + end_time_only) AT TIME ZONE 'UTC' <= $1
            "#
        )
        .bind(now)
        .execute(&self.session_state.db_pool)
        .await?;

        if completed_update_result.rows_affected() > 0 {
            info!("Updated {} activities to 'completed' status", completed_update_result.rows_affected());
        }

        // อัพเดตกิจกรรมที่ยัง 'draft' หรือ 'published' ให้เป็น 'published' 
        // สำหรับกิจกรรมที่ยังไม่เริ่ม
        let published_update_result = sqlx::query(
            r#"
            UPDATE activities 
            SET status = 'published', updated_at = NOW()
            WHERE status = 'draft'
            AND (start_date::timestamp + start_time_only) AT TIME ZONE 'UTC' > $1
            "#
        )
        .bind(now)
        .execute(&self.session_state.db_pool)
        .await?;

        if published_update_result.rows_affected() > 0 {
            debug!("Updated {} activities to 'published' status", published_update_result.rows_affected());
        }

        let total_updated = ongoing_update_result.rows_affected() 
            + completed_update_result.rows_affected() 
            + published_update_result.rows_affected();

        if total_updated > 0 {
            info!("Activity status update completed: {} activities updated", total_updated);
        }

        Ok(())
    }

    /// ดึงสถิติการอัพเดตสถานะกิจกรรม
    pub async fn get_status_statistics(&self) -> Result<ActivityStatusStats, sqlx::Error> {
        let stats_result = sqlx::query(
            r#"
            SELECT 
                status,
                COUNT(*) as count
            FROM activities
            GROUP BY status
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.session_state.db_pool)
        .await?;

        let mut stats = ActivityStatusStats::default();

        for row in stats_result {
            let status: ActivityStatus = row.get("status");
            let count: i64 = row.get("count");
            
            match status {
                ActivityStatus::Draft => stats.draft_count = count,
                ActivityStatus::Published => stats.published_count = count,
                ActivityStatus::Ongoing => stats.ongoing_count = count,
                ActivityStatus::Completed => stats.completed_count = count,
                ActivityStatus::Cancelled => stats.cancelled_count = count,
            }
        }

        stats.total_count = stats.draft_count + stats.published_count + stats.ongoing_count 
            + stats.completed_count + stats.cancelled_count;

        Ok(stats)
    }

    /// อัพเดตสถานะของกิจกรรมเฉพาะ ID
    pub async fn update_single_activity_status(&self, activity_id: uuid::Uuid) -> Result<ActivityStatus, sqlx::Error> {
        let now = Utc::now();
        
        // ดึงข้อมูลกิจกรรม
        let activity_result = sqlx::query(
            r#"
            SELECT 
                id,
                status,
                (start_date::timestamp + start_time_only) AT TIME ZONE 'UTC' as start_time,
                (end_date::timestamp + end_time_only) AT TIME ZONE 'UTC' as end_time
            FROM activities
            WHERE id = $1
            "#
        )
        .bind(activity_id)
        .fetch_one(&self.session_state.db_pool)
        .await?;

        let current_status: ActivityStatus = activity_result.get("status");
        let start_time: DateTime<Utc> = activity_result.get("start_time");
        let end_time: DateTime<Utc> = activity_result.get("end_time");

        // คำนวณสถานะใหม่
        let new_status = if now < start_time {
            ActivityStatus::Published
        } else if now >= start_time && now <= end_time {
            ActivityStatus::Ongoing
        } else {
            ActivityStatus::Completed
        };

        // อัพเดตถ้าสถานะเปลี่ยน
        if current_status != new_status {
            sqlx::query(
                "UPDATE activities SET status = $1, updated_at = NOW() WHERE id = $2"
            )
            .bind(&new_status)
            .bind(activity_id)
            .execute(&self.session_state.db_pool)
            .await?;
            
            info!("Updated activity {} status from {:?} to {:?}", activity_id, current_status, new_status);
        }

        Ok(new_status)
    }
}

#[derive(Debug, Default)]
pub struct ActivityStatusStats {
    pub total_count: i64,
    pub draft_count: i64,
    pub published_count: i64,
    pub ongoing_count: i64,
    pub completed_count: i64,
    pub cancelled_count: i64,
}