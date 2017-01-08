// Shader generated Textures --------------------------------------------------
pub enum Texture {
    None,
    Floor(f32),
    Static(f32)
}

impl Texture {

    pub fn into_tuple(self) -> (u32, f32, f32) {
        match self {
            Texture::None => (0, 0.0, 0.0),
            Texture::Floor(scale) => (1, scale, 0.0),
            Texture::Static(scale) => (2, scale, 0.0)
        }
    }

}

pub static TEXTURE_FRAGMENT_SHADER_120: &'static [u8] = br#"
    #version 120
    varying vec4 v_Color;
    varying vec2 v_TexCoord;
    uniform vec4 u_Texture;

    void main() {
        gl_FragColor = v_Color;
    }
"#;

pub static TEXTURE_FRAGMENT_SHADER_150: &'static [u8] = br#"
    #version 150 core
    in vec4 v_Color;
    in vec2 v_TexCoord;

    uniform TextureLocals {
        vec4 u_Texture;
    };

    out vec4 o_Color;

    // Wall and Floor Textures ------------------------------------------------
    vec3 hash3( vec2 p ) {
        vec3 q = vec3( dot(p,vec2(127.1,311.7)),
                       dot(p,vec2(269.5,183.3)),
                       dot(p,vec2(419.2,371.9)) );
        return fract(sin(q)*43758.5453);
    }

    float iqnoise( in vec2 x, float u, float v ) {
        vec2 p = floor(x);
        vec2 f = fract(x);

        float k = 1.0+63.0*pow(1.0-v,4.0);

        float va = 0.0;
        float wt = 0.0;
        for( int j=-2; j<=2; j++ )
        for( int i=-2; i<=2; i++ )
        {
            vec2 g = vec2( float(i),float(j) );
            vec3 o = hash3( p + g )*vec3(u,u,1.0);
            vec2 r = g - f + o.xy;
            float d = dot(r,r);
            float ww = pow( 1.0-smoothstep(0.0,1.414,sqrt(d)), k );
            va += o.z*ww;
            wt += ww;
        }

        return va/wt;
    }

    float staticEffect(in vec2 co) {
        return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);
    }

    void main() {

        // Flat
        if (u_Texture.x == 0.0) {
            o_Color = v_Color;

        // Wall / Floor
        } else if (u_Texture.x == 1.0) {
            vec2 uv = v_TexCoord * u_Texture.y;
            float a = iqnoise(16.0 * uv, 0.5, 0.0);
            float b = iqnoise(32.0 * uv, 0.5, 0.0);
            float e = (a * 0.5 + b * 0.5);
            o_Color = vec4(v_Color.xyz * (0.75 + e * 0.25), v_Color.w);

        // Fog
        } else if (u_Texture.x == 2.0) {

            float e = staticEffect(vec2(
                v_TexCoord.x * cos(u_Texture.y),
                v_TexCoord.y * sin(u_Texture.y)
            ));

            o_Color = vec4(v_Color.xyz, v_Color.w * (0.6 + e * 0.4));
        }

    }

"#;

