BSCFLAGS=-bdir build -vdir build -simdir build -p .:%/Libraries:lib

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

.PHONY: quartus
quartus:
	quartus_map --read_settings_files=on --write_settings_files=off testhdl -c testhdl
	quartus_fit --read_settings_files=off --write_settings_files=off testhdl -c testhdl
	quartus_asm --read_settings_files=off --write_settings_files=off testhdl -c testhdl
	quartus_eda --read_settings_files=off --write_settings_files=off testhdl -c testhdl
	
