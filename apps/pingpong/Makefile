#
# Created by leoll2 on 9/25/20.
# Copyright (c) 2020 Leonardo Lai. All rights reserved.
#

ROOTDIR=../..
DEPSDIR=${ROOTDIR}/deps
PKGCONF = pkg-config

ifeq ($(UDPDK_PATH),)
	UDPDK_PATH=${ROOTDIR}
endif

SRCS_DPDK:= main.c
SRCS_SOCKETS:= linux_pingpong.c

LIBS_DPDK:= -L${UDPDK_PATH}/udpdk -Wl,--whole-archive,-ludpdk,--no-whole-archive
LIBS_DPDK+= $(shell $(PKGCONF) --libs libdpdk)

LDFLAGS_DPDK+= -Wl,--no-whole-archive -lrt -lm -ldl -lcrypto -pthread -lnuma
LDFLAGS_SOCKETS+=

WERROR_FLAGS:= -Wall
CFLAGS += $(WERROR_FLAGS)
CFLAGS_DPDK:= -O2 $(shell $(PKGCONF) --cflags libdpdk)
CFLAGS_SOCKETS:= -O2

TARGET_DPDK:=pingpong
TARGET_SOCKETS:=pingpong-sockets

all: ${TARGET_DPDK} ${TARGET_SOCKETS}

${TARGET_DPDK}:
	cc ${CFLAGS} ${CFLAGS_DPDK} -I${ROOTDIR}/udpdk -o ${TARGET_DPDK} \
	   ${SRCS_DPDK} ${LIBS_DPDK} ${LDFLAGS}

${TARGET_SOCKETS}:
	cc ${CFLAGS} ${CFLAGS_SOCKETS} -I${ROOTDIR}/udpdk -o ${TARGET_SOCKETS} \
	   ${SRCS_SOCKETS} ${LIBS_SOCKETS} ${LDFLAGS_SOCKETS}

.PHONY: clean
clean:
	rm -f *.o ${TARGET_DPDK} ${TARGET_SOCKETS}
