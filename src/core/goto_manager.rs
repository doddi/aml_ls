use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tower_lsp::lsp_types::Url;

#[derive(Debug, Default)]
pub struct GotoManager {
    aml_files: Arc<RwLock<HashMap<String, Url>>>,
}

impl GotoManager {
    pub fn gather_aml_files_for_project(&self, root: &Path) -> std::io::Result<()> {
        if root.is_dir() {
            for entry in std::fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    if !path.ends_with(".git") && !path.ends_with(".idea") {
                        self.gather_aml_files_for_project(&path)?;
                    }
                }
                else if let Some(extension) = path.extension()  {
                    if extension == "aml" {
                        let key = path.file_stem().unwrap().to_str().unwrap().to_ascii_lowercase();
                        let value = Url::from_file_path(path).unwrap();
                        self.aml_files.write().unwrap().insert(key, value);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn debug_all_aml_files(&self) {
        tracing::info!("all aml files found: {:?}", self.aml_files.read().unwrap());
    }

    pub fn get_url_for_token(&self, token: &str) -> Option<Url> {
        if let Some(value) = self.aml_files.read().unwrap().get(token) {
            return Some(value.clone());
        }
        None
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

   #[test]
   pub fn test_gather_aml_files_for_project() {
       let goto_manager = GotoManager::default();

       goto_manager.gather_aml_files_for_project(Path::new("./templates")).unwrap();
       assert_ne!(goto_manager.aml_files.read().unwrap().len(), 0);
       assert_eq!(goto_manager.aml_files.read().unwrap().get("sample").unwrap().to_string(),
                  "./templates/sample.aml".to_string());
   }
}