# Retained evidence only. Mantle must never execute this generated Makefile.
all:
	clang -target bpf -c demo.ebpf.c -o demo.ebpf.o
	cc demo.c -o demo

run:
	sudo ./demo
