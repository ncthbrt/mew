

mod MyLib {
    @binding(0) @group(0)
    var<uniform> uniforms: MyLib::Uniforms;
    
    @binding(1) @group(0)
    var<uniform> camera: MyLib::Camera;
    
    struct VertexOutput {
        @builtin(position)
        Position: vec4f,
        @location(0)
        fragColor: vec4f
    }
    
    @vertex
    fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> MyLib::VertexOutput {
        var output: VertexOutput;output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
        output.fragColor = color;
        return output;
    }
    
    struct Uniforms {
        modelMatrix: array<mat4x4f, 5>
    }
    
    struct Camera {
        viewProjectionMatrix: mat4x4f
    }
    
    mod Types {
        struct Uniforms {
            modelMatrix: array<mat4x4f, 5>
        }
        
        struct Camera {
            viewProjectionMatrix: mat4x4f
        }
        
        const x: MyLib::Types::Camera = MyLib::Types::Camera();
    }
    
    @binding(0) @group(0)
    var<uniform> uniforms: MyLib::Types::Uniforms;
    
    @binding(1) @group(0)
    var<uniform> camera: MyLib::Types::Camera;
    
    struct VertexOutput {
        @builtin(position)
        Position: vec4f,
        @location(0)
        fragColor: vec4f
    }
    
    @vertex
    fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4f, @location(1) color: vec4f) -> MyLib::VertexOutput {
        var output: VertexOutput;output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
        output.fragColor = color / vec4<f32>(Types::x);
        return output;
    }
}
