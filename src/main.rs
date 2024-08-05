use std::num::NonZeroU32;

use glium::{Surface, implement_vertex, uniform};
use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{SurfaceAttributesBuilder, WindowSurface},
};
use imgui_winit_support::winit::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder};
use raw_window_handle::HasRawWindowHandle;
use winit::{
    event::{Event, WindowEvent},
    window::Window,
};

const TITLE: &str = "Hello, imgui-rs!";

#[derive(Copy, Clone)]
struct VerticesTest {
    position: [f32; 3],
    //texcoords: [f32; 2]
}

implement_vertex!(VerticesTest, position);

fn main() {
    // Common setup for creating a winit window and imgui context, not specifc
    // to this renderer at all except that glutin is used to create the window
    // since it will give us access to a GL context
    let (event_loop, window, display) = create_window();
    let (mut winit_platform, mut imgui_context) = imgui_init(&window);

    // Create renderer from this crate
    let mut renderer = imgui_glium_renderer::Renderer::init(&mut imgui_context, &display)
        .expect("Failed to initialize renderer");

    let data = &[
        VerticesTest { position: [0.0, 0.0, 0.0] },
        VerticesTest { position: [0.0, 1.0, 0.0] },
        VerticesTest { position: [0.0, 0.0, 0.0] },
        VerticesTest { position: [1.0, 0.0, 0.0] },
    ];

    let vertex_buffer = glium::vertex::VertexBuffer::new(&display, data).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    let vertex_shader_src = r#"
    #version 330

    uniform mat4 mvpMatrix;
    in vec3 position;

    void main() {
        gl_Position = mvpMatrix * vec4(position, 1.0);
    }
"#;

    let fragment_shader_src = r#"
    #version 330

    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    let params = glium::DrawParameters {
    depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        .. Default::default()
    },
    .. Default::default()
    };

    // Timer for FPS calculation
    let mut last_frame = std::time::Instant::now();

    let mut distance_to_camera = 0.0;
    let mut mvp_matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0f32]
    ];

    // Standard winit event loop
    event_loop
        .run(move |event, window_target| match event {
            Event::NewEvents(_) => {
                let now = std::time::Instant::now();
                imgui_context.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::AboutToWait => {
                winit_platform
                    .prepare_frame(imgui_context.io_mut(), &window)
                    .expect("Failed to prepare frame");
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Create frame for the all important `&imgui::Ui`
                let ui = imgui_context.frame();

                // Draw our example content
                ui.show_demo_window(&mut true);

                // Setup for drawing
                let mut target = display.draw();

                // Renderer doesn't automatically clear window
                target.clear_color_srgb_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

                target.draw(&vertex_buffer, &indices, &program, &uniform! { mvpMatrix: mvp_matrix },
                    &params).unwrap();

                // Perform rendering
                winit_platform.prepare_render(ui, &window);
                let draw_data = imgui_context.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");

                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => window_target.exit(),
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    display.resize((new_size.width, new_size.height));
                }
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel {delta, ..},
                ..
            } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        distance_to_camera += y / 500.0
                    },
                    _ => {},
                }
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            },
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                /*match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => {
                        distance_to_camera += y / 500.0
                    },
                    _ => {},
                }*/
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            },
            event => {
                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
        })
        .expect("EventLoop error");
}

fn create_window() -> (EventLoop<()>, Window, glium::Display<WindowSurface>) {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let window_builder = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(LogicalSize::new(1024, 768));

    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .expect("Failed to create OpenGL window");
    let window = window.unwrap();

    let context_attribs = ContextAttributesBuilder::new().build(Some(window.raw_window_handle()));
    let context = unsafe {
        cfg.display()
            .create_context(&cfg, &context_attribs)
            .expect("Failed to create OpenGL context")
    };

    let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.raw_window_handle(),
        NonZeroU32::new(1024).unwrap(),
        NonZeroU32::new(768).unwrap(),
    );
    let surface = unsafe {
        cfg.display()
            .create_window_surface(&cfg, &surface_attribs)
            .expect("Failed to create OpenGL surface")
    };

    let context = context
        .make_current(&surface)
        .expect("Failed to make OpenGL context current");

    let display = glium::Display::from_context_surface(context, surface)
        .expect("Failed to create glium Display");

    (event_loop, window, display)
}

fn imgui_init(window: &Window) -> (imgui_winit_support::WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);

    let mut winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

    let dpi_mode = imgui_winit_support::HiDpiMode::Default;

    winit_platform.attach_window(imgui_context.io_mut(), window, dpi_mode);

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    (winit_platform, imgui_context)
}
