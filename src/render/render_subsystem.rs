use crate::render::ReconfigureEvent;

pub trait RenderSubsystem {
	fn initialize(&mut self);
	fn deinitialize(&mut self);
	fn reconfigure(&mut self, event: ReconfigureEvent<'_>);
}
