pub trait Animate {
    fn frame(&mut self) -> anyhow::Result<()>;

    fn resize(&mut self, new_size: (u32, u32)) -> anyhow::Result<()>;
}

use crate::default;
use wgpu_playground::{triangle_rotation, vsbm, WgpuStateInitInfo};

pub struct RotatingTriangleAnimator {
    state: triangle_rotation::State,
    elapsed: f32,
}

impl RotatingTriangleAnimator {
    pub fn new(init_info: WgpuStateInitInfo) -> anyhow::Result<Self> {
        Ok(Self {
            state: pollster::block_on(triangle_rotation::State::new(init_info)),
            elapsed: 0f32,
        })
    }
}

impl Animate for RotatingTriangleAnimator {
        fn frame(&mut self) -> anyhow::Result<()> {
        self.elapsed += 0.005;
        self.state.update_elapsed(self.elapsed);
        self.state.render(|| {});
        Ok(())
    }

    fn resize(&mut self, new_size: (u32, u32)) -> anyhow::Result<()> {
        Ok(self.state.resize(new_size))
    }
}

pub struct VsbmAnimator {
    state: vsbm::State,
}

impl Animate for VsbmAnimator {
        fn frame(&mut self) -> anyhow::Result<()> {
        self.state.update();
        self.state.render(|| {})?;
        Ok(())
    }

    fn resize(&mut self, new_size: (u32, u32)) -> anyhow::Result<()> {
        Ok(self.state.resize(new_size))
    }
}

impl VsbmAnimator {
    pub fn new(init_info: WgpuStateInitInfo) -> anyhow::Result<Self> {
        Ok(Self {
            state: pollster::block_on(vsbm::State::new(
                init_info,
                vsbm::Config {
                    kernel_iterations: 2,
                },
            )),
        })
    }
}

pub struct ShadertoyAnimator {
    state: shadertoy_wgpu::State,
}

impl ShadertoyAnimator {
    pub fn new(init_info: WgpuStateInitInfo, code: &str) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let instance = init_info.instance;
        let adapter = pollster::block_on(instance.request_adapter(&default!()))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&default!()))?;

        let state = shadertoy_wgpu::State::new(
            device,
            queue,
            adapter,
            init_info.size,
            code,
            shadertoy_wgpu::RenderTargetInfo::Surface(init_info.surface),
        );
        let state = pollster::block_on(state);
        Ok(Self { state })
    }
}

impl Animate for ShadertoyAnimator {
    fn frame(&mut self) -> anyhow::Result<()> {
        self.state.frame(|| {})?;
        Ok(())
    }

    fn resize(&mut self, new_size: (u32, u32)) -> anyhow::Result<()> {
        self.state.resize(new_size);
        Ok(())
    }
}
