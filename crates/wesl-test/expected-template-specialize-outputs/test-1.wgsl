

@vertex
fn test__1_main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4<f32>, @location(1) color: vec4f) -> test__1_My__Lib_VertexShader_Types_VertexOutput__test____95____1____58____58____Hey {
    use test_1::My_Lib<test_1::Hey, test_1::WhatsUp>::VertexShader<Hi> as V;
    let uni = test__1_Camera__mat4x4f();
    var output: test__1_My__Lib_VertexShader_Types_VertexOutput__test____95____1____58____58____Hey;
    output.Position = test__1_My__Lib_VertexShader_camera__test____95____1____58____58____Hey__test____95____1____58____58____WhatsUp__test____95____1____58____58____Hi__test____95____1____58____58____Hi.viewProjectionMatrix * test__1_My__Lib_VertexShader_uniforms__test____95____1____58____58____Hey__test____95____1____58____58____WhatsUp__test____95____1____58____58____Hi__test____95____1____58____58____Hi.modelMatrix.viewProjectionMatrix[instanceIdx] * position;
    output.fragColor = color / vec4<f32>(test__1_My__Lib_x__test____95____1____58____58____Hey__test____95____1____58____58____WhatsUp__test____95____1____58____58____Hi);
    return output;
}

const test__1_My__Lib_x__test____95____1____58____58____Hey__test____95____1____58____58____WhatsUp__test____95____1____58____58____Hi: test__1_Camera__mat4x4f = test__1_Camera__mat4x4f();

@binding(0) @group(0)
var<uniform> test__1_My__Lib_VertexShader_uniforms__test____95____1____58____58____Hey__test____95____1____58____58____WhatsUp__test____95____1____58____58____Hi__test____95____1____58____58____Hi: test__1_My__Lib_VertexShader_Types_Uniforms__test____95____1____58____58____Hi;

struct test__1_My__Lib_VertexShader_Types_Uniforms__test____95____1____58____58____Hi {
    modelMatrix: test__1_Camera__mat4x4f
}

struct test__1_My__Lib_VertexShader_Types_VertexOutput__test____95____1____58____58____Hey {
    @builtin(position)
    Position: vec4f,
    @location(0)
    fragColor: vec4f
}

@binding(1) @group(0)
var<uniform> test__1_My__Lib_VertexShader_camera__test____95____1____58____58____Hey__test____95____1____58____58____WhatsUp__test____95____1____58____58____Hi__test____95____1____58____58____Hi: test__1_Camera__mat4x4f;

struct test__1_Camera__mat4x4f {
    viewProjectionMatrix: mat4x4f
}
