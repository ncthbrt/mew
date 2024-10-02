

@vertex
fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4<f32>, @location(1) color: vec4f) -> My__Lib_VertexShader_Types_VertexOutput__Hey {
    let uni = array<mat4x4f, 5>();
    var output: My__Lib_VertexShader_Types_VertexOutput__Hey;
    output.Position = My__Lib_VertexShader_camera__Hey__WhatsUp__Hi__Hi.viewProjectionMatrix * My__Lib_VertexShader_uniforms__Hey__WhatsUp__Hi__Hi.modelMatrix[instanceIdx] * position;
    output.fragColor = color / vec4<f32>(My__Lib_x__Hey__WhatsUp__Hi);
    return output;
}

struct My__Lib_VertexShader_Types_VertexOutput__Hey {
    @builtin(position)
    Position: vec4f,
    @location(0)
    fragColor: vec4f
}

struct My__Lib_VertexShader_Types_Uniforms__Hi {
    modelMatrix: array<mat4x4f, 5>
}

@binding(1) @group(0)
var<uniform> My__Lib_VertexShader_camera__Hey__WhatsUp__Hi__Hi: array<mat4x4f, 5>;

@binding(0) @group(0)
var<uniform> My__Lib_VertexShader_uniforms__Hey__WhatsUp__Hi__Hi: My__Lib_VertexShader_Types_Uniforms__Hi;

const My__Lib_x__Hey__WhatsUp__Hi: array<mat4x4f, 5> = array<mat4x4f, 5>();
