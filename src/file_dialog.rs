use std::fs;
use std::path::Path;

pub struct FileDialog {
    current_dir: String,
    entries: Vec<String>,
    selected_index: usize,
    visible: bool,
}

impl FileDialog {
    pub fn new(default_dir: &str) -> Self {
        let mut dialog = Self {
            current_dir: default_dir.to_string(),
            entries: Vec::new(),
            selected_index: 0,
            visible: false,
        };
        dialog.refresh_entries();
        dialog
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.refresh_entries();
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    pub fn select_current(&mut self) -> Option<String> {
        if let Some(entry) = self.entries.get(self.selected_index) {
            let path = Path::new(&self.current_dir).join(entry);
            if path.is_dir() {
                self.current_dir = path.to_string_lossy().to_string();
                self.selected_index = 0;
                self.refresh_entries();
                None
            } else if path.is_file() && entry.ends_with(".prg") {
                self.hide();
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn go_up(&mut self) {
        let path = Path::new(&self.current_dir);
        if let Some(parent) = path.parent() {
            self.current_dir = parent.to_string_lossy().to_string();
            self.selected_index = 0;
            self.refresh_entries();
        }
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    let name = e.file_name().to_string_lossy().to_string();
                    path.is_dir() || name.ends_with(".prg")
                })
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect();
            files.sort();
            self.entries = files;
        }
        if self.selected_index >= self.entries.len() {
            self.selected_index = self.entries.len().saturating_sub(1);
        }
    }

    pub fn current_dir(&self) -> &str {
        &self.current_dir
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
}

pub fn load_prg_file(path: &str) -> Result<(u16, Vec<u8>), Box<dyn std::error::Error>> {
    let data = fs::read(path)?;
    if data.len() < 2 {
        return Err("PRG file too small".into());
    }
    let load_addr = u16::from_le_bytes([data[0], data[1]]);
    Ok((load_addr, data[2..].to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_dialog_new() {
        let dialog = FileDialog::new("./software");
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_file_dialog_visibility() {
        let mut dialog = FileDialog::new("./software");
        dialog.show();
        assert!(dialog.is_visible());
        dialog.hide();
        assert!(!dialog.is_visible());
    }
}
