

struct VertexOutput {
    @builtin(position)
    Position: vec4f,
    @location(0)
    fragColor: vec4f
}

@vertex
fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> VertexOutput {
    var output: VertexOutput;
    output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
    output.fragColor = color;
    return output;
}

struct Uniforms {
    modelMatrix: array<mat4x4f, 5>
}

struct Camera {
    viewProjectionMatrix: mat4x4f
}

mod Types2 {
    struct Uniforms {
        modelMatrix: array<mat4x4f, 5>
    }
    
    struct Camera {
        viewProjectionMatrix: mat4x4f
    }
    
    const x: Types2::Camera = Types2::Camera();
}

@binding(0) @group(0)
var<uniform> uniforms: Types2::Uniforms;

@binding(1) @group(0)
var<uniform> camera: Camera;

@vertex
fn main2(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> VertexOutput {
    let uni = Types2::Uniforms();
    var output: VertexOutput;
    output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
    output.fragColor = color / vec4<f32>(Types2::x);
    return output;
}
