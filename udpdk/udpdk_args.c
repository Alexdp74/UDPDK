//
// Created by leoll2 on 10/6/20.
// Copyright (c) 2020 Leonardo Lai. All rights reserved.
//
#include <stdio.h>
#include <arpa/inet.h>  // for inet_addr

#include <rte_ether.h>
#include <rte_per_lcore.h>

#include "ini.h"

#include "udpdk_args.h"
#include "udpdk_arp.h"
#include "udpdk_types.h"

extern configuration config;
extern int primary_argc;
extern int secondary_argc;
extern char *primary_argv[MAX_ARGC];
extern char *secondary_argv[MAX_ARGC];
static char *progname;

static int parse_handler(void* configuration, const char* section, const char* name, const char* value) {
#define MATCH(s, n) strcmp(section, s) == 0 && strcmp(name, n) == 0
	if (MATCH("dpdk", "lcores_primary")) {
		strncpy(config.lcores_primary, value, MAX_ARG_LEN-1);
	} else if (MATCH("dpdk", "lcores_secondary")) {
		strncpy(config.lcores_secondary, value, MAX_ARG_LEN-1);
	} else if (MATCH("dpdk", "device")) {
		strncpy(config.device, value, MAX_ARG_LEN-1);
	} else if (MATCH("dpdk", "n_mem_channels")) {
		config.n_mem_channels = atoi(value);
	} else {
		fprintf(stderr, "Do not know how to parse section:%s name:%s\n", section, name);
		return 0;   // unknown section/name
	}
	return 1;
}

static int setup_primary_secondary_args(int argc, char *argv[])
{
	// Build primary args
	primary_argc = 0;
	primary_argv[primary_argc] = malloc(strlen(progname)+1);
	snprintf(primary_argv[primary_argc], MAX_ARG_LEN, "%s", progname);
	if (strlen(config.device) > 0) {
		primary_argc++;
		primary_argv[primary_argc] = malloc(3);
		snprintf(primary_argv[primary_argc], 3, "-a");
		primary_argc++;
		primary_argv[primary_argc] = malloc(strlen(config.device)+1);
		snprintf(primary_argv[primary_argc], MAX_ARG_LEN, "%s", config.device);
	}
	primary_argc++;
	primary_argv[primary_argc] = malloc(3);
	snprintf(primary_argv[primary_argc], 3, "-l");
	primary_argc++;
	primary_argv[primary_argc] = malloc(strlen(config.lcores_primary)+1);
	snprintf(primary_argv[primary_argc], MAX_ARG_LEN, "%s", config.lcores_primary);
	primary_argc++;
	primary_argv[primary_argc] = malloc(3);
	snprintf(primary_argv[primary_argc], 3, "-n");
	primary_argc++;
	primary_argv[primary_argc] = malloc(8);
	snprintf(primary_argv[primary_argc], MAX_ARG_LEN, "%d", config.n_mem_channels);
	primary_argc++;
	primary_argv[primary_argc] = malloc(strlen("--proc-type=primary")+1);
	snprintf(primary_argv[primary_argc], MAX_ARG_LEN, "--proc-type=primary");
	primary_argc++;

	// Build secondary args
	secondary_argc = 0;
	secondary_argv[secondary_argc] = malloc(strlen(progname)+1);
	snprintf(secondary_argv[secondary_argc], MAX_ARG_LEN, "%s", progname);
	if (strlen(config.device) > 0) {
		secondary_argc++;
		secondary_argv[secondary_argc] = malloc(3);
		snprintf(secondary_argv[secondary_argc], 3, "-a");
		secondary_argc++;
		secondary_argv[secondary_argc] = malloc(strlen(config.device)+1);
		snprintf(secondary_argv[secondary_argc], MAX_ARG_LEN, "%s", config.device);
	}
	secondary_argc++;
	secondary_argv[secondary_argc] = malloc(3);
	snprintf(secondary_argv[secondary_argc], 3, "-l");
	secondary_argc++;
	secondary_argv[secondary_argc] = malloc(strlen(config.lcores_secondary)+1);
	snprintf(secondary_argv[secondary_argc], MAX_ARG_LEN, "%s", config.lcores_secondary);
	secondary_argc++;
	secondary_argv[secondary_argc] = malloc(3);
	snprintf(secondary_argv[secondary_argc], 3, "-n");
	secondary_argc++;
	secondary_argv[secondary_argc] = malloc(8);
	snprintf(secondary_argv[secondary_argc], MAX_ARG_LEN, "%d", config.n_mem_channels);
	secondary_argc++;
	secondary_argv[secondary_argc] = malloc(strlen("--proc-type=secondary")+1);
	snprintf(secondary_argv[secondary_argc], MAX_ARG_LEN, "--proc-type=secondary");
	secondary_argc++;

	if (primary_argc + argc >= MAX_ARGC) {
		return -1;
	}

	// Append app arguments to primary after --
	primary_argv[primary_argc] = malloc(3);
	snprintf(primary_argv[primary_argc], 3, "--");
	primary_argc++;
	for (int i = 0; i < argc; i++) {
		primary_argv[primary_argc] = malloc(strlen(argv[i])+1);
		snprintf(primary_argv[primary_argc], MAX_ARG_LEN, "%s", argv[i]);
		primary_argc++;
	}

	printf("Application args: ");
	for (int i = 0; i < primary_argc; i++)
		printf("%s ", primary_argv[i]);
	printf("\n");

	printf("Poller args: ");
	for (int i = 0; i < secondary_argc; i++)
		printf("%s ", secondary_argv[i]);
	printf("\n");

	return 0;
}

//read the file and construct the static arp table
int arp_parse(char* filename) {
	FILE *fp;
	char *line = NULL;
	char *token = NULL;
	size_t len = 0;
	ssize_t read;

	fp = fopen(filename, "r");
	if (fp == NULL) {
		fprintf(stderr, "Cannot continue without my static arp file %s!\n", filename);
		exit(-1);
	}

	while (getline(&line, &len, fp) != -1) {
		struct in_addr ip;
		struct rte_ether_addr mac;

		// this line is a comment
		if (line[0] == '#') {
			continue;
		}

		// remove '\n' at the end if present
		char* l = line;
		while (*l != '\0') {
			if (*l == '\n') {
				*l = '\0';
			} else {
				l++;
			}
		}

		token = strtok(line, " \t");
		if (token == NULL) {
			fprintf(stderr, "Invalid line [%s]. Did you separate the ip and mac by a space/tab?\n", line);
			continue;
		}
		ip.s_addr = inet_addr(token);
		if (ip.s_addr == (in_addr_t)(-1)) {
			fprintf(stderr, "Can't parse IPv4 address: %s\n", token);
			return -1;
		}

		token = strtok(NULL, " \t");
		if (token == NULL) {
			fprintf(stderr, "Invalid line [%s]. Did you separate the ip and mac by a space/tab?\n", line);
			continue;
		}
		// need to trim the token
		if (rte_ether_unformat_addr(token, &mac) < 0) {
			fprintf(stderr, "Can't parse MAC address [%s]: %s\n", token, rte_strerror(rte_errno));
			return -1;
		}

		if (udpdk_arp_add_entry(ip, mac) < 0) {
			fprintf(stderr, "Cannot add entry (%s, %s) to ARP table.\n", line, token);
			return -1;
		} else {
			fprintf(stderr, "Added entry (%s, %s) to ARP table.\n", line, token);
		}
	}

	return 0;
}

int udpdk_parse_args(int argc, char *argv[])
{
	int opt, option_index;
	char *cfg_filename = NULL;
	char *arp_filename = NULL;

	if (argc < 5) {
		fprintf(stderr, "Too few arguments (respect the order): %s -c <config.ini> -a <static_arp.txt>\n", argv[0]);
		return -1;
	}

	progname = argv[0];

	// if using getopt, then the application might not be able to parse the arguments by itself later on
	cfg_filename = argv[2];
	arp_filename = argv[4];

	argc -= 5;
	argv += 5;

	if (ini_parse(cfg_filename, parse_handler, NULL) < 0) {
		fprintf(stderr, "Can not parse configuration file %s\n", cfg_filename);
		return -1;
	}

	if (arp_parse(arp_filename) < 0) {
		fprintf(stderr, "Can not parse static arp file %s\n", arp_filename);
		return -1;
	}

	// Initialize global arrays of arguments for primary and secondary
	if (setup_primary_secondary_args(argc, argv) < 0) {
		fprintf(stderr, "Failed to initialize primary/secondary arguments\n");
		return -1;
	}

	return 0;
}
