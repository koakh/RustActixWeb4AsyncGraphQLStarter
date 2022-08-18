use std::path::Path;

/// check if file exists
pub fn file_exists(file_path: &str) -> bool {
  let package_exist = Path::new(&file_path).exists();
  package_exist
}
