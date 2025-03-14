BSCFLAGS=-bdir build -vdir build -simdir build -p .:%/Libraries:external_lib -aggressive-conditions

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
	cd quartus && \
	quartus_map --read_settings_files=on --write_settings_files=off mona -c mona && \
	quartus_fit --read_settings_files=off --write_settings_files=off mona -c mona && \
	quartus_asm --read_settings_files=off --write_settings_files=off mona -c mona && \
	quartus_eda --read_settings_files=off --write_settings_files=off mona -c mona
	
.PHONY: program
program:
	quartus_pgm -m jtag -o "p;quartus/output_files/mona.sof@2"

