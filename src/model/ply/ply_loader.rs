use std::io::{Read, Seek, BufReader, BufRead, SeekFrom};
use std::error;
use std::fmt;
use crate::model::ply::{PlyFileHeader, PlyElementDescriptor, standard_formats, PlyPropertyDescriptor, PlyScalar, PlyDatatype};
use std::str::{SplitAsciiWhitespace, FromStr};
use byteorder::{LittleEndian, ByteOrder};
use num::{self, NumCast};
use std::marker::PhantomData;

pub struct PlyMeshLoader<'r, R: Read + Seek> {
	reader: &'r mut R,
//	file_header: Option<PlyFileHeader>,
//	parse_state: Option<FileParseState>,
}

impl<'r, R: Read + Seek> PlyMeshLoader<'r, R> {
	pub fn parse_header(self) -> Result<PlyDataPuller<'r, R>, Box<dyn error::Error>> {
		fn ply_err<T>(message: &'static str) -> Result<T, Box<dyn error::Error>> {
			Err(Box::from(PlyError::new(message)))
		}
		
//		if let None = self.file_header {
			// Make buf reader
			let mut buf_reader = BufReader::new(self.reader);
			
			// Read file header
			let mut lines = (&mut buf_reader).lines();
			
			let mut element_vec: Vec<PlyElementDescriptor> = Vec::new();
			let mut current_element: Option<PlyElementDescriptor> = None;
			
			let mut i = 0;
//			let mut k = 0;
			'header_loop: loop {
				let line = if let Some(l) = lines.next() {
					if let Ok(l) = l {
						l
					}
					else {
						return Err(Box::from(l.unwrap_err()));
					}
				}
				else {
					return ply_err("Header missing required fields or has no 'end_header' line");
				};
				
//				// DEBUG:
//				println!("DEBUG: line: {}", line);
//				if k > 40 {
//					break;
//				}
//				k += 1;
				
				// Ignore comment lines
				if line.starts_with("comment") {
					continue 'header_loop;
				}
				
				// End of header
				if line.as_str().eq("end_header") {
					break 'header_loop;
				}
				
				// Magic number
				if i == 0 {
					if !line.as_str().eq("ply") {
						return ply_err("Header missing ply fingerprint");
					}
					i = 1;
				}
				// Format and version
				else if i == 1 {
					if !line.starts_with("format") {
						return ply_err("Header missing ply format line")
					}
					if !line.as_str().eq("format ascii 1.0") {
						return ply_err("Unknown or invalid ply format (only ascii 1.0 is currently supported)");
					}
					i = 2;
				}
				// Element descriptor
				else if line.starts_with("element") {
					// Put previous descriptor into list if we have one
					if let Some(elem) = current_element.take() {
//						elem.recalc_full_element_size();
						element_vec.push(elem);
					}
					
					// Read element line
					let mut split_line = line.split_ascii_whitespace();
					let _ = split_line.next(); // Skip 'element' token
					
					let elem_name = String::from({
						let a = split_line.next();
						if a.is_none() {
							return ply_err("Invalid element descriptor");
						}
						a.unwrap()
					});
					
					let num_entries = {
						let a = split_line.next();
						if a.is_none() {
							return ply_err("Invalid element descriptor");
						}
						let a = a.unwrap();
						
						let a = a.parse::<u32>();
						if a.is_err() {
							return ply_err("Invalid element descriptor");
						}
						a.unwrap()
					};
					
					// Make new descriptor
					let elem_index = element_vec.len() as u32;
					current_element = Some(PlyElementDescriptor::new(elem_index, elem_name, num_entries));
				}
				// Property descriptor
				else if line.starts_with("property") {
					// Check that we are actually in an element
					if let None = current_element {
						return ply_err("Misplaced property line outside of element descriptor");
					}
					
					// Read element line
					let mut split_line = line.split_ascii_whitespace();
					let _ = split_line.next(); // Skip 'property' token
					
					let prop_type = {
						let a = split_line.next();
						if a.is_none() {
							return ply_err("Invalid property descriptor");
						}
						let a = a.unwrap();
						
						if a.eq("list") {
							let list_index_type = {
								let a = split_line.next();
								if a.is_none() {
									return ply_err("Invalid property descriptor: Cannot read list index type");
								}
								
								match PlyScalar::from_str(a.unwrap()) {
									Some(s) => s,
									None => return ply_err("Invalid property descriptor: Unknown list index type"),
								}
							};
							let list_data_type = {
								let a = split_line.next();
								if a.is_none() {
									return ply_err("Invalid property descriptor: Cannot read list data type");
								}
								
								match PlyScalar::from_str(a.unwrap()) {
									Some(s) => s,
									None => return ply_err("Invalid property descriptor: Unknown list data type"),
								}
							};
							
							PlyDatatype::List {
								index: list_index_type,
								element: list_data_type,
							}
						}
						else {
							match PlyScalar::from_str(a) {
								Some(s) => PlyDatatype::Scalar(s),
								None => return ply_err("Unkown type in property descriptor") 
							}
						}
					};
					
					let prop_name = {
						let a = split_line.next();
						let a = if let Some(a) = a {
							String::from(a)
						}
						else {
							return ply_err("Invalid property descriptor: Invalid name");
						};
						a
					};
					
					// Create property descriptor
					let property_descriptor = PlyPropertyDescriptor {
						name: prop_name,
						datatype: prop_type,
					};
					
					// Add to current element
					current_element.as_mut().unwrap().properties.push(property_descriptor);
				}
			}
			
			// Put last descriptor into list
			if let Some(elem) = current_element.take() {
//				elem.recalc_full_element_size();
				element_vec.push(elem);
			}
			
			// Create file header
			let file_header = PlyFileHeader {
				format: standard_formats::ASCII_10,
				elements: element_vec,
			};
			
			// Get back our file at the proper position
			let real_seek_pos = buf_reader.seek(SeekFrom::Current(0)).map_err(|_| PlyError::new("Failed to seek file pos after header (this is probably a bug)"))?;
			let reader = buf_reader.into_inner();
			reader.seek(SeekFrom::Start(real_seek_pos))?;
			
			// Make puller
			let puller = PlyDataPuller {
				buf_reader: BufReader::new(reader),
				file_header,
				parse_state: None,
				_phantom: PhantomData,
			};
			
			return Ok(puller);
//		}
//		else {
//			return ply_err("Cannot parse header more than once");
//		}
	}
	
	pub fn new(source: &'r mut R) -> PlyMeshLoader<'r, R> {
		PlyMeshLoader {
			reader: source,
		}
	}
}

pub struct PlyDataPuller<'r, R: Read + Seek> {
	buf_reader: BufReader<&'r mut R>,
	file_header: PlyFileHeader,
	parse_state: Option<FileParseState>,
	_phantom: PhantomData<()>
}

impl<'r, R: Read + Seek> PlyDataPuller<'r, R> {
	pub fn next_event<'a>(&'a mut self) -> PullEvent<'a, 'r, R> {
		return if self.parse_state.is_none() {
			if self.file_header.elements.len() <= 0 {
				return PullEvent::End
			}
			
			// Create initial parse state
			self.parse_state = Some(FileParseState {
				current_element_index: 0,
//				entries_left: self.file_header.elements.first().unwrap().num_entries,
			});
			
			let parser = PlyElementParser::new(&mut self.buf_reader, self.file_header.elements.first().unwrap(), self.parse_state.as_mut().unwrap());
			PullEvent::Element(parser)
		}
		else {
			// If we still have elements left update index
			let state = self.parse_state.as_mut().unwrap();
			
			if state.current_element_index < self.file_header.elements.len().saturating_sub(1) as u32 {
				state.current_element_index += 1;
				
				let parser = PlyElementParser::new(&mut self.buf_reader, self.file_header.elements.get(state.current_element_index as usize).unwrap(), self.parse_state.as_mut().unwrap());
				PullEvent::Element(parser)
			}
			else {
				PullEvent::End
			}
		}
	}
	
	pub fn header(&self) -> &PlyFileHeader {
		&self.file_header
	}
}

struct FileParseState {
	current_element_index: u32
}

pub enum PullEvent<'a, 'r: 'a, R: Read + Seek> {
	Element(PlyElementParser<'a, 'r, R>),
	End,
}

impl<'a, 'r: 'a, R: Read + Seek> PullEvent<'a, 'r, R> {
	
}

pub struct PlyElementParser<'a, 'r, R: Read + Seek> {
	buf_reader: &'a mut BufReader<&'r mut R>,
//	parse_state: &'a mut FileParseState,
	element_descriptor: &'a PlyElementDescriptor,
//	full_element_size: u32,
	entries_left: u32,
}

impl<'a, 'r: 'a, R: Read + Seek> PlyElementParser<'a, 'r, R> {
	pub fn read_entry(&mut self, buffer: &mut [u8]) -> Result<(), PlyReadError> {
//		fn ply_err<T>(message: &'static str) -> Result<T, Box<dyn error::Error>> {
//			Err(Box::from(PlyError::new(message)))
//		}
		
		// Return appropriate error if no more lines are left
		if self.entries_left <= 0 {
			return Err(PlyReadError::NoMoreEntries);
		}
		
		// Get initial stream pos so we can rewind later when the given buffer is
		// too small.
		// NOTE: This discards the internal buffer of the buffered reader so this
		// is fcking stupid, but without implementing it myself there is no other way
		let initial_stream_pos = match self.buf_reader.seek(SeekFrom::Current(0)) {
			Ok(pos) => pos,
			Err(err) => return Err(PlyReadError::Other(Box::new(err))),
		};
		
		let mut lines = self.buf_reader.lines();
		
		let mut buffer_pos = 0usize;
		let mut only_measuring_size = false;
		
		// Get line
		let line = lines.next();
		let line = if let Some(l) = line {
			if let Ok(l) = l {
				l
			} else {
				return Err(PlyReadError::Other(Box::new(PlyError::new("Unexpected line"))));
			}
		} else {
//			return ply_err("Unexpectedly no more lines left")
			return Err(PlyReadError::Other(Box::new(PlyError::new("Unexpected line"))));
		};
		
		// Split line at whitespace
		let mut split_line = line.split_ascii_whitespace();
		
		// Read entry line
		for p in &self.element_descriptor.properties {
			fn write_value<T: NumCast>(scalar_type: PlyScalar, value: T, data_size: usize, buffer: &mut [u8], buffer_pos: &mut usize, only_measure: &mut bool) {
				// Buffer is too small, eventually return a TooSmall error but
				// for now only set the flag so we can continue calculating the
				// actually needed buffer size
				let final_pos = *buffer_pos + data_size;
				if buffer.len() < final_pos {
					*only_measure = true;
				}
				
				if *only_measure {
					*buffer_pos += data_size; // Increment anyway so we know what the final needed buffer size is
				}
				else {
					// Get offset buffer slice
					let slice = &mut buffer[*buffer_pos..final_pos];
					
					match scalar_type {
						S::uchar => slice[0] = num::cast::<_, u8>(value).unwrap(),
						S::uint => LittleEndian::write_u32(slice, num::cast::<_, u32>(value).unwrap()),
						S::float => LittleEndian::write_f32(slice, num::cast::<_, f32>(value).unwrap()),
						_ => unimplemented!("DEBUG: Datatype not implemented yet"),
					}
					
					// Increment buffer pos
					*buffer_pos += data_size;
				}
			}
			
			fn process_value<T: Copy + FromStr + NumCast>(scalar_type: PlyScalar, split_line: &mut SplitAsciiWhitespace, buffer: &mut [u8], buffer_pos: &mut usize, only_measure: &mut bool) -> Result<T, PlyReadError> {
				let value_str = if let Some(s) = split_line.next() {
					s
				} else {
					return Err(PlyReadError::Other(Box::new(PlyError::new("Invalid entry line: Missing property value"))));
				};
				
				let val: T = match value_str.parse::<T>() {
					Ok(val) => val,
					Err(_err) => return Err(PlyReadError::Other(Box::new(PlyError::new("Invalid entry line: Failed to parse value")))),
				};
				
				// Write the value into the buffer
				write_value::<T>(scalar_type, val, std::mem::size_of::<T>(), buffer, buffer_pos, only_measure);
				
				Ok(val as T)
			}
			
			fn process_scalar_uncast(scalar_type: PlyScalar, split_line: &mut SplitAsciiWhitespace, buffer: &mut [u8], buffer_pos: &mut usize, only_measure: &mut bool) -> Result<(), PlyReadError> {
				match scalar_type {
					S::uchar => process_value::<u8>(scalar_type, split_line, buffer, buffer_pos, only_measure).map(|_| ()),
					S::uint => process_value::<u32>(scalar_type, split_line, buffer, buffer_pos, only_measure).map(|_| ()),
					S::float => process_value::<f32>(scalar_type, split_line, buffer, buffer_pos, only_measure).map(|_| ()),
					_ => unimplemented!("DEBUG: Datatype not implemented yet"),
				}
			}
			
			use PlyScalar as S;
			match p.datatype {
				PlyDatatype::Scalar(scalar) => {
					process_scalar_uncast(scalar, &mut split_line, buffer, &mut buffer_pos, &mut only_measuring_size)?;
				}
				PlyDatatype::List {index, element} => {
					let num_elements = match index {
						S::uchar => process_value::<u8>(index, &mut split_line, buffer, &mut buffer_pos, &mut only_measuring_size)? as u64,
						S::ushort => process_value::<u16>(index, &mut split_line, buffer, &mut buffer_pos, &mut only_measuring_size)? as u64,
						S::uint => process_value::<u32>(index, &mut split_line, buffer, &mut buffer_pos, &mut only_measuring_size)? as u64,
						_ => return Err(PlyReadError::Other(Box::new(PlyError::new("Invalid list index datatype: Only uchar, ushort and uint are valid")))),
					};
					
					for _ in 0..num_elements {
						process_scalar_uncast(element, &mut split_line, buffer, &mut buffer_pos, &mut only_measuring_size)?;
					}
				}
			}
		}
		
		if only_measuring_size {
			// Rewind reader
			if let Err(e) = self.buf_reader.seek(SeekFrom::Start(initial_stream_pos)) {
				return Err(PlyReadError::Other(Box::new(e)));
			}
			
			// Return the min buffer size based on the final offset (since we still go over all elements even if the buffer is too small)
			Err(PlyReadError::BufferTooSmall {min_buffer_size: buffer_pos})
		}
		else {
			self.entries_left -= 1;
			Ok(())
		}
	}
	
	pub fn element_descriptor(&self) -> &'a PlyElementDescriptor {
		self.element_descriptor
	}
	
	fn new(reader: &'a mut BufReader<&'r mut R>, element_descriptor: &'a PlyElementDescriptor, _parse_state: &'a mut FileParseState) -> PlyElementParser<'a, 'r, R> {
//		// Calc full element size
//		let mut full_element_size = 0u32;
//		for p in &element_descriptor.properties {
//			full_element_size += p.datatype.byte_size();
//		}
		
		let entries_left = element_descriptor.num_entries;
		
		PlyElementParser {
			buf_reader: reader,
			element_descriptor,
//			full_element_size,
//			parse_state,
			entries_left,
		}
	}
}

//mod generic_byteorder {
//	use byteorder::{WriteBytesExt, LittleEndian, ByteOrder};
//	
//	pub trait GenericByteOrder<E: ByteOrder> {
//		fn write_into_slice(self, buffer: &mut [u8]);
//	}
//	
//	impl<E: ByteOrder> GenericByteOrder<E> for f32 {
//		fn write_into_slice(self, buffer: &mut [u8]) {
//			E::write_f32(buffer, self)
//		}
//	}
//	
//	impl<E: ByteOrder> GenericByteOrder<E> for u8 {
//		fn write_into_slice(self, buffer: &mut [u8]) {
//			buffer[0] = self
//		}
//	}
//	
//	impl<E: ByteOrder> GenericByteOrder<E> for u32 {
//		fn write_into_slice(self, buffer: &mut [u8]) {
//			E::write_u32(buffer, self)
//		}
//	}
//}

pub enum PlyReadError {
	NoMoreEntries,
	BufferTooSmall {
		min_buffer_size: usize,
	},
	Other(Box<dyn error::Error>),
}

impl error::Error for PlyReadError {}

impl fmt::Display for PlyReadError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use PlyReadError as E;
		match self {
			E::NoMoreEntries => write!(f, "PlyReadError: No more entries"),
			E::BufferTooSmall {min_buffer_size} => write!(f, "PlyReadError: Buffer too small: min size = {}", min_buffer_size),
			E::Other(error) => <Box<dyn error::Error> as fmt::Display>::fmt(error, f)
		}
		
	}
}

impl fmt::Debug for PlyReadError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		<Self as fmt::Display>::fmt(self, f)
	}
}

pub struct PlyError {
	message: &'static str,
}


impl PlyError {
	pub fn new(message: &'static str) -> PlyError {
		PlyError {
			message
		}
	}
}

impl error::Error for PlyError {}

impl fmt::Display for PlyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(f, "PlyError: {}", self.message)
	}
}

impl fmt::Debug for PlyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		<Self as fmt::Display>::fmt(self, f)
	}
}

pub fn dump_ply_header(header: &PlyFileHeader) {
	for element in &header.elements {
		println!("element '{}' {}", element.name, element.num_entries);
		
		for property in &element.properties {
			println!("  property '{}' {:?}", property.name, property.datatype)
		}
	}
}

/*
pub fn test() -> Result<(), Box<dyn error::Error>> {
	let mut file = OpenOptions::new().read(true).open(r"C:\Users\Jan\Desktop\Lee Head\Lee Head.ply")?;
	let loader = PlyMeshLoader::new(&mut file);
	
	let mut puller = loader.parse_header()?;
	
	dump_ply_header(&puller.file_header);
	
//	let mut puller = RefCell::new(puller);
	loop {
//		let mut borrowed_puller = puller.borrow_mut();
		match puller.next_event() {
			PullEvent::Element(mut parser) => {
				let mut buffer = [0u8; 32];
				
				let res = parser.read_entry(&mut buffer);
				if let Err(PlyReadError::BufferTooSmall {min_buffer_size}) = res {
					println!("Buffer too small! (min {})", min_buffer_size);
					return Ok(());
				}
				else if let Ok(_) = res {
					let mut pos = 0;
					for p in parser.element_descriptor.properties() {
						match p.datatype {
							PlyDatatype::Scalar(scalar) => {
								let final_pos = pos + scalar.byte_size(); 
								
								match scalar {
									PlyScalar::float => {
										let val = LittleEndian::read_f32(&buffer[(pos as usize)..(final_pos as usize)]);
										println!("f32({})", val);
									},
									_ => unimplemented!()
								}
								
								pos = final_pos;
							},
							PlyDatatype::List {index, element} => {
								
							}
						}
					}
				}
			}
			PullEvent::End => break,
		}
		break;
	}
	
	Ok(())
}
*/
