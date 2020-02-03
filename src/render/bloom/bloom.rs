use crate::render::{RenderSubsystem, ReconfigureEvent, Framebuffer};

pub struct BloomSubsystem {
	max_octaves: u32,
	
	framebuffer_bloom_temp: Framebuffer,
	framebuffers_bloom_levels: Vec<Framebuffer>,
}

impl BloomSubsystem {
	pub fn new() -> BloomSubsystem {
		BloomSubsystem {
			max_octaves: 6,
			framebuffer_bloom_temp: Framebuffer::new(0, 0),
			framebuffers_bloom_levels: Vec::with_capacity(6),
		}
	}
}

impl RenderSubsystem for BloomSubsystem {
	fn initialize(&mut self) {
		// Do nothing
	}
	
	fn deinitialize(&mut self) {
		// Do nothing
	}
	
	fn reconfigure(&mut self, event: ReconfigureEvent<'_>) {
		let base_width = event.resolution.0;
		let base_height = event.resolution.1;
		
		// Resize framebuffers
		self.framebuffer_bloom_temp.resize(base_width, base_height);
		if !self.framebuffer_bloom_temp.is_allocated() {
			self.framebuffer_bloom_temp.allocate();
		}
	}
}

pub struct BloomOctave {
	
}
