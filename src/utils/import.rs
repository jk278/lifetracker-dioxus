//! # 数据导入工具模块
//!
//! 提供各种格式的数据导入功能

use chrono::{DateTime, Local, NaiveDateTime};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::core::{
    category::{CategoryColor, CategoryIcon},
    Category,
};
use crate::storage::models::TimeEntry;
use crate::utils::generate_id;
use anyhow::Result;
use uuid::Uuid;

/// 导入格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    Json,
    Csv,
    Xml,
}

impl ImportFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "xml" => Some(Self::Xml),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Xml => "xml",
        }
    }
}

/// 导入选项
#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub format: ImportFormat,
    pub skip_duplicates: bool,
    pub update_existing: bool,
    pub create_missing_categories: bool,
    pub default_category_id: Option<String>,
    pub validate_data: bool,
    pub dry_run: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            format: ImportFormat::Json,
            skip_duplicates: true,
            update_existing: false,
            create_missing_categories: true,
            default_category_id: None,
            validate_data: true,
            dry_run: false,
        }
    }
}

/// 导入结果
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub success: bool,
    pub imported_categories: usize,
    pub imported_entries: usize,
    pub skipped_categories: usize,
    pub skipped_entries: usize,
    pub errors: Vec<ImportError>,
    pub warnings: Vec<String>,
}

impl Default for ImportResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportResult {
    pub fn new() -> Self {
        Self {
            success: true,
            imported_categories: 0,
            imported_entries: 0,
            skipped_categories: 0,
            skipped_entries: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ImportError) {
        self.success = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn total_processed(&self) -> usize {
        self.imported_categories
            + self.imported_entries
            + self.skipped_categories
            + self.skipped_entries
    }
}

/// 导入错误
#[derive(Debug, Clone)]
pub struct ImportError {
    pub line_number: Option<usize>,
    pub field: Option<String>,
    pub message: String,
    pub data: Option<String>,
}

impl ImportError {
    pub fn new(message: String) -> Self {
        Self {
            line_number: None,
            field: None,
            message,
            data: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line_number = Some(line);
        self
    }

    pub fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }

    pub fn with_data(mut self, data: String) -> Self {
        self.data = Some(data);
        self
    }
}

/// CSV记录结构
#[derive(Debug, Clone, Deserialize)]
struct CsvTimeEntry {
    #[serde(rename = "任务名称")]
    task_name: String,
    #[serde(rename = "分类")]
    category: String,
    #[serde(rename = "开始时间")]
    start_time: String,
    #[serde(rename = "结束时间")]
    end_time: Option<String>,
    #[serde(rename = "描述")]
    description: Option<String>,
}

/// 数据导入器
pub struct DataImporter {
    options: ImportOptions,
}

impl DataImporter {
    pub fn new(options: ImportOptions) -> Self {
        Self { options }
    }

    /// 从文件导入数据
    pub fn import_from_file<P: AsRef<Path>>(
        &self,
        file_path: P,
        existing_categories: &[Category],
        existing_entries: &[TimeEntry],
    ) -> Result<(Vec<Category>, Vec<TimeEntry>, ImportResult)> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);

        match self.options.format {
            ImportFormat::Json => {
                self.import_json(&mut reader, existing_categories, existing_entries)
            }
            ImportFormat::Csv => {
                self.import_csv(&mut reader, existing_categories, existing_entries)
            }
            ImportFormat::Xml => {
                self.import_xml(&mut reader, existing_categories, existing_entries)
            }
        }
    }

    /// 导入JSON格式数据
    fn import_json<R: Read>(
        &self,
        reader: &mut R,
        existing_categories: &[Category],
        existing_entries: &[TimeEntry],
    ) -> Result<(Vec<Category>, Vec<TimeEntry>, ImportResult)> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut result = ImportResult::new();

        // 尝试解析为完整的导出数据
        if let Ok(export_data) = serde_json::from_str::<crate::utils::export::ExportData>(&content)
        {
            return self.process_export_data(export_data, existing_categories, existing_entries);
        }

        // 尝试解析为时间记录数组
        if let Ok(entries) = serde_json::from_str::<Vec<TimeEntry>>(&content) {
            let (processed_entries, entry_result) =
                self.process_time_entries(entries, existing_categories, existing_entries);
            result.imported_entries = entry_result.imported_entries;
            result.skipped_entries = entry_result.skipped_entries;
            result.errors.extend(entry_result.errors);
            result.warnings.extend(entry_result.warnings);

            return Ok((Vec::new(), processed_entries, result));
        }

        // 尝试解析为分类数组
        if let Ok(categories) = serde_json::from_str::<Vec<Category>>(&content) {
            let (processed_categories, category_result) =
                self.process_categories(categories, existing_categories);
            result.imported_categories = category_result.imported_categories;
            result.skipped_categories = category_result.skipped_categories;
            result.errors.extend(category_result.errors);
            result.warnings.extend(category_result.warnings);

            return Ok((processed_categories, Vec::new(), result));
        }

        result.add_error(ImportError::new("无法解析JSON数据".to_string()));
        Ok((Vec::new(), Vec::new(), result))
    }

    /// 导入CSV格式数据
    fn import_csv<R: Read>(
        &self,
        reader: &mut R,
        existing_categories: &[Category],
        existing_entries: &[TimeEntry],
    ) -> Result<(Vec<Category>, Vec<TimeEntry>, ImportResult)> {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut result = ImportResult::new();
        let mut new_categories = Vec::new();
        let mut new_entries = Vec::new();
        let mut category_map = HashMap::new();

        // 创建现有分类映射
        for category in existing_categories {
            category_map.insert(category.name.clone(), category.id);
        }

        let mut csv_reader = csv::Reader::from_reader(content.as_bytes());

        for (line_num, record_result) in csv_reader.deserialize::<CsvTimeEntry>().enumerate() {
            let line_number = line_num + 2; // CSV行号从2开始（包含头部）

            match record_result {
                Ok(csv_entry) => {
                    // 验证数据
                    if self.options.validate_data {
                        if let Err(validation_error) = self.validate_csv_entry(&csv_entry) {
                            result.add_error(
                                ImportError::new(validation_error.to_string())
                                    .with_line(line_number),
                            );
                            continue;
                        }
                    }

                    // 处理分类
                    let category_id =
                        if let Some(existing_id) = category_map.get(&csv_entry.category) {
                            *existing_id
                        } else if self.options.create_missing_categories {
                            let new_category_id = generate_id();
                            let new_category = Category {
                                id: new_category_id,
                                name: csv_entry.category.clone(),
                                description: None,
                                color: CategoryColor::Gray, // 默认灰色
                                icon: CategoryIcon::Other,  // 默认图标
                                created_at: Local::now(),
                                updated_at: Local::now(),
                                daily_target: None,
                                weekly_target: None,
                                target_duration: None,
                                is_active: true,
                                sort_order: 0,
                                parent_id: None,
                            };

                            category_map.insert(csv_entry.category.clone(), new_category_id);
                            new_categories.push(new_category);
                            result.imported_categories += 1;

                            new_category_id
                        } else if let Some(ref default_id) = self.options.default_category_id {
                            Uuid::parse_str(default_id).unwrap_or_else(|_| Uuid::new_v4())
                        } else {
                            result.add_error(
                                ImportError::new(format!("未找到分类: {}", csv_entry.category))
                                    .with_line(line_number)
                                    .with_field("分类".to_string()),
                            );
                            continue;
                        };

                    // 解析时间
                    let start_time = match self.parse_datetime(&csv_entry.start_time) {
                        Ok(time) => time,
                        Err(e) => {
                            result.add_error(
                                ImportError::new(format!("无法解析开始时间: {}", e))
                                    .with_line(line_number)
                                    .with_field("开始时间".to_string())
                                    .with_data(csv_entry.start_time.clone()),
                            );
                            continue;
                        }
                    };

                    // 解析结束时间
                    let end_time = if let Some(end_str) = &csv_entry.end_time {
                        if !end_str.trim().is_empty() {
                            Some(self.parse_datetime(end_str)?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // 创建时间记录
                    let time_entry = TimeEntry {
                        id: generate_id(),
                        task_name: csv_entry.task_name,
                        category_id: Some(category_id),
                        start_time,
                        end_time,
                        duration_seconds: if let Some(end) = end_time {
                            end.signed_duration_since(start_time).num_seconds()
                        } else {
                            0
                        },
                        description: csv_entry.description.filter(|s| !s.is_empty()),
                        tags: vec![],
                        created_at: Local::now(),
                        updated_at: Some(Local::now()),
                    };

                    // 检查重复
                    if self.options.skip_duplicates
                        && self.is_duplicate_entry(&time_entry, existing_entries, &new_entries)
                    {
                        result.skipped_entries += 1;
                        result.add_warning(format!(
                            "跳过重复记录: {} (行 {})",
                            time_entry.task_name, line_number
                        ));
                        continue;
                    }

                    new_entries.push(time_entry);
                    result.imported_entries += 1;
                }
                Err(e) => {
                    result.add_error(
                        ImportError::new(format!("CSV解析错误: {}", e)).with_line(line_number),
                    );
                }
            }
        }

        Ok((new_categories, new_entries, result))
    }

    /// 导入XML格式数据（简化实现）
    fn import_xml<R: Read>(
        &self,
        reader: &mut R,
        existing_categories: &[Category],
        existing_entries: &[TimeEntry],
    ) -> Result<(Vec<Category>, Vec<TimeEntry>, ImportResult)> {
        let mut result = ImportResult::new();

        // 读取XML内容
        let mut xml_content = String::new();
        reader.read_to_string(&mut xml_content)?;

        // 检查现有数据
        let existing_category_count = existing_categories.len();
        let existing_entry_count = existing_entries.len();

        log::info!(
            "开始XML导入，现有分类数量: {}, 现有记录数量: {}",
            existing_category_count,
            existing_entry_count
        );

        // 简化的XML解析 - 目前返回未实现错误
        // 在实际实现中，这里应该解析XML并创建Category和TimeEntry对象
        result.add_error(ImportError::new(format!(
            "XML导入暂未实现，内容长度: {} 字符，现有数据: {} 分类, {} 记录",
            xml_content.len(),
            existing_category_count,
            existing_entry_count
        )));

        // 返回空的结果集
        Ok((Vec::new(), Vec::new(), result))
    }

    /// 处理导出数据
    fn process_export_data(
        &self,
        export_data: crate::utils::export::ExportData,
        existing_categories: &[Category],
        existing_entries: &[TimeEntry],
    ) -> Result<(Vec<Category>, Vec<TimeEntry>, ImportResult)> {
        let mut result = ImportResult::new();

        let (categories, category_result) =
            self.process_categories(export_data.categories, existing_categories);

        let (entries, entry_result) = self.process_time_entries(
            export_data.time_entries,
            existing_categories,
            existing_entries,
        );

        result.imported_categories = category_result.imported_categories;
        result.skipped_categories = category_result.skipped_categories;
        result.imported_entries = entry_result.imported_entries;
        result.skipped_entries = entry_result.skipped_entries;
        result.errors.extend(category_result.errors);
        result.errors.extend(entry_result.errors);
        result.warnings.extend(category_result.warnings);
        result.warnings.extend(entry_result.warnings);

        Ok((categories, entries, result))
    }

    /// 处理分类数据
    fn process_categories(
        &self,
        categories: Vec<Category>,
        existing_categories: &[Category],
    ) -> (Vec<Category>, ImportResult) {
        let mut result = ImportResult::new();
        let mut processed_categories = Vec::new();

        let existing_ids: std::collections::HashSet<_> =
            existing_categories.iter().map(|c| &c.id).collect();

        let existing_names: std::collections::HashSet<_> =
            existing_categories.iter().map(|c| &c.name).collect();

        for category in categories {
            // 检查ID重复
            if existing_ids.contains(&category.id) {
                if self.options.skip_duplicates {
                    result.skipped_categories += 1;
                    result.add_warning(format!("跳过重复分类ID: {}", category.id));
                    continue;
                } else if self.options.update_existing {
                    // 更新现有分类（这里简化处理，实际应该通过数据库更新）
                    result.add_warning(format!("更新分类: {}", category.name));
                }
            }

            // 检查名称重复
            if existing_names.contains(&category.name) && self.options.skip_duplicates {
                result.skipped_categories += 1;
                result.add_warning(format!("跳过重复分类名称: {}", category.name));
                continue;
            }

            // 验证分类数据
            if self.options.validate_data {
                if let Err(validation_error) = self.validate_category(&category) {
                    result.add_error(
                        ImportError::new(validation_error.to_string())
                            .with_data(category.name.clone()),
                    );
                    continue;
                }
            }

            processed_categories.push(category);
            result.imported_categories += 1;
        }

        (processed_categories, result)
    }

    /// 处理时间记录数据
    fn process_time_entries(
        &self,
        entries: Vec<TimeEntry>,
        existing_categories: &[Category],
        existing_entries: &[TimeEntry],
    ) -> (Vec<TimeEntry>, ImportResult) {
        let mut result = ImportResult::new();
        let mut processed_entries = Vec::new();

        let existing_ids: std::collections::HashSet<_> =
            existing_entries.iter().map(|e| &e.id).collect();

        let category_ids: std::collections::HashSet<_> =
            existing_categories.iter().map(|c| &c.id).collect();

        for entry in entries {
            // 检查ID重复
            if existing_ids.contains(&entry.id) {
                if self.options.skip_duplicates {
                    result.skipped_entries += 1;
                    result.add_warning(format!("跳过重复记录ID: {}", entry.id));
                    continue;
                } else if self.options.update_existing {
                    result.add_warning(format!("更新记录: {}", entry.task_name));
                }
            }

            // 检查分类是否存在
            if let Some(category_id) = entry.category_id {
                if !category_ids.contains(&category_id) {
                    if let Some(ref default_id) = self.options.default_category_id {
                        result.add_warning(format!("分类 {:?} 不存在，使用默认分类", category_id));
                        let mut updated_entry = entry;
                        if let Ok(default_uuid) = default_id.parse::<Uuid>() {
                            updated_entry.category_id = Some(default_uuid);
                        }
                        processed_entries.push(updated_entry);
                    } else {
                        result.add_error(
                            ImportError::new(format!("分类不存在: {:?}", category_id))
                                .with_data(entry.task_name.clone()),
                        );
                        continue;
                    }
                } else {
                    // 验证时间记录数据
                    if self.options.validate_data {
                        if let Err(validation_error) = self.validate_time_entry(&entry) {
                            result.add_error(
                                ImportError::new(validation_error.to_string())
                                    .with_data(entry.task_name.clone()),
                            );
                            continue;
                        }
                    }

                    // 检查重复
                    if self.options.skip_duplicates
                        && self.is_duplicate_entry(&entry, existing_entries, &processed_entries)
                    {
                        result.skipped_entries += 1;
                        result.add_warning(format!("跳过重复记录: {}", entry.task_name));
                        continue;
                    }

                    processed_entries.push(entry);
                }
            } else {
                // 没有分类ID的处理
                processed_entries.push(entry);
            }

            result.imported_entries += 1;
        }

        (processed_entries, result)
    }

    /// 解析日期时间字符串
    fn parse_datetime(&self, datetime_str: &str) -> Result<DateTime<Local>> {
        // 尝试多种日期时间格式
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d %H:%M",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d %H:%M",
            "%Y/%m/%d",
            "%d/%m/%Y %H:%M:%S",
            "%d/%m/%Y %H:%M",
            "%d/%m/%Y",
        ];

        for format in &formats {
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(datetime_str, format) {
                use chrono::TimeZone;
                let local_time =
                    Local
                        .from_local_datetime(&naive_dt)
                        .single()
                        .ok_or_else(|| {
                            anyhow::Error::msg(format!("无法转换为本地时间: {}", datetime_str))
                        })?;

                return Ok(local_time);
            }
        }

        // 尝试ISO 8601格式
        if let Ok(dt) = datetime_str.parse::<DateTime<Local>>() {
            return Ok(dt);
        }

        Err(anyhow::Error::msg(format!(
            "无法解析日期时间: {}",
            datetime_str
        )))
    }

    /// 验证CSV记录
    fn validate_csv_entry(&self, entry: &CsvTimeEntry) -> Result<()> {
        if entry.task_name.trim().is_empty() {
            return Err(anyhow::Error::msg("任务名称不能为空"));
        }

        if entry.category.trim().is_empty() {
            return Err(anyhow::Error::msg("分类不能为空"));
        }

        if entry.start_time.trim().is_empty() {
            return Err(anyhow::Error::msg("开始时间不能为空"));
        }

        Ok(())
    }

    /// 验证分类
    fn validate_category(&self, category: &Category) -> Result<()> {
        if category.name.trim().is_empty() {
            return Err(anyhow::Error::msg("分类名称不能为空"));
        }

        if category.name.len() > 100 {
            return Err(anyhow::Error::msg("分类名称过长"));
        }

        if !CategoryColor::is_valid_hex(&category.color.to_hex()) {
            return Err(anyhow::Error::msg("颜色格式无效"));
        }

        Ok(())
    }

    /// 验证时间记录
    fn validate_time_entry(&self, entry: &TimeEntry) -> Result<()> {
        if entry.task_name.trim().is_empty() {
            return Err(anyhow::Error::msg("任务名称不能为空"));
        }

        if entry.task_name.len() > 200 {
            return Err(anyhow::Error::msg("任务名称过长"));
        }

        if let Some(end_time) = entry.end_time {
            if end_time <= entry.start_time {
                return Err(anyhow::Error::msg("结束时间必须晚于开始时间"));
            }
        }

        if let Some(ref desc) = entry.description {
            if desc.len() > 1000 {
                return Err(anyhow::Error::msg("描述过长"));
            }
        }

        Ok(())
    }

    /// 检查是否为重复记录
    fn is_duplicate_entry(
        &self,
        entry: &TimeEntry,
        existing_entries: &[TimeEntry],
        new_entries: &[TimeEntry],
    ) -> bool {
        let check_duplicate = |other: &TimeEntry| {
            entry.task_name == other.task_name
                && entry.category_id == other.category_id
                && entry.start_time == other.start_time
                && entry.end_time == other.end_time
        };

        existing_entries.iter().any(check_duplicate) || new_entries.iter().any(check_duplicate)
    }
}

/// 创建默认导入选项
pub fn create_import_options(format: ImportFormat) -> ImportOptions {
    ImportOptions {
        format,
        ..Default::default()
    }
}

/// 检测文件格式
pub fn detect_format<P: AsRef<Path>>(file_path: P) -> Option<ImportFormat> {
    file_path
        .as_ref()
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ImportFormat::from_extension)
}

/// 从JSON文件导入数据
pub fn import_from_json<P: AsRef<Path>>(file_path: P) -> Result<crate::utils::export::ExportData> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let export_data: crate::utils::export::ExportData = serde_json::from_reader(reader)?;
    Ok(export_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::Cursor;

    fn create_test_category() -> Category {
        use crate::core::category::{CategoryColor, CategoryIcon};

        use uuid::Uuid;

        Category {
            id: Uuid::new_v4(),
            name: "工作".to_string(),
            description: Some("工作相关任务".to_string()),
            color: CategoryColor::Red,
            icon: CategoryIcon::Work,
            created_at: Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            updated_at: Local.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap(),
            daily_target: None,
            weekly_target: None,
            target_duration: None,
            is_active: true,
            sort_order: 0,
            parent_id: None,
        }
    }

    fn create_test_entry() -> TimeEntry {
        use uuid::Uuid;

        TimeEntry {
            id: Uuid::new_v4(),
            task_name: "编程".to_string(),
            category_id: Some(Uuid::new_v4()),
            start_time: Local.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
            end_time: Some(Local.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap()),
            duration_seconds: 3600,
            description: Some("学习Rust".to_string()),
            tags: vec![],
            created_at: Local.with_ymd_and_hms(2023, 1, 1, 9, 0, 0).unwrap(),
            updated_at: Some(Local.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap()),
        }
    }

    #[test]
    fn test_import_format_from_extension() {
        assert_eq!(
            ImportFormat::from_extension("json"),
            Some(ImportFormat::Json)
        );
        assert_eq!(ImportFormat::from_extension("CSV"), Some(ImportFormat::Csv));
        assert_eq!(ImportFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format("test.json"), Some(ImportFormat::Json));
        assert_eq!(detect_format("test.csv"), Some(ImportFormat::Csv));
        assert_eq!(detect_format("test.txt"), None);
    }

    #[test]
    fn test_import_json() {
        let entries = vec![create_test_entry()];
        let json_data = serde_json::to_string(&entries).unwrap();

        let options = ImportOptions {
            format: ImportFormat::Json,
            ..Default::default()
        };
        let importer = DataImporter::new(options);

        let mut reader = Cursor::new(json_data.as_bytes());
        let result = importer.import_json(&mut reader, &[create_test_category()], &[]);

        assert!(result.is_ok());
        let (_, imported_entries, import_result) = result.unwrap();
        assert_eq!(imported_entries.len(), 1);
        assert_eq!(import_result.imported_entries, 1);
    }

    #[test]
    fn test_parse_datetime() {
        let options = ImportOptions::default();
        let importer = DataImporter::new(options);

        assert!(importer.parse_datetime("2023-01-01 12:00:00").is_ok());
        assert!(importer.parse_datetime("2023-01-01 12:00").is_ok());
        assert!(importer.parse_datetime("2023-01-01").is_ok());
        assert!(importer.parse_datetime("invalid").is_err());
    }

    #[test]
    fn test_validate_category() {
        let options = ImportOptions::default();
        let importer = DataImporter::new(options);

        let valid_category = create_test_category();
        assert!(importer.validate_category(&valid_category).is_ok());

        let mut invalid_category = create_test_category();
        invalid_category.name = "".to_string();
        assert!(importer.validate_category(&invalid_category).is_err());

        invalid_category.name = "工作".to_string();
        invalid_category.color =
            crate::core::category::CategoryColor::Custom("invalid".to_string());
        assert!(importer.validate_category(&invalid_category).is_err());
    }

    #[test]
    fn test_validate_time_entry() {
        let options = ImportOptions::default();
        let importer = DataImporter::new(options);

        let valid_entry = create_test_entry();
        assert!(importer.validate_time_entry(&valid_entry).is_ok());

        let mut invalid_entry = create_test_entry();
        invalid_entry.task_name = "".to_string();
        assert!(importer.validate_time_entry(&invalid_entry).is_err());

        invalid_entry.task_name = "编程".to_string();
        invalid_entry.end_time = Some(invalid_entry.start_time - chrono::Duration::hours(1));
        assert!(importer.validate_time_entry(&invalid_entry).is_err());
    }
}
