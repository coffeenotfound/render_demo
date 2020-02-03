use crate::structured_shader_language::{ParsedSource, SourceToken, ExportedFunction};
use std::ops::Index;

pub struct SSLSourceParser {}

impl SSLSourceParser {
	pub fn parse_source(&mut self, source_code: String) -> ParsedSource {
		// Allocate full body string
		//let mut full_body_buffer = String::with_capacity(source_code.len());
		let mut token_tree = Vec::<SourceToken>::new();
		let mut current_body_buffer = String::new();
		
		let mut shader_namespace = Option::<String>::None;
		let mut glsl_version = Option::<String>::None;
		
		let mut import_declarations = Vec::<String>::new();
		
		let mut export_func_list = Vec::<ExportedFunction>::new();
		
		let mut inside_export_func = false;
		let mut expect_export_func_signature = true;
		let mut export_func_signature = Option::<String>::None;
		
		let mut inside_hide_block = false;
		
		// Split lines
		let mut lines = source_code.lines();
		
		// Iterate over source lines
		for line in lines {
			// Check for ssl directive
			if line.trim_start().starts_with("@") {
				// Process ssl directive
				let mut split_line = line.split_ascii_whitespace();
				
				let directive = split_line.next();
				
				let valid_directive: bool = if let Some(directive) = directive {
					if directive.eq("@shadertype") {
						// TODO: Implement
						true
					}
					else if directive.eq("@glslversion") {
						let version_string = format!("{} {}", split_line.next().unwrap(), split_line.next().unwrap_or(""));
						glsl_version = Some(version_string);
						true
					} 
					else if directive.eq("@namespace") {
						shader_namespace = Some(String::from(split_line.next().unwrap()));
						true
					}
					else if directive.eq("@import") {
						if let Some(import) = split_line.next() {
							import_declarations.push(String::from(import));
						}
						true
					}
					else if directive.eq("@exportfunc") {
						// End current normal text block
						token_tree.push(SourceToken::TextSource {body: current_body_buffer});
						current_body_buffer = String::new();
						
						inside_export_func = true;
						expect_export_func_signature = true;
						true
					}
					else if directive.eq("@hide") {
						// End current normal text block
						token_tree.push(SourceToken::TextSource {body: current_body_buffer});
						current_body_buffer = String::new();
						
						inside_hide_block = true;
						true
					}
					else if directive.eq("@end") {
						if inside_export_func {
							inside_export_func = false;
							
							// Push onto tree
							token_tree.push(SourceToken::TextSource {body: current_body_buffer});
							current_body_buffer = String::new();
						}
						else if inside_hide_block {
							inside_hide_block = false;
							
							// Push onto tree
							token_tree.push(SourceToken::HiddenSource {body: current_body_buffer});
							current_body_buffer = String::new();
						}
						true
					}
					else {
						false
					}
				} else {
					false
				};
			}
			else {
				// Get export func signature
				if inside_export_func && expect_export_func_signature {
					expect_export_func_signature = false;
					
					// Get signature (TODO: This is super bad, but works for now)
					let signature = String::from(line.index(..line.find("{").unwrap()).trim());
					export_func_list.push(ExportedFunction {signature})
				}
				
				// Simply append line to buffer
				current_body_buffer.push_str(line);
				current_body_buffer.push('\n');
			}
		}
		
		// End last block
		if !current_body_buffer.is_empty() {
			if inside_hide_block {
				// Push onto tree
				token_tree.push(SourceToken::HiddenSource {body: current_body_buffer});
			}
			else {
				// Push onto tree
				token_tree.push(SourceToken::TextSource {body: current_body_buffer});
			}
		}
		
		// Make parsed source
		let parsed_source = ParsedSource {
			shader_type: None,
			namespace: shader_namespace,
			glsl_version,
			import_declarations,
			exported_functions: export_func_list,
			source_tree: token_tree,
		};
		parsed_source
	}
	
	pub fn new() -> Self {
		Self {}
	}
}
