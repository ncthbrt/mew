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
    alias Camera = VertexShader<C>::Types::Camera<mat4x4f>;
    const x: VertexShader<B>::E = VertexShader<B>::E();
    
    mod VertexShader<C> {
        extend Frog;
        
        mod Types {
            struct VertexOutput {
                @builtin(position) Position : vec4f,
                @location(0) fragColor : A::T,
            }
            struct Uniforms {
                modelMatrix : C::T,
            }
            struct Camera<Mat> {
                viewProjectionMatrix : mat4x4f,
            }

            alias Nested = E;
        }

        alias E = C::T;

        mod Frog {
            const y: C::T = 0f;
        }

        @binding(0) @group(0) var<uniform> uniforms : Types::Uniforms;
        @binding(1) @group(0) var<uniform> camera : Types::Nested;

        @vertex
        fn main<D>(
        @builtin(instance_index) instanceIdx : u32,
        @location(0) position : D::T,
        @location(1) color : vec4f
        ) -> D::T {
            use Types::{Nested}
            use My_Lib::x as y;
            let uni = Nested();
            var output : Types::VertexOutput;
            output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
            output.fragColor = color / vec4<f32>(y);
            return D::T();
        }
    }
}

