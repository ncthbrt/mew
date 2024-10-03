

mod MyLib {
    mod Types2 {
        struct MyLib_Types2_Uniforms {
            modelMatrix: array<mat4x4f, 5>
        }
        
        struct MyLib_Types2_Camera {
            viewProjectionMatrix: mat4x4f
        }
        
        const MyLib_Types2_x: MyLib_Types2_Camera = MyLib_Types2_Camera();
    }
    
    @binding(0) @group(0)
    var<uniform> MyLib_uniforms: MyLib_Types2_Uniforms;
    
    @binding(1) @group(0)
    var<uniform> MyLib_camera: MyLib_Camera;
    
    @vertex
    fn MyLib_main2(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> MyLib_VertexOutput {
        let uni = MyLib_Types2_Uniforms();
        var output: MyLib_VertexOutput;
        output.Position = MyLib_camera.viewProjectionMatrix * MyLib_uniforms.modelMatrix[instanceIdx] * position;
        output.fragColor = color / vec4<f32>(MyLib_Types2_x);
        return output;
    }
    
    struct MyLib_Uniforms {
        modelMatrix: array<mat4x4f, 5>
    }
    
    struct MyLib_Camera {
        viewProjectionMatrix: mat4x4f
    }
    
    @fragment
    fn MyLib_util__main() -> u32 {
        return MyLib_util_my__fn();
    }
    
    mod util {
        fn MyLib_util_my__fn() -> u32 {
            return 42;
        }
    }
    
    struct MyLib_VertexOutput {
        @builtin(position)
        Position: vec4f,
        @location(0)
        fragColor: vec4f
    }
    
    @vertex
    fn MyLib_main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> MyLib_VertexOutput {
        var output: MyLib_VertexOutput;
        output.Position = MyLib_camera.viewProjectionMatrix * MyLib_uniforms.modelMatrix[instanceIdx] * position;
        output.fragColor = color;
        return output;
    }
}
