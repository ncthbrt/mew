extend My_Lib<Hey, WhatsUp>::VertexShader<Hi>;

mod Hey {
    alias T = vec4f;
}

mod Hi {
    alias T = array<mat4x4f, 5>;
}

mod WhatsUp {
    alias T = f32;
}

mod My_Lib<A, B, C = Hi> {
    alias Camera = VertexShader<C>::Types::Camera<Mat = mat4x4f>;
    
    const x: VertexShader<C>::E = VertexShader<C>::E();
    
    mod VertexShader<C> {
        extend Frog;
        
        mod Types {
            struct VertexOutput {
                @builtin(position)
                Position: vec4f,
                @location(0)
                fragColor: A::T
            }
            
            struct Uniforms {
                modelMatrix: C::T
            }
            
            struct Camera<Mat> {
                viewProjectionMatrix: mat4x4f
            }
            
            alias Nested = E;
        }
        
        alias E = C::T;
        
        mod Frog {
            const y: C::T = 0f;
        }
        
        @binding(0) @group(0)
        var<uniform> uniforms: VertexShader<C>::Types::Uniforms;
        
        @binding(1) @group(0)
        var<uniform> camera: Types::Nested;

        @vertex
        fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4<f32>, @location(1) color: vec4f) -> Types::VertexOutput {
            let uni = Types::Nested();
            var output: Types::VertexOutput;
            output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
            output.fragColor = color / vec4<f32>(x);
            return output;
        }
    }
}
