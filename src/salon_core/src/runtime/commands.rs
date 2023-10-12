use super::Runtime;

pub struct CommandsContext<'a> {
    encoder: wgpu::CommandEncoder,
    compute_pass: Option<wgpu::ComputePass<'a>>,
}

impl<'a> CommandsContext<'a> {
    pub fn new(runtime: &Runtime) -> Self {
        CommandsContext {
            encoder: runtime
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
            compute_pass: None,
        }
    }

    // TODO: cache render pass as well? (need to compare render pass descriptor)
    pub fn render_pass(&'a mut self, desc: &'a wgpu::RenderPassDescriptor) -> wgpu::RenderPass<'a> {
        if self.compute_pass.is_some() {
            self.compute_pass = None
        }
        self.encoder.begin_render_pass(desc)
    }

    pub fn compute_pass(&'a mut self) -> &'a mut wgpu::ComputePass<'a> {
        if let Some(ref mut c) = self.compute_pass {
            c
        } else {
            self.compute_pass = Some(self.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {label: None}));
            self.compute_pass.as_mut().unwrap()
        }
    }
}
