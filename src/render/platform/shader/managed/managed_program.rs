use std::error;
use std::fs::{OpenOptions};
use std::io::{self, Read};
use crate::asset::{AssetPathBuf, AssetPath, ASSET_MANAGER_INSTANCE};
use crate::structured_shader_language::{SSLSourceParser, ParsedSource, SSLTranspiler};
use crate::render::platform::shader::{ShaderProgram, ShaderStage, ShaderCompileStatus, ProgramLinkStatus, ProgramLinkOptions, ShaderCompileOptions, Shader, ShaderCode};
use crate::render::platform::shader::managed::ProgramAssetSchema;

pub struct ManagedProgram {
	program_asset_path: Option<AssetPathBuf>,
	
	program_object: Option<ShaderProgram>,
	needs_recompile: bool,
}

impl ManagedProgram {
	pub fn program(&self) -> Option<&ShaderProgram> {
		self.program_object.as_ref()
	}
	
	pub fn program_mut(&mut self) -> Option<&mut ShaderProgram> {
		self.program_object.as_mut()
	}
	
	pub fn program_asset_path<'a>(&'a self) -> Option<AssetPath<'a>> {
		if let Some(path) = &self.program_asset_path {
			Some(path.as_path())
		}
		else {
			None
		}
	}
	
	pub fn mark_recompile_needed(&mut self) {
		self.needs_recompile = true;
	}
	
	pub fn needs_recompile(&self) -> bool {
		self.needs_recompile
	}
	
//	pub fn reload(&mut self) {
//		// Create program
//		if let None = self.program_object {
//			self.program_object = Some(ShaderProgram::new());
//		}
//		
//		// Load program asset
//		if let Some(program_asset_path) = &self.program_asset_path {
//			
//		}
//	}
	
	pub fn do_recompile(&mut self) {
		let program = self.program_object.get_or_insert_with(|| ShaderProgram::new());
		
		// (Re-)compile all shaders
		for stage in ShaderStage::stages() {
			if let Some(shader) = program.attached_shader_mut(stage) {
				let mut compile_options = ShaderCompileOptions::default();
				
				let compile_result = shader.compile(compile_options.with_info_log(false, true));
				
				match compile_result.status {
					ShaderCompileStatus::Success => {},
					_ => {
						println!("Shader compile failed (status {:?}):", compile_result.status);
						println!("{}", compile_result.info_log.as_deref().unwrap_or("<< no infolog >>"));
					}
				}
			}
		}
		
		// Link the program
		let link_result = program.link(ProgramLinkOptions::default().with_info_log(false, true));
		match link_result.status {
			ProgramLinkStatus::Success => {},
			_ => {
				println!("Program link failed (status {:?}):", link_result.status);
				println!("{}", link_result.info_log.as_deref().unwrap_or("<< no infolog >>"));
			}
		}
	
		// Reset flag
		self.needs_recompile = false;
	}
	
	pub fn reload_from_asset(&mut self) -> Result<(), Box<dyn error::Error>> {
		// Open the program asset
		let (program_asset_contents, program_asset_path) = {
			let asset_path = self.program_asset_path.as_ref().unwrap().as_path();
			
			// Resolve the actual file path
			let real_path = unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(&asset_path);
			
			// Read the program asset to string
			let mut file = OpenOptions::new().read(true).open(real_path)?;
			let mut buffer = String::new();
			file.read_to_string(&mut buffer)?;
			
			(buffer, asset_path)
		};
		
		// Deserialize the program def
		let program_def = ron::de::from_str::<ProgramAssetSchema::ProgramDef>(&program_asset_contents)?;
		
		// Setup the ssl source parser
		let mut source_parser = SSLSourceParser::new();
		
		fn load_asset_as_str(asset_path: &AssetPath, base_program_path: &AssetPath) -> Result<String, io::Error> {
			// Relativize the asset path (if not absolute)
			let joined_path_buf: AssetPathBuf;
			let joined_path: AssetPath;
			
			let real_asset_path = if asset_path.is_absolute() {
				asset_path
			} else {
				joined_path_buf = base_program_path.parent().unwrap().join(asset_path).unwrap();
				joined_path = joined_path_buf.as_path();
				&joined_path
			};
			
			// Resolve the file path
			let file_path = unsafe {&ASSET_MANAGER_INSTANCE}.resolve_asset_fs_path(real_asset_path);
			
//			// DEBUG: Print file path of ssl source
//			println!("Loading source file {:?}", file_path);
			
			// Read the file to buffer
			let mut file = OpenOptions::new().read(true).open(file_path.as_path())?;
			
			let mut buffer = String::new();
			file.read_to_string(&mut buffer)?;
			
			Ok(buffer)
		};
		
		// Parse the includes
//		let mut parsed_includes = HashMap::<String, ParsedSource>::new();
		let mut parsed_includes = Vec::<ParsedSource>::new();
		
//		if let Some(include_defs) = &program_def.includes {
			for include_path in &program_def.includes /*include_defs*/ {
				let asset_path = AssetPathBuf::from(include_path.as_str());
				let source_code = load_asset_as_str(&asset_path.as_path(), &program_asset_path)?;
				
				let parsed_source = source_parser.parse_source(source_code);
//				parsed_includes.insert(String::from(&parsed_source.namespace), parsed_source);
				parsed_includes.push(parsed_source);
			}
//		}
		
		// Create program object
		let program = {
			self.program_object = Some(ShaderProgram::new());
			self.program_object.as_mut().unwrap() // SAFETY: Is always Some because it's replaced by the line above
		};
		
		// Load the shaders
		for shader_def in &program_def.shaders {
			// Load the referenced code
			let asset_path = AssetPathBuf::from(&shader_def.source);
			let source_code = load_asset_as_str(&asset_path.as_path(), &program_asset_path)?;
			
			let parsed_source = source_parser.parse_source(source_code);
			
			// Setup the transpiler
			let mut transpiler = SSLTranspiler::new();
			
			// Add the includes
			for include in &parsed_includes {
				transpiler.add_include(include);
			}
			
			// Transpile the shader
			let transpiled_code = transpiler.transpile(&parsed_source);
			
			// Create the shader object
			let mut shader = Shader::new(shader_def.stage.as_engine_stage_enum());
			
			let shader_source = ShaderCode::new(Vec::from(transpiled_code.source_code.as_bytes()));
			shader.attach_source(shader_source);
			
			// Attach shader
			program.attach_shader(shader);
		}
		
		// Mark recompile needed
		self.mark_recompile_needed();
		
		// Return ok
		Ok(())
	}
	
	pub fn new(program_asset_path: Option<AssetPathBuf>) -> Self {
		Self {
			program_asset_path,
			
			program_object: None,
			needs_recompile: false,
		}
	}
	
//	#[deprecated]
//	pub fn new_from_file(vertex_file: &Path, fragment_file: &Path, tess_eval_file: Option<&Path>) -> ShaderProgram {
//		fn load_shader(stage: ShaderStage, )
//		
//		let vertex_shader = Shader::new(ShaderStage::Vertex);
//		let fragment_shader = Shader::new(ShaderStage::Fragment);
//		
//		let tess_eval_shader = if let Some(file) = tess_eval_file {
//			Some(Shader::new(ShaderStage::TessellationEval))
//		}
//		else {None};
//		
//		let mut program = ShaderProgram::new();
//		
//		program
//	}
}
