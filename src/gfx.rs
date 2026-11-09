//! Camada DirectX 11: dispositivo, swapchain e render do egui.
//! Se a API do `egui-directx11` na sua versao for diferente, so este arquivo muda.

use windows::core::Interface;
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Aceita tanto `()` quanto `Result<(), E>` como retorno de `Renderer::render`.
trait Report {
    fn report(self) -> Result<()>;
}
impl Report for () {
    fn report(self) -> Result<()> {
        Ok(())
    }
}
impl<E: std::fmt::Debug> Report for std::result::Result<(), E> {
    fn report(self) -> Result<()> {
        self.map_err(|e| format!("{e:?}").into())
    }
}

pub struct Gfx {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap: IDXGISwapChain1,
    rtv: Option<ID3D11RenderTargetView>,
    renderer: egui_directx11::Renderer,
}

impl Gfx {
    pub fn new(hwnd: isize, w: u32, h: u32) -> Result<Self> {
        unsafe {
            let mut device = None;
            let mut context = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
            let device = device.ok_or("sem ID3D11Device")?;
            let context = context.ok_or("sem contexto D3D11")?;

            let dxgi: IDXGIDevice = device.cast()?;
            let adapter = dxgi.GetAdapter()?;
            let factory: IDXGIFactory2 = adapter.GetParent()?;

            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: w.max(1),
                Height: h.max(1),
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                ..Default::default()
            };
            let swap = factory.CreateSwapChainForHwnd(&device, HWND(hwnd as _), &desc, None, None)?;

            let renderer = egui_directx11::Renderer::new(&device).map_err(|e| format!("{e:?}"))?;
            let mut g = Gfx { device, context, swap, rtv: None, renderer };
            g.make_rtv()?;
            Ok(g)
        }
    }

    unsafe fn make_rtv(&mut self) -> Result<()> {
        let back: ID3D11Texture2D = self.swap.GetBuffer(0)?;
        let mut rtv = None;
        self.device.CreateRenderTargetView(&back, None, Some(&mut rtv))?;
        self.rtv = rtv;
        Ok(())
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        unsafe {
            self.rtv = None; // libera referencias ao backbuffer antes do resize
            self.context.OMSetRenderTargets(None, None);
            if self.swap.ResizeBuffers(0, w, h, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0)).is_ok() {
                let _ = self.make_rtv();
            }
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, out: egui_directx11::RendererOutput, ppp: f32) -> Result<()> {
        unsafe {
            let Some(rtv) = self.rtv.clone() else { return Ok(()) };
            self.context.OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            self.context.ClearRenderTargetView(&rtv, &[0.059, 0.071, 0.125, 1.0]);

            // Ponto de integracao com o egui-directx11 (ajuste aqui se a assinatura mudar):
            self.renderer.render(&self.context, &rtv, ctx, out, ppp).report()?;

            self.swap.Present(1, DXGI_PRESENT(0)).ok()?;
        }
        Ok(())
    }
}
