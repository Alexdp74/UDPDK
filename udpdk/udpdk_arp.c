#include "udpdk_arp.h"

#define MAX_ARP_ENTRIES 32

struct arp_entry {
	struct in_addr ip;
	struct rte_ether_addr mac;
};

static struct arp_entry ARP_TABLE[MAX_ARP_ENTRIES];
static int arp_table_num_entries = 0;

// add (ip, mac) to the ARP table. Returns 0 on success, -1 on error
int udpdk_arp_add_entry(struct in_addr ip, struct rte_ether_addr mac) {
	if (arp_table_num_entries >= MAX_ARP_ENTRIES) {
		return -1;
	}

	ARP_TABLE[arp_table_num_entries].ip = ip;
	ARP_TABLE[arp_table_num_entries].mac = mac;
	arp_table_num_entries++;

	return 0;
}

// return the ip assoctiated with MAC mac; NULL if not found
struct in_addr* udpdk_arp_lookup_mac(const struct rte_ether_addr* mac) {
	for (int i=0; i<arp_table_num_entries; i++) {
		if (rte_is_same_ether_addr(&ARP_TABLE[i].mac, mac)) {
			return &ARP_TABLE[i].ip;
		}
	}
	return NULL;
}

// return the mac assoctiated with IP ip; NULL if not found
struct rte_ether_addr* udpdk_arp_lookup_ip(const struct in_addr* ip) {
	for (int i=0; i<arp_table_num_entries; i++) {
		if (ARP_TABLE[i].ip.s_addr == ip->s_addr) {
			return &ARP_TABLE[i].mac;
			break;
		}
	}

	return NULL;
}

