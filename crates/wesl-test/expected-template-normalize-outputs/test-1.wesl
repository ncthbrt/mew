

mod Hey {
    alias T = vec4f;
}

mod Hi {
    alias T = array<mat4x4f, 5>;
}

mod WhatsUp {
    alias T = f32;
}

mod My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = Hi> {
    alias Camera = My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_C>::Types::Camera<mat4x4f>;
    
    const x: My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_B>::E = My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_B>::E();
    
    mod VertexShader<My__Lib_VertexShader_C> {
        mod Types {
            struct VertexOutput {
                @builtin(position)
                Position: vec4f,
                @location(0)
                fragColor: My__Lib_A::T
            }
            
            struct Uniforms {
                modelMatrix: My__Lib_VertexShader_C::T
            }
            
            struct Camera<Mat> {
                viewProjectionMatrix: mat4x4f
            }
            
            alias Nested = My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::E;
        }
        
        alias E = My__Lib_VertexShader_C::T;
        
        mod Frog {
            const y: My__Lib_VertexShader_C::T = 0f;
        }
        
        @binding(0) @group(0)
        var<uniform> uniforms: My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::Types::Uniforms;
        
        @binding(1) @group(0)
        var<uniform> camera: My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::Types::Nested;
        
        @vertex
        fn main<My__Lib_VertexShader_main_D>(@builtin(instance_index) instanceIdx: u32, @location(0) position: My__Lib_VertexShader_main_D::T, @location(1) color: vec4f) -> My__Lib_VertexShader_main_D::T {
            let uni = My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::Types::Nested();
            var output: My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::Types::VertexOutput;
            output.Position = My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::camera.viewProjectionMatrix * My_Lib<My__Lib_A, My__Lib_B, My__Lib_C = My__Lib_C>::VertexShader<My__Lib_VertexShader_C>::uniforms.modelMatrix[instanceIdx] * position;
            output.fragColor = color / vec4<f32>(My_Lib::x);
            return My__Lib_VertexShader_main_D::T();
        }
    }
}
