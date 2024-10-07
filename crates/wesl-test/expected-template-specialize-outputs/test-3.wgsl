extend ReduceBuffer<ReduceBuffer_Op = SumBinaryOp<SumBinaryOp_N = F32>, ReduceBuffer_BlockArea = BlockArea, ReduceBuffer_WorkSize = WorkSize, ReduceBuffer_Threads = Threads>;


fn ReduceBuffer_reduceSrcBlock__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(a: array<Sum_T__F32, BlockArea_value>) -> Sum_T__F32 {
    var v = a[0];
    for (var i = 1u; i < BlockArea_value; i = i + 1u) {
        v = SumBinaryOp_binaryOp__F32(v, a[i]);
    }
    return v;
}

fn ReduceBuffer_fetchSrcBuffer__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(gridX: u32) -> array<Sum_T__F32, BlockArea_value> {
    let start = ReduceBuffer_u__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads.sourceOffset + (gridX * BlockArea_value);
    let end = arrayLength(&ReduceBuffer_src__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads);
    var a = array<Sum_T__F32, BlockArea_value>();
    for (var i = 0u; i < BlockArea_value; i = i + 1u) {
        var idx = i + start;
        if idx < end {
            a[i] = SumBinaryOp_loadOp__F32(ReduceBuffer_src__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads[idx]);
        }
        else {
            a[i] = SumBinaryOp_identityOp__F32();
        }
    }
    return a;
}

fn ReduceBuffer_reduceBufferToWork__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(grid: vec2<u32>, localId: u32) {
    var values = ReduceBuffer_fetchSrcBuffer__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(grid.x);
    var v = ReduceBuffer_reduceSrcBlock__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(values);
    ReduceWorkgroup_work__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize[localId] = v;
}

@compute @workgroup_size(workgroupThreads, 1, 1)
fn ReduceBuffer_main__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(@builtin(global_invocation_id) grid: vec3<u32>, @builtin(local_invocation_index) localIndex: u32, @builtin(num_workgroups) numWorkgroups: vec3<u32>, @builtin(workgroup_id) workgroupId: vec3<u32>) {
    ReduceBuffer_reduceBufferToWork__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads(grid.xy, localIndex);
    let outDex = workgroupId.x + ReduceBuffer_u__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads.resultOffset;
    ReduceWorkgroup_reduceWorkgroup__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize__Threads(localIndex);
    if localIndex == 0u {
        ReduceBuffer_out__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads[outDex] = ReduceWorkgroup_work__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize[0];
    }
}

const ReduceBuffer_workgroupThreads = 4u;

@group(0) @binding(11)
var<storage, read_write> ReduceBuffer_debug: array<f32>;

@group(0) @binding(2)
var<storage, read_write> ReduceBuffer_out__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads: array<Sum_T__F32>;

@group(0) @binding(1)
var<storage, read> ReduceBuffer_src__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads: array<Sum_T__F32>;

@group(0) @binding(0)
var<uniform> ReduceBuffer_u__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__BlockArea__WorkSize__Threads: ReduceBuffer_Uniforms;

struct ReduceBuffer_Uniforms {
    sourceOffset: u32,
    resultOffset: u32
}

fn ReduceWorkgroup_reduceWorkgroup__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize__Threads(localId: u32) {
    let workDex = localId << 1u;
    for (var step = 1u; step < Threads_value; step <<= 1u) {
        workgroupBarrier();
        if localId % step == 0u {
            ReduceWorkgroup_work__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize[workDex] = SumBinaryOp_binaryOp__F32(ReduceWorkgroup_work__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize[workDex], ReduceWorkgroup_work__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize[workDex + step]);
        }
    }
}

var<workgroup> ReduceWorkgroup_work__SumBinaryOp__60__SumBinaryOp__95__N__61__F32__62__WorkSize: array<Sum_T__F32, WorkSize_value>;

const Threads_value: u32 = 10u;

fn SumBinaryOp_binaryOp__F32(a: Sum_T__F32, b: Sum_T__F32) -> Sum_T__F32 {
    return Sum_T__F32(F32_add(a.sum, b.sum));
}

fn SumBinaryOp_identityOp__F32() -> Sum_T__F32 {
    return Sum_T__F32();
}

fn SumBinaryOp_loadOp__F32(a: Sum_T__F32) -> Sum_T__F32 {
    return Sum_T__F32(a.sum);
}

fn F32_add(a: f32, b: f32) -> f32 {
    return a + b;
}

struct Sum_T__F32 {
    sum: f32
}

const WorkSize_value: u32 = 18u;

const BlockArea_value: u32 = 4u;
