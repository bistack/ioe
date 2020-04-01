PREFIX := /opt/compiler/gcc-4.8.2
LINKER := $(PREFIX)/bin/gcc
LINK_ARGS := '-Wl,-rpath,$(PREFIX)/lib -Wl,--dynamic-linker=$(PREFIX)/lib/ld-linux-x86-64.so.2'
OUT_DIR := ./target/debug

all:
	@gcc -v
	cargo rustc -vv -p bidx --lib 	    	-- -v -C relocation-model=dynamic-no-pic -C linker=$(LINKER) -C link-args=$(LINK_ARGS)
	cp target/debug/libbidx.a lib/
	cargo rustc -vv -p bioe --bin biobench	-- -v -C relocation-model=dynamic-no-pic -C linker=$(LINKER) -C link-args=$(LINK_ARGS) 
	- mkdir sim
	cp $(OUT_DIR)/biobench ~/project
	cd sim; #RUST_BACKTRACE=1 rust-gdb ../$(OUT_DIR)/biobench

cbind:
	cargo build -p bidx --bin cbind
	cp $(OUT_DIR)/cbind ./

clean:
	- rm lib/libbidx.a
	- rm $(OUT_DIR)/biobench
	- rm $(OUT_DIR)/deps/libbidx.*
	- rm $(OUT_DIR)/deps/bidx.d
	- rm $(OUT_DIR)/deps/bidx.d
	- rm $(OUT_DIR)/deps/bioe-*
	- rm $(OUT_DIR)/build/bioe-*
	- rm $(OUT_DIR)/incremental/bioe-*
	- rm $(OUT_DIR)/libbidx.*
	- rm $(OUT_DIR)/libbidx.d
	- rm -r $(OUT_DIR)/.fingerprint/bidx-*
	- rm -r $(OUT_DIR)/incremental/bidx-*
	- rm -r sim

cleanall: clean
	cargo clean


