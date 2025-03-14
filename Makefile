
.PHONY: hw
hw:
	make -C hw

.PHONY: quartus
quartus:
	make -C hw quartus

.PHONY: program
program:
	make -C hw program

.PHONY: driver
driver:
	@(cd driver; cargo build --target armv7-unknown-linux-gnueabihf --release)
