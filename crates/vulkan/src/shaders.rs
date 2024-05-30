pub mod simple_vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "../../assets/shaders/glsl/simple_vertex_shader.vert",
    }
}

pub mod simple_fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "../../assets/shaders/glsl/simple_fragment_shader.frag",
    }
}