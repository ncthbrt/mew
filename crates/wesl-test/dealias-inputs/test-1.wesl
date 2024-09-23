extend My_Lib::VertexShader;

mod My_Lib {
    alias Camera = VertexShader::Types::Camera;
    const x: VertexShader::C = Camera();
    
    mod VertexShader {
        extend Frog;
        
        mod Types {
            struct VertexOutput {
                @builtin(position) Position : vec4f,
                @location(0) fragColor : vec4f,
            }
            struct Uniforms {
                modelMatrix : array<mat4x4f, 5>,
            }
            struct Camera {
                viewProjectionMatrix : mat4x4f,
            }

            alias U = Uniforms;
        }

        alias C = Types::Camera;

        mod Frog {
            const y: f32 = 0f;
        }

        @binding(0) @group(0) var<uniform> uniforms : Types::Uniforms;
        @binding(1) @group(0) var<uniform> camera : Types::Camera;

        @vertex
        fn main(
        @builtin(instance_index) instanceIdx : u32,
        @location(0) position : vec4f,
        @location(1) color : vec4f
        ) -> Types::VertexOutput {
            use Types::{U}
            use My_Lib::x as y;
            let uni = U();
            var output : Types::VertexOutput;
            output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
            output.fragColor = color / vec4<f32>(y);
            return output;
        }
    }
}

