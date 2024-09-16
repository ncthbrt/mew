struct VertexOutput {
  @builtin(position) Position : vec4f,
  @location(0) fragColor : vec4f,
}

@vertex
fn main(
  @builtin(instance_index) instanceIdx : u32,
  @location(0) position : vec4f,
  @location(1) color : vec4f
) -> VertexOutput {
  var output : VertexOutput;
  output.Position = camera.viewProjectionMatrix * uniforms.modelMatrix[instanceIdx] * position;
  output.fragColor = color;
  return output;
}