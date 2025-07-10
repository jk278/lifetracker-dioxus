//! # 笔记管理命令
//!
//! 提供笔记功能的所有后端API命令

use crate::errors::AppError;
use crate::storage::models::{Note, NoteQuery, NoteSortBy, NoteStats, NoteUpdate, SortOrder};
use crate::tauri_commands::AppState;
use chrono::{DateTime, Local};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{command, State};
use uuid::Uuid;

// ========== 数据传输对象 (DTOs) ==========

/// 笔记数据传输对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteDto {
    pub id: String,
    pub title: String,
    pub content: String,
    pub mood: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

/// 创建笔记请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub mood: Option<String>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
}

/// 更新笔记请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub mood: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
}

/// 笔记查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesQueryRequest {
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub mood: Option<String>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 笔记统计 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesStatsDto {
    pub total_notes: i64,
    pub favorite_notes: i64,
    pub archived_notes: i64,
    pub notes_this_week: i64,
    pub notes_this_month: i64,
    pub most_used_tags: Vec<TagStatsDto>,
    pub mood_distribution: Vec<MoodStatsDto>,
    pub daily_notes_trend: Vec<DailyNoteStatsDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStatsDto {
    pub tag: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodStatsDto {
    pub mood: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyNoteStatsDto {
    pub date: String,
    pub count: i64,
}

// ========== 辅助函数 ==========

/// 将 Note 模型转换为 NoteDto
fn note_to_dto(note: Note) -> NoteDto {
    NoteDto {
        id: note.id.to_string(),
        title: note.title,
        content: note.content,
        mood: note.mood,
        tags: note.tags,
        is_favorite: note.is_favorite,
        is_archived: note.is_archived,
        created_at: note.created_at,
        updated_at: note.updated_at,
    }
}

/// 将 NoteStats 模型转换为 NotesStatsDto
fn stats_to_dto(stats: NoteStats) -> NotesStatsDto {
    NotesStatsDto {
        total_notes: stats.total_notes,
        favorite_notes: stats.favorite_notes,
        archived_notes: stats.archived_notes,
        notes_this_week: stats.notes_this_week,
        notes_this_month: stats.notes_this_month,
        most_used_tags: stats
            .most_used_tags
            .into_iter()
            .map(|tag| TagStatsDto {
                tag: tag.tag,
                count: tag.count,
                percentage: tag.percentage,
            })
            .collect(),
        mood_distribution: stats
            .mood_distribution
            .into_iter()
            .map(|mood| MoodStatsDto {
                mood: mood.mood,
                count: mood.count,
                percentage: mood.percentage,
            })
            .collect(),
        daily_notes_trend: stats
            .daily_notes_trend
            .into_iter()
            .map(|trend| DailyNoteStatsDto {
                date: trend.date.to_string(),
                count: trend.count,
            })
            .collect(),
    }
}

// ========== Tauri 命令 ==========

/// 获取所有笔记
#[command]
pub async fn get_notes(
    query: Option<NotesQueryRequest>,
    state: State<'_, AppState>,
) -> Result<Vec<NoteDto>, String> {
    info!("获取笔记列表");

    let storage = &state.storage;
    let note_query = if let Some(q) = query {
        NoteQuery {
            search: q.search,
            tags: q.tags,
            mood: q.mood,
            is_favorite: q.is_favorite,
            is_archived: q.is_archived,
            created_from: q.start_date.as_ref().and_then(|d| d.parse().ok()),
            created_to: q.end_date.as_ref().and_then(|d| d.parse().ok()),
            sort_by: Some(NoteSortBy::UpdatedAt),
            sort_order: Some(SortOrder::Desc),
            offset: q.offset,
            limit: q.limit,
        }
    } else {
        NoteQuery::default()
    };

    match storage.get_notes(&note_query).await {
        Ok(notes) => {
            debug!("获取到 {} 条笔记", notes.len());
            Ok(notes.into_iter().map(note_to_dto).collect())
        }
        Err(e) => {
            error!("获取笔记失败: {}", e);
            Err(format!("获取笔记失败: {}", e))
        }
    }
}

/// 根据ID获取笔记
#[command]
pub async fn get_note_by_id(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<NoteDto, String> {
    info!("获取笔记: {}", note_id);

    let storage = &state.storage;
    let uuid = Uuid::parse_str(&note_id).map_err(|e| format!("无效的笔记ID: {}", e))?;

    match storage.get_note_by_id(uuid).await {
        Ok(Some(note)) => {
            debug!("找到笔记: {}", note.title);
            Ok(note_to_dto(note))
        }
        Ok(None) => {
            error!("笔记不存在: {}", note_id);
            Err(format!("笔记不存在: {}", note_id))
        }
        Err(e) => {
            error!("获取笔记失败: {}", e);
            Err(format!("获取笔记失败: {}", e))
        }
    }
}

/// 创建笔记
#[command]
pub async fn create_note(
    request: CreateNoteRequest,
    state: State<'_, AppState>,
) -> Result<NoteDto, String> {
    info!("创建笔记: {}", request.title);

    let storage = &state.storage;
    let now = Local::now();

    let note = Note {
        id: Uuid::new_v4(),
        title: request.title,
        content: request.content,
        mood: request.mood,
        tags: request.tags.unwrap_or_default(),
        is_favorite: request.is_favorite.unwrap_or(false),
        is_archived: false,
        created_at: now,
        updated_at: now,
    };

    match storage.insert_note(&note).await {
        Ok(_) => {
            debug!("笔记创建成功: {}", note.title);
            Ok(note_to_dto(note))
        }
        Err(e) => {
            error!("创建笔记失败: {}", e);
            Err(format!("创建笔记失败: {}", e))
        }
    }
}

/// 更新笔记
#[command]
pub async fn update_note(
    note_id: String,
    request: UpdateNoteRequest,
    state: State<'_, AppState>,
) -> Result<NoteDto, String> {
    info!("更新笔记: {}", note_id);

    let storage = &state.storage;
    let uuid = Uuid::parse_str(&note_id).map_err(|e| format!("无效的笔记ID: {}", e))?;

    let update = NoteUpdate {
        title: request.title,
        content: request.content,
        mood: request.mood,
        tags: request.tags,
        is_favorite: request.is_favorite,
        is_archived: request.is_archived,
        updated_at: Local::now(),
    };

    match storage.update_note(uuid, &update).await {
        Ok(_) => {
            debug!("笔记更新成功: {}", note_id);
            // 返回更新后的笔记
            match storage.get_note_by_id(uuid).await {
                Ok(Some(note)) => Ok(note_to_dto(note)),
                Ok(None) => Err("笔记不存在".to_string()),
                Err(e) => Err(format!("获取更新后的笔记失败: {}", e)),
            }
        }
        Err(e) => {
            error!("更新笔记失败: {}", e);
            Err(format!("更新笔记失败: {}", e))
        }
    }
}

/// 删除笔记
#[command]
pub async fn delete_note(note_id: String, state: State<'_, AppState>) -> Result<(), String> {
    info!("删除笔记: {}", note_id);

    let storage = &state.storage;
    let uuid = Uuid::parse_str(&note_id).map_err(|e| format!("无效的笔记ID: {}", e))?;

    match storage.delete_note(uuid).await {
        Ok(_) => {
            debug!("笔记删除成功: {}", note_id);
            Ok(())
        }
        Err(e) => {
            error!("删除笔记失败: {}", e);
            Err(format!("删除笔记失败: {}", e))
        }
    }
}

/// 切换笔记收藏状态
#[command]
pub async fn toggle_note_favorite(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<NoteDto, String> {
    info!("切换笔记收藏状态: {}", note_id);

    let storage = &state.storage;
    let uuid = Uuid::parse_str(&note_id).map_err(|e| format!("无效的笔记ID: {}", e))?;

    // 先获取当前笔记
    let current_note = match storage.get_note_by_id(uuid).await {
        Ok(Some(note)) => note,
        Ok(None) => return Err("笔记不存在".to_string()),
        Err(e) => return Err(format!("获取笔记失败: {}", e)),
    };

    // 切换收藏状态
    let update = NoteUpdate {
        title: None,
        content: None,
        mood: None,
        tags: None,
        is_favorite: Some(!current_note.is_favorite),
        is_archived: None,
        updated_at: Local::now(),
    };

    match storage.update_note(uuid, &update).await {
        Ok(_) => {
            debug!("笔记收藏状态更新成功: {}", note_id);
            // 返回更新后的笔记
            match storage.get_note_by_id(uuid).await {
                Ok(Some(note)) => Ok(note_to_dto(note)),
                Ok(None) => Err("笔记不存在".to_string()),
                Err(e) => Err(format!("获取更新后的笔记失败: {}", e)),
            }
        }
        Err(e) => {
            error!("更新笔记收藏状态失败: {}", e);
            Err(format!("更新笔记收藏状态失败: {}", e))
        }
    }
}

/// 切换笔记归档状态
#[command]
pub async fn toggle_note_archive(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<NoteDto, String> {
    info!("切换笔记归档状态: {}", note_id);

    let storage = &state.storage;
    let uuid = Uuid::parse_str(&note_id).map_err(|e| format!("无效的笔记ID: {}", e))?;

    // 先获取当前笔记
    let current_note = match storage.get_note_by_id(uuid).await {
        Ok(Some(note)) => note,
        Ok(None) => return Err("笔记不存在".to_string()),
        Err(e) => return Err(format!("获取笔记失败: {}", e)),
    };

    // 切换归档状态
    let update = NoteUpdate {
        title: None,
        content: None,
        mood: None,
        tags: None,
        is_favorite: None,
        is_archived: Some(!current_note.is_archived),
        updated_at: Local::now(),
    };

    match storage.update_note(uuid, &update).await {
        Ok(_) => {
            debug!("笔记归档状态更新成功: {}", note_id);
            // 返回更新后的笔记
            match storage.get_note_by_id(uuid).await {
                Ok(Some(note)) => Ok(note_to_dto(note)),
                Ok(None) => Err("笔记不存在".to_string()),
                Err(e) => Err(format!("获取更新后的笔记失败: {}", e)),
            }
        }
        Err(e) => {
            error!("更新笔记归档状态失败: {}", e);
            Err(format!("更新笔记归档状态失败: {}", e))
        }
    }
}

/// 搜索笔记
#[command]
pub async fn search_notes(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<NoteDto>, String> {
    info!("搜索笔记: {}", query);

    let storage = &state.storage;
    let note_query = NoteQuery {
        search: Some(query),
        tags: None,
        mood: None,
        is_favorite: None,
        is_archived: None,
        created_from: None,
        created_to: None,
        sort_by: Some(NoteSortBy::UpdatedAt),
        sort_order: Some(SortOrder::Desc),
        offset: None,
        limit: None,
    };

    match storage.get_notes(&note_query).await {
        Ok(notes) => {
            debug!("搜索到 {} 条笔记", notes.len());
            Ok(notes.into_iter().map(note_to_dto).collect())
        }
        Err(e) => {
            error!("搜索笔记失败: {}", e);
            Err(format!("搜索笔记失败: {}", e))
        }
    }
}

/// 获取笔记统计信息
#[command]
pub async fn get_notes_stats(state: State<'_, AppState>) -> Result<NotesStatsDto, String> {
    info!("获取笔记统计信息");

    let storage = &state.storage;
    match storage.get_notes_stats().await {
        Ok(stats) => {
            debug!("获取笔记统计信息成功");
            Ok(stats_to_dto(stats))
        }
        Err(e) => {
            error!("获取笔记统计信息失败: {}", e);
            Err(format!("获取笔记统计信息失败: {}", e))
        }
    }
}

/// 获取所有笔记标签
#[command]
pub async fn get_note_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    info!("获取笔记标签");

    let storage = &state.storage;
    match storage.get_all_note_tags().await {
        Ok(tags) => {
            debug!("获取到 {} 个标签", tags.len());
            Ok(tags)
        }
        Err(e) => {
            error!("获取标签失败: {}", e);
            Err(format!("获取标签失败: {}", e))
        }
    }
}
