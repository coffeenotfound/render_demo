use std::fmt;

/// http://paulbourke.net/dataformats/ply/

pub mod standard_formats {
	use super::{PlyFormat, PlyFormatVersion};
	
	pub const ASCII_10: PlyFormat = PlyFormat::Ascii(PlyFormatVersion::get(1, 0));
}

pub struct PlyFileHeader {
	pub format: PlyFormat,
	pub elements: Vec<PlyElementDescriptor>,
}

#[derive(Copy, Clone)]
pub enum PlyFormat {
	Ascii(PlyFormatVersion),
	BinaryLE(PlyFormatVersion),
	BinaryBE(PlyFormatVersion),
}

#[derive(Copy, Clone)]
pub struct PlyFormatVersion {
	pub major: u16,
	pub minor: u16
}

impl PlyFormatVersion {
	pub const fn get(major: u16, minor: u16) -> PlyFormatVersion {
		PlyFormatVersion {
			major,
			minor,
		}
	}
}

pub struct PlyElementDescriptor {
	pub element_index: u32,
	pub name: String,
	pub num_entries: u32,
	pub properties: Vec<PlyPropertyDescriptor>
}

impl PlyElementDescriptor {
//	pub fn recalc_full_element_size(&mut self) {
//		let mut full_size = 0u32;
//		for p in &self.properties {
//			full_size += p.datatype.byte_size();
//		}
//		self.full_element_size = full_size;
//	}
	
	pub fn properties(&self) -> &[PlyPropertyDescriptor] {
		self.properties.as_slice()
	}
	
	pub fn new(element_index: u32, name: String, num_entries: u32) -> PlyElementDescriptor {
		PlyElementDescriptor {
			element_index,
			name,
			num_entries,
			properties: Vec::new(),
//			full_element_size: 0,
		}
	}
}

pub struct PlyPropertyDescriptor {
	pub name: String,
	pub datatype: PlyDatatype,
}

#[derive(Copy, Clone)]
pub enum PlyDatatype {
	Scalar(PlyScalar),
	List {
		index: PlyScalar,
		element: PlyScalar,
	}
}

//impl PlyDatatype {
//	pub fn scalar_byte_size(&self) -> Option<u32> {
//		if let PlyDatatype::Scalar(s) = self {
//			Some(s.byte_size())
//		} else {
//			None
//		}
//	}
//}

impl fmt::Debug for PlyDatatype {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use PlyDatatype as D;
		match self {
			D::Scalar(s) => <PlyScalar as fmt::Debug>::fmt(s, f),
			D::List {index, element} => {
				write!(f, "list ")?;
				<PlyScalar as fmt::Debug>::fmt(index, f)?;
				write!(f, " ")?;
				<PlyScalar as fmt::Debug>::fmt(element, f)
			}
		}
	}
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum PlyScalar {
	char,
	uchar,
	short,
	ushort,
	int,
	float,
	uint,
	double,
}

impl PlyScalar {
	pub fn byte_size(&self) -> u32 {
		use PlyScalar as S;
		match self {
			S::char => 1,
			S::uchar => 1,
			S::short => 2,
			S::ushort => 2,
			S::int => 4,
			S::uint => 4,
			S::float => 4,
			S::double => 8,
		}
	}
	
	pub fn from_str(string: &str) -> Option<PlyScalar> {
		use super::PlyScalar as S;
		match string {
			"char" => Some(S::char),
			"uchar" => Some(S::uchar),
			"short" => Some(S::short),
			"ushort" => Some(S::ushort),
			"int" => Some(S::int),
			"uint" => Some(S::uint),
			"float" => Some(S::float),
			"double" => Some(S::double),
			_ => None,
		}
	}
}

impl fmt::Debug for PlyScalar {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use PlyScalar as S;
		let name = match self {
			S::char => "char",
			S::uchar => "uchar",
			S::short => "short",
			S::ushort => "ushort",
			S::int => "int",
			S::uint => "uint",
			S::float => "float",
			S::double => "double",
		};
		write!(f, "{}", name)
	}
}
