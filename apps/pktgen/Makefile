#
# Created by leoll2 on 10/07/20.
# Copyright (c) 2020 Leonardo Lai. All rights reserved.
#

ROOTDIR=../..
DEPSDIR=${ROOTDIR}/deps

ifeq ($(RTE_TARGET),)
$(error "Please define RTE_TARGET environment variable")
endif

ifeq ($(UDPDK_PATH),)
	UDPDK_PATH=${ROOTDIR}
endif

# all source are stored in SRCS-y
SRCS= main.c

UDPDK_DPDK=${DEPSDIR}/dpdk/install/lib/${RTE_TARGET}/

LIBS+= -L${UDPDK_PATH}/udpdk -Wl,--whole-archive,-ludpdk,--no-whole-archive
LIBS+= $(shell PKG_CONFIG_PATH=${UDPDK_DPDK}/pkgconfig pkg-config --libs libdpdk)
LIBS+= -Wl,--no-whole-archive -lrt -lm -ldl -lcrypto -pthread -lnuma

CFLAGS += $(WERROR_FLAGS) -g -O2 #3
UDPDK_CFLAGS = $(shell PKG_CONFIG_PATH=${UDPDK_DPDK}/pkgconfig pkg-config --cflags libdpdk)

TARGET="pktgen"
all:
	cc -I${ROOTDIR}/udpdk -I${ROOTDIR}/udpdk/list ${UDPDK_CFLAGS} -o ${TARGET} ${SRCS} ${LIBS}
	cc ${CFLAGS} -o linux_pktgen linux_pktgen.c -lpthread

.PHONY: clean
clean:
	rm -f *.o ${TARGET} linux_pktgen
