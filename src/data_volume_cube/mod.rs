use glium::index::{NoIndices, PrimitiveType};
use glium::{implement_vertex, uniform, Display, DrawParameters, Frame, Program, Surface};
use glium::vertex::VertexBuffer;
use glutin::surface::WindowSurface;

#[derive(Copy, Clone)]
struct Vertices {
    position: [f32; 3],
}

implement_vertex!(Vertices, position);

pub struct DataVolumeBoundingBoxRenderer <'a> {
    vertex_buffer: VertexBuffer<Vertices>,
    //indices: 
    program: Program,
    params: &'a DrawParameters<'a>
}

impl <'a> DataVolumeBoundingBoxRenderer <'a> {
    pub fn new(display: &Display<WindowSurface>, params: &'a DrawParameters) -> Self {
        let data = &[
            Vertices { position: [0.0, 0.0, 0.0] },
            Vertices { position: [1.0, 0.0, 0.0] },
            Vertices { position: [1.0, 1.0, 0.0] },
            Vertices { position: [0.0, 1.0, 0.0] },
    
            Vertices { position: [0.0, 0.0, 0.0] },
            Vertices { position: [0.0, 0.0, 1.0] },
            Vertices { position: [1.0, 0.0, 1.0] },
            Vertices { position: [1.0, 0.0, 0.0] },
    
            Vertices { position: [1.0, 0.0, 1.0] },
            Vertices { position: [1.0, 1.0, 1.0] },
            Vertices { position: [1.0, 1.0, 0.0] },
            Vertices { position: [1.0, 1.0, 1.0] },
    
            Vertices { position: [0.0, 1.0, 1.0] },
            Vertices { position: [0.0, 1.0, 0.0] },
            Vertices { position: [0.0, 1.0, 1.0] },
            Vertices { position: [0.0, 0.0, 1.0] },
        ];

        let vertex_buffer = glium::vertex::VertexBuffer::new(display, data).unwrap();

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

        let program = Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

        DataVolumeBoundingBoxRenderer {
            vertex_buffer,
            program,
            params
        }
    }


    pub fn draw(&self, target: &mut Frame, mvp_matrix: [[f32; 4]; 4]) {
        target.draw(&self.vertex_buffer,
            &NoIndices(PrimitiveType::LineStrip),
            &self.program,
            &uniform! { mvpMatrix: mvp_matrix },
            &self.params).unwrap();
    }
}