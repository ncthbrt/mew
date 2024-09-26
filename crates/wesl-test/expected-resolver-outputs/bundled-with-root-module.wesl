

mod MyLib {
    mod Types2 {
        struct Uniforms {
            modelMatrix: array<mat4x4f, 5>
        }
        
        struct Camera {
            viewProjectionMatrix: mat4x4f
        }
        
        const x: MyLib::Types2::Camera = MyLib::Types2::Camera();
    }
    
    @binding(0) @group(0)
    var<uniform> uniforms: MyLib::Types2::Uniforms;
    
    @binding(1) @group(0)
    var<uniform> camera: MyLib::Camera;
    
    @vertex
    fn main2(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> MyLib::VertexOutput {
        let uni = MyLib::Types2::Uniforms();
        var output: MyLib::VertexOutput;
        output.Position = MyLib::camera.viewProjectionMatrix * MyLib::uniforms.modelMatrix[instanceIdx] * position;
        output.fragColor = color / vec4<f32>(MyLib::Types2::x);
        return output;
    }
    
    struct Uniforms {
        modelMatrix: array<mat4x4f, 5>
    }
    
    struct Camera {
        viewProjectionMatrix: mat4x4f
    }
    
    struct VertexOutput {
        @builtin(position)
        Position: vec4f,
        @location(0)
        fragColor: vec4f
    }
    
    @vertex
    fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> MyLib::VertexOutput {
        var output: MyLib::VertexOutput;
        output.Position = MyLib::camera.viewProjectionMatrix * MyLib::uniforms.modelMatrix[instanceIdx] * position;
        output.fragColor = color;
        return output;
    }
}
