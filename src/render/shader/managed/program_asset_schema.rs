
#[allow(non_snake_case)]
pub mod ProgramAssetSchema {
	use crate::render::shader::ShaderStage;
	use serde::{Deserialize};
	
	#[derive(Deserialize)]
	#[serde(rename = "Program")]
	pub struct ProgramDef {
		pub id: String,
		
		#[serde(default = "ProgramDef::default_includes")]
		pub includes: Vec<String>,
		pub shaders: Vec<self::ShaderDef>,
	}
	
	impl ProgramDef {
		pub fn default_includes() -> Vec<String> {
			Vec::new()
		}
	}
	
	#[derive(Deserialize)]
	#[serde(rename = "Shader")]
	pub struct ShaderDef {
		pub stage: self::ShaderStageDef,
		pub source: String,
	}
	
	#[derive(Deserialize, Copy, Clone)]
	#[serde(rename = "Stage")]
	#[allow(non_camel_case_types)]
	pub enum ShaderStageDef {
		Vertex,
		Fragment,
		TessControl,
		TessEvaluation,
		Geometry,
		Compute,
	}
	
	impl ShaderStageDef {
		pub fn as_engine_stage_enum(&self) -> ShaderStage {
			use ShaderStageDef as D;
			use ShaderStage as E;
			
			match self {
				D::Vertex => E::Vertex,
				D::Fragment => E::Fragment,
				D::TessControl => E::TessellationControl,
				D::TessEvaluation => E::TessellationEval,
				D::Geometry => E::Geometry,
				D::Compute => E::Compute,
			}
		}
	}
}
