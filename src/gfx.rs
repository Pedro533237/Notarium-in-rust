use windows::core::Interface;
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL_10_0,
    D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0,
};
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
            // Primeiro tenta a GPU normal. Em drivers antigos a criação do
            // dispositivo pode falhar mesmo quando o Direct3D está instalado.
            // Nesse caso o WARP mantém a interface funcionando por software.
            let hardware = create_device(D3D_DRIVER_TYPE_HARDWARE);
            let (device, context) = match hardware {
                Ok((device, context, D3D_FEATURE_LEVEL_11_0)) => (device, context),
                Ok((_, _, level)) => create_device(D3D_DRIVER_TYPE_WARP)
                    .map(|(device, context, _)| (device, context))
                    .map_err(|warp_error| {
                        format!(
                            "não foi possível iniciar uma GPU compatível nem o renderizador por software.\n\nFeature level encontrado: {level:?}\nWARP: {warp_error}"
                        )
                    })?,
                Err(hardware_error) => create_device(D3D_DRIVER_TYPE_WARP)
                    .map(|(device, context, _)| (device, context))
                    .map_err(|warp_error| {
                        format!(
                            "não foi possível iniciar a GPU nem o renderizador por software.\n\nGPU: {hardware_error}\nWARP: {warp_error}"
                        )
                    })?,
            };

            let dxgi: IDXGIDevice = device.cast()?;
            let adapter = dxgi.GetAdapter()?;
            let factory: IDXGIFactory2 = adapter.GetParent()?;

            // FLIP_DISCARD não é suportado por várias GPUs/Drivers antigos,
            // incluindo Intel HD de primeira geração. DISCARD é o caminho
            // legado, porém compatível com D3D11.
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                Width: w.max(1),
                Height: h.max(1),
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 1,
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Scaling: DXGI_SCALING_STRETCH,
                AlphaMode: DXGI_ALPHA_MODE_UNSPECIFIED,
                ..Default::default()
            };

            let swap = factory.CreateSwapChainForHwnd(
                &device,
                HWND(hwnd as _),
                &desc,
                None,
                None,
            )?;
            let renderer = egui_directx11::Renderer::new(&device)
                .map_err(|e| format!("falha ao criar renderizador egui: {e:?}"))?;

            let mut g = Gfx {
                device,
                context,
                swap,
                rtv: None,
                renderer,
            };
            g.make_rtv()?;
            Ok(g)
        }
    }

    unsafe fn make_rtv(&mut self) -> Result<()> {
        let back: ID3D11Texture2D = self.swap.GetBuffer(0)?;
        let mut rtv = None;
        self.device
            .CreateRenderTargetView(&back, None, Some(&mut rtv))?;
        self.rtv = rtv;
        Ok(())
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return;
        }
        unsafe {
            // Libera referências ao backbuffer antes do ResizeBuffers.
            self.rtv = None;
            self.context.OMSetRenderTargets(None, None);
            if self
                .swap
                .ResizeBuffers(0, w, h, DXGI_FORMAT_UNKNOWN, DXGI_SWAP_CHAIN_FLAG(0))
                .is_ok()
            {
                let _ = self.make_rtv();
            }
        }
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        out: egui_directx11::RendererOutput,
        ppp: f32,
    ) -> Result<()> {
        unsafe {
            let Some(rtv) = self.rtv.clone() else {
                return Err("o backbuffer não possui render target".into());
            };
            self.context
                .OMSetRenderTargets(Some(&[Some(rtv.clone())]), None);
            self.context
                .ClearRenderTargetView(&rtv, &[15.0 / 255.0, 18.0 / 255.0, 32.0 / 255.0, 1.0]);
            self.renderer
                .render(&self.context, &rtv, ctx, out, ppp)
                .report()?;
            self.swap.Present(1, DXGI_PRESENT(0)).ok()?;
        }
        Ok(())
    }
}

unsafe fn create_device(
    driver_type: D3D_DRIVER_TYPE,
) -> Result<(ID3D11Device, ID3D11DeviceContext, D3D_FEATURE_LEVEL)> {
    let mut device = None;
    let mut context = None;
    let mut feature_level = D3D_FEATURE_LEVEL_10_0;
    D3D11CreateDevice(
        None,
        driver_type,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        Some(&[
            D3D_FEATURE_LEVEL_11_0,
            D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_10_0,
        ]),
        D3D11_SDK_VERSION,
        Some(&mut device),
        Some(&mut feature_level),
        Some(&mut context),
    )?;
    let device = device.ok_or("sem ID3D11Device")?;
    let context = context.ok_or("sem contexto D3D11")?;
    Ok((device, context, feature_level))
}
