BSCFLAGS=-bdir build -vdir build -simdir build -p .:%/Libraries:lib -aggressive-conditions

.PHONY: build
build: 
	bsc -verilog -g mkTopLevel -u $(BSCFLAGS) TopLevel.bsv

verilog.%:
	bsc -verilog -g mk$* -u $(BSCFLAGS) $*.bsv

simbuild.%:
	bsc -sim -g mk$* -u $(BSCFLAGS) $*.bsv
	bsc -sim -e mk$* $(BSCFLAGS) -o build/$*.sim

sim.%: simbuild.%
	build/$*.sim -V out.vcd

simv.%: verilog.%
	bsc -verilog -e mk$* $(BSCFLAGS) -o build/$*.sim
	build/$*.sim +bscvcd

.PHONY: quartus
quartus:
	quartus_map --read_settings_files=on --write_settings_files=off testhdl -c testhdl
	quartus_fit --read_settings_files=off --write_settings_files=off testhdl -c testhdl
	quartus_asm --read_settings_files=off --write_settings_files=off testhdl -c testhdl
	quartus_eda --read_settings_files=off --write_settings_files=off testhdl -c testhdl
	
.PHONY: program
program:
	quartus_pgm -m jtag -o "p;output_files/testhdl.sof@2"

.PHONY: driver
driver:
	@(cd driver; cargo build --target armv7-unknown-linux-gnueabihf --release)
