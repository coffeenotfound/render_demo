use crate::structured_shader_language::{ParsedSource, SourceToken};
use std::fmt::{self, Write};

pub struct SSLTranspiler<'a> {
	import_scope: Vec<&'a ParsedSource>,
}

impl<'a> SSLTranspiler<'a> {
	/// Adds the given include into the import scope.
	pub fn add_include(&mut self, include: &'a ParsedSource) {
		self.import_scope.push(include);
	}
	
	#[allow(unused_must_use)] // DEBUG: Allow unused fmt results for now
	pub fn transpile(&mut self, source: &ParsedSource) -> TranspiledShaderSource {
		let mut buffer = String::new();
		
		// Write glsl version directive
		if let Some(version) = &source.glsl_version {
			write!(buffer, "#version {}\n", version);
		}
		
		// Write divider
		buffer.push_str("\n// [[ import forward declarations ]] //\n\n");
		
		// Write import functions' forward declarations
		for import_decl in &source.import_declarations {
			// Find include
			for include in &self.import_scope {
				if let Some(include_namespace) = &include.namespace {
					if include_namespace.eq(import_decl) {
						// Emit forward declartions for all exported functions
						for exported_func in &include.exported_functions {
							buffer.push_str(&exported_func.signature);
							buffer.push_str(";\n");
						}
					}
				}
			}
		}
		
		fn emit_source_code(buffer: &mut String, source: &ParsedSource, emit_hidden: bool) -> Result<(), fmt::Error> {
			for token in &source.source_tree {
				match token {
					SourceToken::TextSource {body} => {
						buffer.push_str(&body);
					}
					SourceToken::HiddenSource {body} => {
						// Only emit hidden block if the flag is given
						if emit_hidden {
							buffer.push_str(&body);
						}
					}
				}
			}
			Ok(())
		}
		
		// Write divider
		write!(buffer, "\n// [[ own source ]] //\n\n");
		
		// Emit actual source code
		emit_source_code(&mut buffer, source, true);
		
		// Emit import source code
		// Write import functions' forward declarations
		for import_decl in &source.import_declarations {
			// Find include
			for include in &self.import_scope {
				if let Some(include_namespace) = &include.namespace {
					if include_namespace.eq(import_decl) {
						// Write divider
						write!(buffer, "\n// [[ import source for \"{}\" ]] //\n\n", include_namespace);
						
						// Emit the source code (without the hidden blocks)
						emit_source_code(&mut buffer, include, false);
					}
				}
			}
		}
		
		// Writer trailer
		buffer.push_str("\n// [[ end of transpiled source ]] //\n");
		
//		// DEBUG: Print transpiled source to console
//		println!("--------{}\n{}\n--------", source.namespace.as_ref().map_or("<unknown>", |s| s.as_str()), buffer);
		
		// Make TranspiledSource and return
		TranspiledShaderSource::new(buffer)
	}
	
	pub fn new() -> Self {
		Self {
			import_scope: Vec::new(),
		}
	}
}

pub struct TranspiledShaderSource {
	pub source_code: String,
}

impl TranspiledShaderSource {
	pub fn new(source_code: String) -> Self {
		Self {
			source_code
		}
	}
}
