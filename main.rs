#![windows_subsystem = "windows"]

mod gfx;
mod ui;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowId};

struct Running {
    window: Arc<Window>,
    gfx: gfx::Gfx,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    home: ui::Home,
}

#[derive(Default)]
struct App {
    running: Option<Running>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.running.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Notarium")
            .with_inner_size(LogicalSize::new(1200.0, 760.0))
            .with_min_inner_size(LogicalSize::new(900.0, 560.0));
        let window = Arc::new(el.create_window(attrs).expect("falha ao criar janela"));

        let hwnd = match window.window_handle().expect("sem handle").as_raw() {
            RawWindowHandle::Win32(h) => h.hwnd.get(),
            _ => panic!("Notarium so roda no Windows"),
        };
        let size = window.inner_size();
        let gfx = gfx::Gfx::new(hwnd, size.width, size.height).expect("falha ao iniciar DirectX 11");

        let egui_ctx = egui::Context::default();
        ui::apply_theme(&egui_ctx);
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &*window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        window.request_redraw();
        self.running = Some(Running { window, gfx, egui_ctx, egui_state, home: ui::Home::default() });
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(r) = self.running.as_mut() else { return };
        let response = r.egui_state.on_window_event(&r.window, &event);

        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                r.gfx.resize(s.width, s.height);
                r.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let raw = r.egui_state.take_egui_input(&r.window);
                let home = &mut r.home;
                let out = r.egui_ctx.run(raw, |ctx| home.show(ctx));
                r.egui_state.handle_platform_output(&r.window, out.platform_output.clone());
                if home.quit {
                    el.exit();
                    return;
                }
                let ppp = out.pixels_per_point;
                if let Err(e) = r.gfx.render(&r.egui_ctx, out, ppp) {
                    eprintln!("erro de render: {e}");
                }
                r.window.request_redraw();
            }
            _ => {
                if response.repaint {
                    r.window.request_redraw();
                }
            }
        }
    }
}

fn main() {
    let el = EventLoop::new().expect("event loop");
    el.set_control_flow(ControlFlow::Wait);
    el.run_app(&mut App::default()).expect("loop");
}
