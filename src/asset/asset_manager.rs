use std::path::{PathBuf, Path};
use crate::asset::AssetPath;

pub static mut ASSET_MANAGER_INSTANCE: AssetManager = AssetManager::new();

pub struct AssetManager {
	asset_root: Option<PathBuf>,
}

impl AssetManager {
	pub fn init(&mut self, asset_root: PathBuf) {
		self.asset_root = Some(asset_root);
	}
	
	pub fn resolve_asset_fs_path(&self, asset_path: &AssetPath) -> PathBuf {
		let mut relative_asset_path = Path::new(asset_path.inner_path_slice);
		
		if relative_asset_path.has_root() {
			// Ensure the asset path is not absolute (starts with a seperator) because then
			// joining it to the asset root would return the (in our view) relative asset path as an absolute path
			relative_asset_path = relative_asset_path.strip_prefix(Path::new("/")).expect("Failed to make asset path relative");
		}
		
		self.asset_root.as_ref().unwrap().as_path().join(relative_asset_path)
	}
	
	pub const fn new() -> Self {
		Self {
			asset_root: None,
		}
	}
}
