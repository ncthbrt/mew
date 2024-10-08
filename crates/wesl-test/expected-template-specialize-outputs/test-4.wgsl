

struct Sum__F32 {
    sum: f32
}

const ReduceBuffer__ReduceBuffer____threads: u32 = 10u;

const ReduceBuffer__ReduceBuffer____work________size: u32 = 18u;

const ReduceBuffer__ReduceBuffer____block________area: u32 = 4u;

fn ReduceBuffer_reduceSrcBlock__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(a: array<Sum__F32, ReduceBuffer__ReduceBuffer____block________area>) -> Sum__F32 {
    var v = a[0];
    for (var i = 1u; i < ReduceBuffer__ReduceBuffer____block________area; i = i + 1u) {
        v = SumBinaryOp_binaryOp__F32(v, a[i]);
    }
    return v;
}

fn ReduceBuffer_fetchSrcBuffer__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(gridX: u32) -> array<Sum__F32, ReduceBuffer__ReduceBuffer____block________area> {
    let start = ReduceBuffer_u__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads.sourceOffset + (gridX * ReduceBuffer__ReduceBuffer____block________area);
    let end = arrayLength(&ReduceBuffer_src__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads);
    var a = array<Sum__F32, ReduceBuffer__ReduceBuffer____block________area>();
    for (var i = 0u; i < ReduceBuffer__ReduceBuffer____block________area; i = i + 1u) {
        var idx = i + start;
        if idx < end {
            a[i] = SumBinaryOp_loadOp__F32(ReduceBuffer_src__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads[idx]);
        }
        else {
            a[i] = SumBinaryOp_identityOp__F32();
        }
    }
    return a;
}

fn ReduceBuffer_reduceBufferToWork__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(grid: vec2<u32>, localId: u32) {
    var values = ReduceBuffer_fetchSrcBuffer__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(grid.x);
    var v = ReduceBuffer_reduceSrcBlock__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(values);
    ReduceWorkgroup_work__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size[localId] = v;
}

@compute @workgroup_size(workgroup_threads, 1, 1)
fn ReduceBuffer_main__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(@builtin(global_invocation_id) grid: vec3<u32>, @builtin(local_invocation_index) localIndex: u32, @builtin(num_workgroups) numWorkgroups: vec3<u32>, @builtin(workgroup_id) workgroupId: vec3<u32>) {
    ReduceBuffer_reduceBufferToWork__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(grid.xy, localIndex);
    let outDex = workgroupId.x + ReduceBuffer_u__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads.resultOffset;
    ReduceWorkgroup_reduceWorkgroup__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(localIndex);
    if localIndex == 0u {
        ReduceBuffer_out__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads[outDex] = ReduceWorkgroup_work__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size[0];
    }
}

const ReduceBuffer_workgroup__threads = 4u;

@group(0) @binding(11)
var<storage, read_write> ReduceBuffer_debug: array<f32>;

@group(0) @binding(2)
var<storage, read_write> ReduceBuffer_out__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads: array<Sum__F32>;

@group(0) @binding(1)
var<storage, read> ReduceBuffer_src__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads: array<Sum__F32>;

@group(0) @binding(0)
var<uniform> ReduceBuffer_u__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__block__95__95__95__95__area__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads: ReduceBuffer_Uniforms;

struct ReduceBuffer_Uniforms {
    sourceOffset: u32,
    resultOffset: u32
}

fn ReduceWorkgroup_reduceWorkgroup__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size__ReduceBuffer__95__ReduceBuffer__95__95__threads(localId: u32) {
    let workDex = localId << 1u;
    for (var step = 1u; step < ReduceBuffer__ReduceBuffer____threads; step <<= 1u) {
        workgroupBarrier();
        if localId % step == 0u {
            ReduceWorkgroup_work__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size[workDex] = SumBinaryOp_binaryOp__F32(ReduceWorkgroup_work__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size[workDex], ReduceWorkgroup_work__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size[workDex + step]);
        }
    }
}

var<workgroup> ReduceWorkgroup_work__ReduceBuffer__95__ReduceBuffer__95__95__Op__ReduceBuffer__95__ReduceBuffer__95__95__work__95__95__95__95__size: array<Sum__F32, ReduceBuffer__ReduceBuffer____work________size>;

fn SumBinaryOp_binaryOp__F32(a: Sum__F32, b: Sum__F32) -> Sum__F32 {
    return Sum__F32(Intrinsic_add__f32(a.sum, b.sum));
}

fn SumBinaryOp_identityOp__F32() -> Sum__F32 {
    return Sum__F32();
}

fn SumBinaryOp_loadOp__F32(a: Sum__F32) -> Sum__F32 {
    return Sum__F32(a.sum);
}

fn Intrinsic_add__f32(a: f32, b: f32) -> f32 {
    return a + b;
}
