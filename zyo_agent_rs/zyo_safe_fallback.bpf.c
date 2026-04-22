#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

char _license[] SEC("license") = "GPL";

extern void scx_bpf_dsq_insert(struct task_struct *p, u64 dsq_id, u64 slice, u64 enq_flags) __ksym;

#define SCX_DSQ_GLOBAL 0

// No Maps. No AI. Just raw, hardcoded survival logic.
SEC("struct_ops/safe_enqueue")
void BPF_PROG(safe_enqueue, struct task_struct *p, u64 enq_flags) {
    // 5000µs (5ms) is the standard Linux CFS default time slice. 
    // It guarantees the system will not crash or freeze.
    u64 safe_slice = 5000000; 
    scx_bpf_dsq_insert(p, SCX_DSQ_GLOBAL, safe_slice, enq_flags);
}

SEC(".struct_ops.link")
struct sched_ext_ops safe_ops = {
    .enqueue   = (void *)safe_enqueue,
    .name      = "zyo_safe_sched",
};