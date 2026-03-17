pub mod context;
pub mod panels;
pub mod status_bar;
pub mod theme;

use crate::gui::context::AppContext;
use crate::gui::panels::console::ConsolePanel;
use crate::gui::panels::scripts::ScriptsPanel;
use crate::gui::panels::settings::SettingsPanel;
use crate::gui::panels::GuiPanel;
use crate::gui::status_bar::StatusBar;
use anyhow::Result;
use std::num::NonZeroU32;

use glow::HasContext;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use imgui_glow_renderer::AutoRenderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct GlState {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    renderer: AutoRenderer,
    imgui: imgui::Context,
    platform: WinitPlatform,
}

struct App {
    gl_state: Option<GlState>,
    app_ctx: AppContext,
    panels: Vec<Box<dyn GuiPanel>>,
    status_bar: StatusBar,
}

impl App {
    fn new(app_ctx: AppContext) -> Self {
        let panels: Vec<Box<dyn GuiPanel>> = vec![
            Box::new(ConsolePanel::new()),
            Box::new(ScriptsPanel::new()),
            Box::new(SettingsPanel::new()),
        ];
        Self {
            gl_state: None,
            app_ctx,
            panels,
            status_bar: StatusBar::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gl_state.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title(window_title(&self.app_ctx))
            .with_inner_size(LogicalSize::new(1100u32, 700u32));

        let config_template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attrs));

        let (window, gl_config) = display_builder
            .build(event_loop, config_template, |configs| {
                configs
                    .reduce(|a, b| {
                        if a.num_samples() > b.num_samples() {
                            a
                        } else {
                            b
                        }
                    })
                    .unwrap()
            })
            .expect("Failed to build display");

        let window = window.expect("No window created");

        let raw_window_handle = window
            .window_handle()
            .expect("Failed to get window handle")
            .as_raw();
        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .expect("Failed to create GL context")
        };

        let size = window.inner_size();
        let attrs =
            glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new().build(
                raw_window_handle,
                NonZeroU32::new(size.width.max(1)).unwrap(),
                NonZeroU32::new(size.height.max(1)).unwrap(),
            );
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &attrs)
                .expect("Failed to create GL surface")
        };

        let gl_context = gl_context
            .make_current(&gl_surface)
            .expect("Failed to make GL context current");

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .ok();

        let glow_context = unsafe {
            glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s).cast())
        };

        // ImGui setup
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        load_system_font(&mut imgui, 1.0);

        let mut platform = WinitPlatform::new(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

        let scale = platform.hidpi_factor() as f32;
        theme::apply(&mut imgui, scale);

        let renderer = AutoRenderer::new(glow_context, &mut imgui)
            .expect("Failed to create renderer");

        // Log initial state
        if self.app_ctx.connected() {
            self.app_ctx
                .log_success(format!("Connected to game pipe: {}", self.app_ctx.pipe_name));
        } else {
            self.app_ctx.log_warn(format!(
                "Could not connect to '{}'. Running in offline mode.",
                self.app_ctx.pipe_name
            ));
        }
        let script_count = self.app_ctx.total_script_count();
        if script_count > 0 {
            self.app_ctx
                .log_info(format!("{} script(s) registered.", script_count));
        }

        self.gl_state = Some(GlState {
            window,
            gl_context,
            gl_surface,
            renderer,
            imgui,
            platform,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(gl) = self.gl_state.as_mut() else {
            return;
        };

        // Wrap WindowEvent into Event for imgui-winit-support
        let wrapped_event: Event<()> = Event::WindowEvent {
            window_id: _window_id,
            event: event.clone(),
        };
        gl.platform
            .handle_event(gl.imgui.io_mut(), &gl.window, &wrapped_event);

        match event {
            WindowEvent::CloseRequested => {
                self.app_ctx.disconnect();
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    gl.gl_surface.resize(
                        &gl.gl_context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
                }
            }
            WindowEvent::RedrawRequested => {
                gl.platform
                    .prepare_frame(gl.imgui.io_mut(), &gl.window)
                    .expect("Failed to prepare frame");
                let ui = gl.imgui.new_frame();

                // Full-window ImGui window using io display size
                let display_size = ui.io().display_size;
                ui.window("##main")
                    .position([0.0, 0.0], imgui::Condition::Always)
                    .size(display_size, imgui::Condition::Always)
                    .no_decoration()
                    .movable(false)
                    .bring_to_front_on_focus(false)
                    .build(|| {
                        let status_bar_height = ui.frame_height_with_spacing() + 4.0;

                        if let Some(_tab_bar) = ui.tab_bar("##mainTabs") {
                            for panel in self.panels.iter_mut() {
                                if let Some(_tab) = ui.tab_item(panel.title()) {
                                    let panel_height =
                                        ui.content_region_avail()[1] - status_bar_height;
                                    if let Some(_child) = ui
                                        .child_window("##tabContent")
                                        .size([0.0, panel_height])
                                        .border(false)
                                        .begin()
                                    {
                                        panel.render(ui, &mut self.app_ctx);
                                    }
                                }
                            }
                        }

                        self.status_bar.render(ui, &self.app_ctx);
                    });

                gl.platform.prepare_render(ui, &gl.window);
                let draw_data = gl.imgui.render();

                unsafe {
                    let gl_ctx = gl.renderer.gl_context();
                    gl_ctx.clear(glow::COLOR_BUFFER_BIT);
                }
                gl.renderer.render(draw_data).expect("Failed to render");

                gl.gl_surface
                    .swap_buffers(&gl.gl_context)
                    .expect("Failed to swap buffers");

                // Check if exit was requested via command
                if self.app_ctx.exit_requested {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(gl) = &self.gl_state {
            gl.window.request_redraw();
        }
    }
}

/// Run the ImGui application. Blocks the calling thread until the window is closed.
pub fn run(app_ctx: AppContext) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new(app_ctx);
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn window_title(ctx: &AppContext) -> String {
    if ctx.connected() {
        format!("BotWithUs \u{2014} {}", ctx.pipe_name)
    } else {
        "BotWithUs \u{2014} disconnected".to_string()
    }
}

fn load_system_font(imgui: &mut imgui::Context, scale: f32) {
    let ui_size = (19.0 * scale).round();
    let font_dir = std::env::var("WINDIR")
        .unwrap_or_else(|_| "C:\\Windows".to_string())
        + "\\Fonts";

    let candidates = ["segoeui.ttf", "arial.ttf", "verdana.ttf"];

    for name in &candidates {
        let path = format!("{}\\{}", font_dir, name);
        if std::path::Path::new(&path).exists() {
            if let Ok(data) = std::fs::read(&path) {
                imgui.fonts().add_font(&[imgui::FontSource::TtfData {
                    data: &data,
                    size_pixels: ui_size,
                    config: Some(imgui::FontConfig {
                        oversample_h: 3,
                        oversample_v: 3,
                        pixel_snap_h: true,
                        ..Default::default()
                    }),
                }]);
                return;
            }
        }
    }

    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                size_pixels: ui_size,
                ..Default::default()
            }),
        }]);
}
