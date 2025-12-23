use crate::automation::{AutomationError, AutomationResult};
use calamine::{open_workbook, Data, Reader, Xlsx};
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelCell {
    pub row: u32,
    pub col: u32,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelSheet {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelData {
    pub sheets: Vec<ExcelSheet>,
}

pub struct FileAutomation;

impl FileAutomation {
    pub fn new() -> Self {
        Self
    }

    pub async fn read_file(&self, path: &str) -> AutomationResult<String> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read file: {}", e)))
    }

    pub async fn write_file(&self, path: &str, content: &str) -> AutomationResult<()> {
        tokio::fs::write(path, content)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to write file: {}", e)))
    }

    pub async fn copy_file(&self, src: &str, dest: &str) -> AutomationResult<()> {
        tokio::fs::copy(src, dest)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to copy file: {}", e)))?;
        Ok(())
    }

    pub async fn move_file(&self, src: &str, dest: &str) -> AutomationResult<()> {
        tokio::fs::rename(src, dest)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to move file: {}", e)))
    }

    pub async fn delete_file(&self, path: &str) -> AutomationResult<()> {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to delete file: {}", e)))
    }

    pub async fn create_directory(&self, path: &str) -> AutomationResult<()> {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to create directory: {}", e)))
    }

    pub async fn list_files(&self, path: &str) -> AutomationResult<Vec<String>> {
        let mut entries = tokio::fs::read_dir(path)
            .await
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to list files: {}", e)))?;

        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            AutomationError::ExecutionFailed(format!("Failed to read entry: {}", e))
        })? {
            if let Some(name) = entry.file_name().to_str() {
                files.push(name.to_string());
            }
        }
        Ok(files)
    }

    pub async fn file_exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    /// Read Excel file and return all sheets with their data
    pub fn read_excel(&self, path: &str) -> AutomationResult<ExcelData> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to open Excel file: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let mut sheets = Vec::new();

        for sheet_name in sheet_names {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                let mut rows = Vec::new();
                for row in range.rows() {
                    let row_data: Vec<String> = row
                        .iter()
                        .map(|cell| match cell {
                            Data::Int(i) => i.to_string(),
                            Data::Float(f) => f.to_string(),
                            Data::String(s) => s.clone(),
                            Data::Bool(b) => b.to_string(),
                            Data::DateTime(dt) => dt.to_string(),
                            Data::DateTimeIso(s) => s.clone(),
                            Data::DurationIso(s) => s.clone(),
                            Data::Error(e) => format!("Error: {:?}", e),
                            Data::Empty => String::new(),
                        })
                        .collect();
                    rows.push(row_data);
                }
                sheets.push(ExcelSheet {
                    name: sheet_name,
                    rows,
                });
            }
        }

        Ok(ExcelData { sheets })
    }

    /// Read a specific sheet from Excel file
    pub fn read_excel_sheet(&self, path: &str, sheet_name: &str) -> AutomationResult<ExcelSheet> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to open Excel file: {}", e)))?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read sheet '{}': {}", sheet_name, e)))?;

        let mut rows = Vec::new();
        for row in range.rows() {
            let row_data: Vec<String> = row
                .iter()
                .map(|cell| match cell {
                    Data::Int(i) => i.to_string(),
                    Data::Float(f) => f.to_string(),
                    Data::String(s) => s.clone(),
                    Data::Bool(b) => b.to_string(),
                    Data::DateTime(dt) => dt.to_string(),
                    Data::DateTimeIso(s) => s.clone(),
                    Data::DurationIso(s) => s.clone(),
                    Data::Error(e) => format!("Error: {:?}", e),
                    Data::Empty => String::new(),
                })
                .collect();
            rows.push(row_data);
        }

        Ok(ExcelSheet {
            name: sheet_name.to_string(),
            rows,
        })
    }

    /// Read a specific cell from Excel file
    pub fn read_excel_cell(&self, path: &str, sheet_name: &str, row: u32, col: u32) -> AutomationResult<String> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to open Excel file: {}", e)))?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to read sheet '{}': {}", sheet_name, e)))?;

        let cell = range.get((row as usize, col as usize));
        let value = match cell {
            Some(Data::Int(i)) => i.to_string(),
            Some(Data::Float(f)) => f.to_string(),
            Some(Data::String(s)) => s.clone(),
            Some(Data::Bool(b)) => b.to_string(),
            Some(Data::DateTime(dt)) => dt.to_string(),
            Some(Data::DateTimeIso(s)) => s.clone(),
            Some(Data::DurationIso(s)) => s.clone(),
            Some(Data::Error(e)) => format!("Error: {:?}", e),
            Some(Data::Empty) | None => String::new(),
        };

        Ok(value)
    }

    /// Write data to a new Excel file
    pub fn write_excel(&self, path: &str, data: &ExcelData) -> AutomationResult<()> {
        let mut workbook = Workbook::new();

        for sheet in &data.sheets {
            let worksheet = workbook.add_worksheet();
            worksheet
                .set_name(&sheet.name)
                .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to set sheet name: {}", e)))?;

            for (row_idx, row) in sheet.rows.iter().enumerate() {
                for (col_idx, cell_value) in row.iter().enumerate() {
                    worksheet
                        .write_string(row_idx as u32, col_idx as u16, cell_value)
                        .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to write cell: {}", e)))?;
                }
            }
        }

        workbook
            .save(path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to save Excel file: {}", e)))?;

        Ok(())
    }

    /// Write data to a single sheet in a new Excel file
    pub fn write_excel_sheet(&self, path: &str, sheet_name: &str, rows: &[Vec<String>]) -> AutomationResult<()> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        worksheet
            .set_name(sheet_name)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to set sheet name: {}", e)))?;

        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, cell_value) in row.iter().enumerate() {
                // Try to parse as number first
                if let Ok(num) = cell_value.parse::<f64>() {
                    worksheet
                        .write_number(row_idx as u32, col_idx as u16, num)
                        .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to write cell: {}", e)))?;
                } else {
                    worksheet
                        .write_string(row_idx as u32, col_idx as u16, cell_value)
                        .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to write cell: {}", e)))?;
                }
            }
        }

        workbook
            .save(path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to save Excel file: {}", e)))?;

        Ok(())
    }

    /// Get list of sheet names from Excel file
    pub fn get_excel_sheet_names(&self, path: &str) -> AutomationResult<Vec<String>> {
        let workbook: Xlsx<_> = open_workbook(path)
            .map_err(|e| AutomationError::ExecutionFailed(format!("Failed to open Excel file: {}", e)))?;

        Ok(workbook.sheet_names().to_vec())
    }
}

impl Default for FileAutomation {
    fn default() -> Self {
        Self::new()
    }
}
