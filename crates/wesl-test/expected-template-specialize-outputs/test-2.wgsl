

@vertex
fn test__2_My__Lib_VertexShader_main__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi__test__95__2__58__58__Hi(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4<f32>, @location(1) color: vec4f) -> test__2_My__Lib_VertexShader_Types_VertexOutput__test__95__2__58__58__Hey {
    let uni = test__2_Camera__mat4x4f();
    var output: test__2_My__Lib_VertexShader_Types_VertexOutput__test__95__2__58__58__Hey;
    output.Position = test__2_My__Lib_VertexShader_camera__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi__test__95__2__58__58__Hi.viewProjectionMatrix * test__2_My__Lib_VertexShader_uniforms__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi__test__95__2__58__58__Hi.modelMatrix.viewProjectionMatrix[instanceIdx] * position;
    output.fragColor = color / vec4<f32>(test__2_My__Lib_x__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi);
    return output;
}

@binding(0) @group(0)
var<uniform> test__2_My__Lib_VertexShader_uniforms__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi__test__95__2__58__58__Hi: test__2_My__Lib_VertexShader_Types_Uniforms__test__95__2__58__58__Hi;

struct test__2_My__Lib_VertexShader_Types_Uniforms__test__95__2__58__58__Hi {
    modelMatrix: test__2_Camera__mat4x4f
}

struct test__2_My__Lib_VertexShader_Types_VertexOutput__test__95__2__58__58__Hey {
    @builtin(position)
    Position: vec4f,
    @location(0)
    fragColor: vec4f
}

@binding(1) @group(0)
var<uniform> test__2_My__Lib_VertexShader_camera__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi__test__95__2__58__58__Hi: test__2_Camera__mat4x4f;

const test__2_My__Lib_x__test__95__2__58__58__Hey__test__95__2__58__58__WhatsUp__test__95__2__58__58__Hi: test__2_Camera__mat4x4f = test__2_Camera__mat4x4f();

struct test__2_Camera__mat4x4f {
    viewProjectionMatrix: mat4x4f
}
