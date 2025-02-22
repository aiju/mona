BSCFLAGS=-bdir build -vdir build -simdir build -p .:%/Libraries:lib

.PHONY: build
build: 
	bsc -verilog -g mkTopLevel -u $(BSCFLAGS) TopLevel.bsv

simbuild.%:
	bsc -sim -g mk$* -u $(BSCFLAGS) $*.bsv
	bsc -sim -e mk$* $(BSCFLAGS) -o build/$*.sim

sim.%: simbuild.%
	build/$*.sim -V out.vcd