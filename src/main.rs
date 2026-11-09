#![windows_subsystem = "windows"]

mod gfx;
mod ui;

use std::sync::Arc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowBuilder;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND as WinHwnd;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

fn show_error(message: &str) {
    let text: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "Notarium - erro".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = MessageBoxW(
            WinHwnd(std::ptr::null_mut()),
            PCWSTR(text.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn main() {
    // Em builds GUI do Windows, panics não aparecem em um console.
    // Exiba o motivo em uma caixa de diálogo para facilitar o diagnóstico.
    std::panic::set_hook(Box::new(|info| show_error(&info.to_string())));
    let el = EventLoop::new().expect("event loop");
    el.set_control_flow(ControlFlow::Wait);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Notarium")
            .with_inner_size(LogicalSize::new(1200.0, 760.0))
            .with_min_inner_size(LogicalSize::new(900.0, 560.0))
            .build(&el)
            .expect("falha ao criar janela"),
    );

    let hwnd = match window.window_handle().expect("sem handle").as_raw() {
        RawWindowHandle::Win32(h) => h.hwnd.get(),
        _ => panic!("Notarium so roda no Windows"),
    };
    let size = window.inner_size();
    let mut gfx = gfx::Gfx::new(hwnd, size.width, size.height).expect("falha ao iniciar DirectX 11");

    let egui_ctx = egui::Context::default();
    ui::apply_theme(&egui_ctx);
    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &*window,
        Some(window.scale_factor() as f32),
        None,
    );
    let mut home = ui::Home::default();
    window.request_redraw();

    el.run(move |event, elwt| {
        let Event::WindowEvent { window_id, event } = event else { return };
        if window_id != window.id() {
            return;
        }
        let response = egui_state.on_window_event(&window, &event);

        match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(s) => {
                gfx.resize(s.width, s.height);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let raw = egui_state.take_egui_input(&window);
                let out = egui_ctx.run(raw, |ctx| home.show(ctx));
                if home.quit {
                    elwt.exit();
                    return;
                }
                let ppp = out.pixels_per_point;
                let (renderer_output, platform_output, _viewport_output) = egui_directx11::split_output(out);
                egui_state.handle_platform_output(&window, platform_output);
                if let Err(e) = gfx.render(&egui_ctx, renderer_output, ppp) {
                    show_error(&format!("Falha ao renderizar a interface: {e}"));
                    elwt.exit();
                }
            }
            _ => {
                if response.repaint {
                    window.request_redraw();
                }
            }
        }
    })
    .expect("loop");
}
