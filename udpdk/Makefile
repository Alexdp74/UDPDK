#
# Created by leoll2 on 9/25/20.
# Copyright (c) 2020 Leonardo Lai. All rights reserved.
#

ROOTDIR?=${CURDIR}/..
PREFIX=/usr/local
PREFIX_LIB=${PREFIX}/lib
PREFIX_INCLUDE=/usr/local/include
PREFIX_BIN=/usr/local/bin
UDPDK_VERSION=1.0
PKGCONF = pkg-config

CFLAGS= -march=native -O2
CFLAGS+= -Wall -Wno-deprecated-declarations -Werror -Wno-unused-variable
CFLAGS+= -fno-common -finline-limit=8000
CFLAGS+= --param inline-unit-growth=100
CFLAGS+= --param large-function-growth=1000
CFLAGS += $(shell $(PKGCONF) --cflags inih) $(shell $(PKGCONF) --cflags libdpdk)

UDPDK_CFLAGS+= -I. -Ilist/ -Ishmalloc/

UDPDK_CORE_SRCS+=    \
	udpdk_args.c     \
	udpdk_arp.c     \
	udpdk_dump.c     \
	udpdk_globals.c  \
	udpdk_init.c     \
	udpdk_bind_table.c \
	udpdk_monitor.c  \
	udpdk_poller.c   \
	udpdk_syscall.c  \
	udpdk_sync.c

UDPDK_LIST_SRCS+=    \
	list/udpdk_list.c      \
	list/udpdk_list_node.c \
	list/udpdk_list_iterator.c \
	list/udpdk_list_globals.c \
	list/udpdk_list_init.c

UDPDK_SHM_SRCS+=    \
	shmalloc/udpdk_shmalloc.c

SRCS:= ${UDPDK_CORE_SRCS} ${UDPDK_LIST_SRCS} ${UDPDK_SHM_SRCS}
OBJS+= $(patsubst %.c,%.o,${SRCS})

UDPDK_CC= ${CC} -c $(DPDK_CFLAGS) $(UDPDK_CFLAGS) ${CFLAGS} ${WERROR} $<
LIBS:= $(shell $(PKGCONF) --libs inih)

all: libudpdk.a

# Export only symbols defined in the UDPDK API (also exclude DPDK symbols)
libudpdk.a: ${OBJS}
	${LD} -d -r -o $*.ro ${OBJS} ${LIBS}
	nm $*.ro  | grep -v ' U ' | cut -d ' ' -f 3 > $*_localize_list.tmp
	objcopy --localize-symbols=$*_localize_list.tmp $*.ro
	rm $*_localize_list.tmp
	objcopy --globalize-symbols=udpdk_api.symlist $*.ro
	rm -f $@
	ar -cqs $@ $*.ro
	rm -f $*.ro

${OBJS}: %.o: %.c
	${UDPDK_CC} -o $@

.PHONY: clean
clean:
	rm -f libudpdk.a
	rm -f ${OBJS} ${PROGRAM}

.PHONY: install
install:
	rm -f ${PREFIX_LIB}/libudpdk.a.${UDPDK_VERSION}
	rm -f ${PREFIX_LIB}/libudpdk.a
	rm -f ${PREFIX_INCLUDE}/udpdk_*.h
	rm -f ${PREFIX_INCLUDE}/list/udpdk_*.h
	rm -f ${PREFIX_INCLUDE}/shmalloc/udpdk_*.h

	cp -f libudpdk.a ${PREFIX_LIB}/libudpdk.a.${UDPDK_VERSION}
	ln -sf ${PREFIX_LIB}/libudpdk.a.${UDPDK_VERSION} ${PREFIX_LIB}/libudpdk.a
	cp -f udpdk_*.h ${PREFIX_INCLUDE}/
	cp -f list/udpdk_*.h ${PREFIX_INCLUDE}/
	cp -f shmalloc/udpdk_*.h ${PREFIX_INCLUDE}/

.PHONY: uninstall
uninstall:
	rm -f ${PREFIX_LIB}/libudpdk.a.${UDPDK_VERSION}
	rm -f ${PREFIX_LIB}/libudpdk.a
	rm -f ${PREFIX_INCLUDE}/udpdk_*.h

