//
// Created by plaublin on 18/7/22.
// Copyright (c) 2022 Pierre Louis Aublin. All rights reserved.
//

#ifndef UDPDK_ARP_H
#define UDPDK_ARP_H

#include <rte_ether.h>
#include <netinet/in.h>

// add (ip, mac) to the ARP table. Returns 0 on success, -1 on error
int udpdk_arp_add_entry(struct in_addr ip, struct rte_ether_addr mac);

// return the ip assoctiated with MAC mac; NULL if not found
struct in_addr* udpdk_arp_lookup_mac(const struct rte_ether_addr* mac);

// return the mac assoctiated with IP ip; NULL if not found
struct rte_ether_addr* udpdk_arp_lookup_ip(const struct in_addr* ip);

#endif
