

@vertex
fn main(@builtin(instance_index) instanceIdx: u32, @location(0) position: vec4<f32>, @location(1) color: vec4f) -> My_Lib::VertexShader::Types::VertexOutput_Hey {
    let uni = array_mat4x4f_5();
    var output: My_Lib::VertexShader::Types::VertexOutput_Hey;
    output.Position = My_Lib::VertexShader::camera_Hey_WhatsUp_Hi_Hi.viewProjectionMatrix * My_Lib::VertexShader::uniforms_Hey_WhatsUp_Hi_Hi.modelMatrix[instanceIdx] * position;
    output.fragColor = color / vec4<f32>(My_Lib::x_Hey_WhatsUp_Hi);
    return output;
}

mod Hey {

}

mod WhatsUp {

}

mod Hi {

}

mod My_Lib {
    mod VertexShader {
        mod Types {
            struct VertexOutput_Hey {
                @builtin(position)
                Position: vec4f,
                @location(0)
                fragColor: vec4f
            }
            
            struct Uniforms_Hi {
                modelMatrix: array_mat4x4f_5
            }
        }
        
        @binding(1) @group(0)
        var<uniform> camera_Hey_WhatsUp_Hi_Hi: array_mat4x4f_5;
        
        @binding(0) @group(0)
        var<uniform> uniforms_Hey_WhatsUp_Hi_Hi: My_Lib::VertexShader::Types::Uniforms_Hi;
    }
    
    const x_Hey_WhatsUp_Hi: f32 = f32();
}

mod My__Lib_Hey_WhatsUp_Hi {
    mod VertexShader_Hi {
        mod Types {
        
        }
    }
    
    mod VertexShader_WhatsUp {
    
    }
}
