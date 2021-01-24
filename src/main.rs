use wibaeowibtnr::*;
use anyhow::Result;

fn main() -> Result<()> {
    let app_info = ApplicationInfo {
        name: "Test app".into(),
        version: vk::make_version(1, 0, 0),
    };
    let mut setup = default_engine::default_vk_setup(true);
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&event_loop)?;
    let _ = windowed::basics(&app_info, &mut setup, &window)?;
    Ok(())
}
