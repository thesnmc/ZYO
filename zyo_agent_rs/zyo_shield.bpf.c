#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/if_ether.h>
#include <linux/ip.h>

// The Ring-0 Drop List (Memory Map)
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 10000);
    __type(key, __u32);   // The Malicious IP Address
    __type(value, __u8);  // 1 = Drop
} drop_list SEC(".maps");

SEC("xdp")
int zyo_xdp_firewall(struct xdp_md *ctx) {
    void *data_end = (void *)(long)ctx->data_end;
    void *data = (void *)(long)ctx->data;

    // 1. Parse the Ethernet Header
    struct ethhdr *eth = data;
    if ((void *)(eth + 1) > data_end)
        return XDP_PASS;

    // We only care about IP packets (IPv4)
    if (eth->h_proto != __builtin_bswap16(ETH_P_IP))
        return XDP_PASS;

    // 2. Parse the IP Header
    struct iphdr *ip = (void *)(eth + 1);
    if ((void *)(ip + 1) > data_end)
        return XDP_PASS;

    // 3. Look up the Source IP in our AI Drop List
    __u32 src_ip = ip->saddr;
    __u8 *action = bpf_map_lookup_elem(&drop_list, &src_ip);

    // 4. If the AI flagged it, vaporize it!
    if (action && *action == 1) {
        return XDP_DROP; // Packet deleted in Ring-0
    }

    return XDP_PASS; // Safe packet, let it through to Linux
}

char _license[] SEC("license") = "GPL";
