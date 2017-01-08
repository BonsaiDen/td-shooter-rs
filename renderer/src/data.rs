// External Dependencies ------------------------------------------------------
use gfx;


// Rendering Pipeline ---------------------------------------------------------
pub const POS_COMPONENTS: usize = 2;

gfx_defines! {
    vertex PositionFormat {
        pos: [f32; POS_COMPONENTS] = "pos",
    }

    vertex ScaleFormat {
        pos: [f32; 2] = "scale",
    }

    constant Locals {
        view: [[f32; 4]; 4] = "u_View",
        color: [f32; 4] = "u_Color",
    }

    constant TextureLocals {
        texture: [f32; 4] = "u_Texture",
    }

    vertex ColorFormat {
        color: [f32; 4] = "color",
    }

    vertex TexCoordsFormat {
        uv: [f32; 2] = "uv",
    }
}

gfx_pipeline_base!( pipe_colored {
    pos: gfx::VertexBuffer<PositionFormat>,
    scale: gfx::VertexBuffer<ScaleFormat>,
    locals: gfx::ConstantBuffer<Locals>,
    texture: gfx::ConstantBuffer<TextureLocals>,
    color: gfx::VertexBuffer<ColorFormat>,
    blend_target: gfx::BlendTarget<gfx::format::Srgba8>,
    stencil_target: gfx::StencilTarget<gfx::format::DepthStencil>,
    blend_ref: gfx::BlendRef,
});


// Shaders --------------------------------------------------------------------
pub static TRIANGLE_VERTEX_SHADER_120: &'static [u8] = br#"
    #version 120
    attribute vec2 pos;

    varying vec4 v_Color;
    uniform vec4 u_Color;
    uniform mat4 u_View;

    varying vec2 v_TexCoord;

    void main() {
        v_Color = u_Color;
        gl_Position = u_View * vec4(pos, 0.0, 1.0);
        v_TexCoord = pos * vec2(0.5) + vec2(0.5);
    }
"#;

pub static TRIANGLE_VERTEX_SHADER_150: &'static [u8] = br#"
    #version 150 core
    in vec2 pos;

    out vec4 v_Color;
    out vec2 v_TexCoord;

    uniform Locals {
        mat4 u_View;
        vec4 u_Color;
    };

    void main() {
        v_Color = u_Color;
        gl_Position = u_View * vec4(pos, 0.0, 1.0);
        v_TexCoord = pos * vec2(0.5) + vec2(0.5);
    }
"#;

pub static POINT_VERTEX_SHADER_120: &'static [u8] = br#"
    #version 120
    attribute vec4 color;
    attribute vec2 pos;
    attribute vec2 scale;

    varying vec4 v_Color;
    uniform mat4 u_View;

    void main() {
        v_Color = color;
        gl_PointSize = scale.x;
        gl_Position = u_View * vec4(pos, 0.0, 1.0);

    }
"#;

pub static POINT_VERTEX_SHADER_150: &'static [u8] = br#"
    #version 150 core
    in vec4 color;
    in vec2 pos;
    in vec2 scale;

    out vec4 v_Color;

    uniform Locals {
        mat4 u_View;
    };

    void main() {
        v_Color = color;
        gl_PointSize = scale.x;
        gl_Position = u_View * vec4(pos, 0.0, 1.0);
    }
"#;

pub static DEFAULT_FRAGMENT_SHADER_120: &'static [u8] = br#"
    #version 120
    varying vec4 v_Color;

    void main() {
        gl_FragColor = v_Color;
    }
"#;

pub static DEFAULT_FRAGMENT_SHADER_150: &'static [u8] = br#"
    #version 150 core
    in vec4 v_Color;

    out vec4 o_Color;

    void main() {
        o_Color = v_Color;
    }
"#;

