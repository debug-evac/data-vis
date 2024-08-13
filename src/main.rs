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
use nalgebra::{
    Perspective3,
    Point3, 
    base::{Matrix4, Vector3}
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
        VerticesTest { position: [1.0, 0.0, 0.0] },
        VerticesTest { position: [1.0, 1.0, 0.0] },
        VerticesTest { position: [0.0, 1.0, 0.0] },

        VerticesTest { position: [0.0, 0.0, 0.0] },
        VerticesTest { position: [0.0, 0.0, 1.0] },
        VerticesTest { position: [1.0, 0.0, 1.0] },
        VerticesTest { position: [1.0, 0.0, 0.0] },

        VerticesTest { position: [1.0, 0.0, 1.0] },
        VerticesTest { position: [1.0, 1.0, 1.0] },
        VerticesTest { position: [1.0, 1.0, 0.0] },
        VerticesTest { position: [1.0, 1.0, 1.0] },

        VerticesTest { position: [0.0, 1.0, 1.0] },
        VerticesTest { position: [0.0, 1.0, 0.0] },
        VerticesTest { position: [0.0, 1.0, 1.0] },
        VerticesTest { position: [0.0, 0.0, 1.0] },
    ];

    let vertex_buffer = glium::vertex::VertexBuffer::new(&display, data).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LineStrip);

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

    let mut mvp_matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0f32]
    ];

    let mut distance_to_camera = -8.0;
    let mut want_cursor_capture = false;
    let mut rotation_angles = (0.0, 0.0);
    let mut last_position = (0.0, 0.0);

    let aspect_ration = window.inner_size().width as f32 / std::cmp::max(1, window.inner_size().width) as f32;

    let mut projection_matrix: Perspective3<f32> = Perspective3::new(aspect_ration, 45.0, 0.05, 25.0);

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
                //ui.show_demo_window(&mut true);

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
                let aspect_ration: f32 = new_size.width as f32 / std::cmp::max(1, new_size.height) as f32;

                projection_matrix.set_aspect(aspect_ration);
                mvp_matrix = update_mvp_matrix(projection_matrix, distance_to_camera, &rotation_angles).into();

                winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
            }
            Event::WindowEvent {
                event: WindowEvent::MouseWheel {delta, ..},
                ..
            } => {
                if imgui_context.io().want_capture_mouse {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                } else {
                    if let winit::event::MouseScrollDelta::LineDelta(_, y) = delta {
                        distance_to_camera += y / 5.0;
                        mvp_matrix = update_mvp_matrix(projection_matrix, distance_to_camera, &rotation_angles).into();
                        window.request_redraw();
                    }
                }
            },
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button: winit::event::MouseButton::Left, .. },
                ..
            } => {
                if imgui_context.io().want_capture_mouse {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                } else {
                    match state {
                        winit::event::ElementState::Pressed => {
                            last_position = (imgui_context.io().mouse_pos[0], imgui_context.io().mouse_pos[1]);
                            want_cursor_capture = true;
                        },
                        winit::event::ElementState::Released => {
                            want_cursor_capture = false;
                        },
                    }
                }
            },
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                if imgui_context.io().want_capture_mouse || !want_cursor_capture {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                } else {
                    let mouse_dif = ((position.x as f32) - last_position.0, (position.y as f32) - last_position.1);

                    rotation_angles = ((rotation_angles.0 + mouse_dif.0/220.0) % 3.6,
                        (rotation_angles.1 + mouse_dif.1/220.0) % 3.6);

                    last_position = (position.x as f32, position.y as f32);

                    mvp_matrix = update_mvp_matrix(projection_matrix, distance_to_camera, &rotation_angles).into();

                    window.request_redraw();
                }
            },
            event => winit_platform.handle_event(imgui_context.io_mut(), &window, &event),
        })
        .expect("EventLoop error");
}

fn update_mvp_matrix(projection_matrix: Perspective3<f32>, distance_to_camera: f32, rotation_angles: &(f32, f32)) -> Matrix4<f32> {
    let mut mv_matrix: Matrix4<f32> = Matrix4::identity();

    mv_matrix = mv_matrix.append_translation(&Vector3::new(0.0, 0.0, distance_to_camera));
    mv_matrix *= Matrix4::new_rotation_wrt_point(Vector3::new(rotation_angles.1, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
    mv_matrix *= Matrix4::new_rotation_wrt_point(Vector3::new(0.0, rotation_angles.0, 0.0), Point3::new(1.0, 1.0, 1.0));
    mv_matrix = mv_matrix.append_translation(&Vector3::new(-1.0, -1.0, -1.0));
    mv_matrix = mv_matrix.prepend_scaling(2.0);

    projection_matrix.to_homogeneous() * mv_matrix
}

fn create_window() -> (EventLoop<()>, Window, glium::Display<WindowSurface>) {
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let window_builder = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(LogicalSize::new(1600, 900));

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
        NonZeroU32::new(1600).unwrap(),
        NonZeroU32::new(900).unwrap(),
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
