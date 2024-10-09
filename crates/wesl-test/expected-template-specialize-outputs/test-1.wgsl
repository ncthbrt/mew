

@vertex
fn test__1_main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4<f32>, @location(1) color: vec4f) -> test__1_My__Lib_VertexShader_Types_VertexOutput__test__95__1__58__58__Hey {
    use test_1::My_Lib<test_1::Hey, test_1::WhatsUp>::VertexShader<Hi> as V;
    let uni = Hi_T();
    var output: test__1_My__Lib_VertexShader_Types_VertexOutput__test__95__1__58__58__Hey;
    output.Position = test__1_My__Lib_VertexShader_camera__test__95__1__58__58__Hey__test__95__1__58__58__WhatsUp__test__95__1__58__58__Hi__Hi.viewProjectionMatrix * test__1_My__Lib_VertexShader_uniforms__test__95__1__58__58__Hey__test__95__1__58__58__WhatsUp__test__95__1__58__58__Hi__Hi.modelMatrix.viewProjectionMatrix[instanceIdx] * position;
    output.fragColor = color / vec4<f32>(test__1_My__Lib_x__test__95__1__58__58__Hey__test__95__1__58__58__WhatsUp__test__95__1__58__58__Hi);
    return output;
}

const test__1_My__Lib_x__test__95__1__58__58__Hey__test__95__1__58__58__WhatsUp__test__95__1__58__58__Hi: test__1_Camera__mat4x4f = test__1_Camera__mat4x4f();

@binding(0) @group(0)
var<uniform> test__1_My__Lib_VertexShader_uniforms__test__95__1__58__58__Hey__test__95__1__58__58__WhatsUp__test__95__1__58__58__Hi__Hi: test__1_My__Lib_VertexShader_Types_Uniforms__Hi;

struct test__1_My__Lib_VertexShader_Types_Uniforms__Hi {
    modelMatrix: Hi_T
}

struct test__1_My__Lib_VertexShader_Types_VertexOutput__test__95__1__58__58__Hey {
    @builtin(position)
    Position: vec4f,
    @location(0)
    fragColor: vec4f
}

struct test__1_My__Lib_VertexShader_Types_VertexOutput__test__95__1__58__58__Hey {
    @builtin(position)
    Position: vec4f,
    @location(0)
    fragColor: vec4f
}

@binding(1) @group(0)
var<uniform> test__1_My__Lib_VertexShader_camera__test__95__1__58__58__Hey__test__95__1__58__58__WhatsUp__test__95__1__58__58__Hi__Hi: Hi_T;

struct test__1_Camera__mat4x4f {
    viewProjectionMatrix: mat4x4f
}
