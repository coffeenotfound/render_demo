
pub struct ParsedSource {
	pub shader_type: Option<SourceShaderType>,
	
	pub namespace: Option<String>,
	pub glsl_version: Option<String>,
	
	pub import_declarations: Vec<String>,
	
	/// The full usable "stripped" glsl source code without ssl directives
	pub source_tree: Vec<SourceToken>,
	
	pub exported_functions: Vec<ExportedFunction>,
}

pub enum SourceToken {
	TextSource {
		body: String,
	},
	HiddenSource {
		body: String,
	},
}

pub struct ExportedFunction {
	pub signature: String,
}

pub enum SourceShaderType {
	Include,
	Vertex,
	Fragment,
	TessEval,
}
