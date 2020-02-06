use std::collections::HashMap;
use crate::btex::BTexImageFormat;

pub trait BTexFormatRegistry<'a> {
	fn lookup_format(&self, name: &str) -> Option<&BTexImageFormat<'a>>;
}

pub struct HashMapFormatRegistry<'a> {
	format_map: HashMap<&'a str, BTexImageFormat<'a>>,
}

impl<'a> BTexFormatRegistry<'a> for HashMapFormatRegistry<'a> {
	fn lookup_format(&self, name: &str) -> Option<&BTexImageFormat<'a>> {
		self.format_map.get(name)
	}
}
