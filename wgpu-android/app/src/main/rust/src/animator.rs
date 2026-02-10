pub trait Animate {
    fn new(init_info: WgpuStateInitInfo) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn frame(&mut self) -> anyhow::Result<()>;

    fn resize(&mut self, new_size: (u32, u32)) -> anyhow::Result<()>;
}

use log::info;
use wgpu_playground::{triangle_rotation, vsbm, WgpuStateInitInfo};

pub struct RotatingTriangleAnimator {
    state: triangle_rotation::State,
    elapsed: f32,
}

impl Animate for RotatingTriangleAnimator {
    fn new(init_info: WgpuStateInitInfo) -> anyhow::Result<Self> {
        Ok(Self {
            state: pollster::block_on(triangle_rotation::State::new(init_info)),
            elapsed: 0f32,
        })
    }

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
    fn new(init_info: WgpuStateInitInfo) -> anyhow::Result<Self> {
        Ok(Self {
            state: pollster::block_on(vsbm::State::new(init_info)),
        })
    }

    fn frame(&mut self) -> anyhow::Result<()> {
        self.state.update();
        self.state.render(|| {})?;
        Ok(())
    }

    fn resize(&mut self, new_size: (u32, u32)) -> anyhow::Result<()> {
        Ok(self.state.resize(new_size))
    }
}
