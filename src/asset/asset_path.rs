use std::fmt::{self, Debug};

pub struct AssetPathBuf {
	inner_path: String,
}

impl AssetPathBuf {
	pub fn as_path<'a>(&'a self) -> AssetPath<'a> {
		AssetPath::from_str(self.inner_path.as_str())
	}
	
	pub fn from_owned(owned_path: String) -> Self {
		Self {
			inner_path: owned_path,
		}
	}
	
	pub fn from(path: &str) -> Self {
		Self::from_owned(String::from(path))
	}
}

impl Debug for AssetPathBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "AssetPathBuf(\"{}\")", &self.inner_path)
	}
}

/*
impl<'a> Deref for &'a AssetPathBuf {
	type Target = AssetPath<'a>;
	
	fn deref(&self) -> &Self::Target {
		&AssetPath::from_buf(self.inner_path.as_str())
	}
}
*/

/*
impl<'a> Deref for AssetPathBuf where Self: 'a {
	type Target = AssetPath<'a>;
	
	fn deref(&'a self) -> &Self::Target {
		&self.as_path()
	}
}
*/

pub struct AssetPath<'a> {
	pub inner_path_slice: &'a str,
}

impl<'a> AssetPath<'a> {
	pub fn is_absolute(&self) -> bool {
		self.inner_path_slice.starts_with("/")
	}
	
	pub fn join(&self, other_relative: &AssetPath) -> Option<AssetPathBuf> {
		// If `other` is absolute joining doesn't make sense
		if other_relative.is_absolute() {
			None
		}
		else {
			// Join the paths
			let lhs = self.inner_path_slice.strip_suffix("/").unwrap_or(self.inner_path_slice);
			let rhs = other_relative.inner_path_slice.strip_prefix("/").unwrap_or(other_relative.inner_path_slice);
			let path_buf = format!("{}/{}", lhs, rhs);
			
			// Make a new path buf and return
			Some(AssetPathBuf::from_owned(path_buf))
		}
	}
	
	pub fn parent<'b>(&'b self) -> Option<AssetPath<'b>> {
		if self.is_absolute() {
			// Already at root, cannot go further
			if self.inner_path_slice.eq("/") {
				None
			}
			else {
				// Remove the trailing slash if any
				let path = self.inner_path_slice.strip_prefix("/").unwrap_or(self.inner_path_slice);
				
				// Strip to the separator
				let parent_path = &path[0..path.rfind("/").unwrap()];
				
				// Return new path slice
				Some(AssetPath::from_str(parent_path))
			}
		}
		else {
			if self.inner_path_slice.eq("") {
				None
			}
			else {
				if let Some(last_separator) = self.inner_path_slice.rfind("/") {
					Some(AssetPath::from_str(&self.inner_path_slice[0..last_separator]))
				}
				else {
					Some(AssetPath::from_str(""))
				}
			}
		}
	}
	
	pub fn from_str(path_slice: &'a str) -> Self {
		Self {
			inner_path_slice: path_slice.trim(),
		}
	}
}

impl<'a> Debug for AssetPath<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "AssetPath(\"{}\")", self.inner_path_slice)
	}
}

/*
impl AsRef<Path> for AssetPath {
	fn as_ref(&self) -> &Path {
		self.inner_path.as_path()
	}
}
*/
