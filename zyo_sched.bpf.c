#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char _license[] SEC("license") = "GPL";

extern void scx_bpf_dsq_insert(struct task_struct *p, u64 dsq_id, u64 slice, u64 enq_flags) __ksym;

#define SCX_DSQ_GLOBAL 0

// The ONLY map we need. Python -> Kernel.
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __type(key, u32);
    __type(value, u64); 
    __uint(max_entries, 1);
    __uint(pinning, LIBBPF_PIN_BY_NAME);
} zyo_tuning_map SEC(".maps");

SEC("struct_ops/zyo_enqueue")
void BPF_PROG(zyo_enqueue, struct task_struct *p, u64 enq_flags) {
    u32 key = 0;
    u64 *ai_weight = bpf_map_lookup_elem(&zyo_tuning_map, &key); 
    u64 slice = 500000; // Priority throughput fallback

    // Apply the exact nanosecond time slice dictated by PyTorch
    if (ai_weight && *ai_weight > 0) {
        slice = *ai_weight; 
    }

    scx_bpf_dsq_insert(p, SCX_DSQ_GLOBAL, slice, enq_flags);
}

SEC(".struct_ops.link")
struct sched_ext_ops zyo_ops = {
    .enqueue   = (void *)zyo_enqueue,
    .name      = "zyo_ai_sched",
};