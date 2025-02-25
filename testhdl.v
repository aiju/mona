module testhdl(
	input wire clk,
	output wire [7:0] led,
	output wire scl,
	inout wire sda,
	input wire hdmi_int,
	output wire [2:0] debug,
	output wire video_clk,
	output wire [23:0] video_data,
	output wire video_hsync,
	output wire video_vsync,
	output wire video_de
);

	/*AUTOWIRE*/
	// Beginning of automatic wires (for undeclared instantiated-module outputs)
	wire [31:0]	fpga_to_hps_araddr;	// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	fpga_to_hps_arburst;	// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	fpga_to_hps_arcache;	// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	fpga_to_hps_arid;	// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	fpga_to_hps_arlen;	// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	fpga_to_hps_arlock;	// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	fpga_to_hps_arprot;	// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	fpga_to_hps_arsize;	// From mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_arvalid;	// From mkTopLevel_i of mkTopLevel.v
	wire [31:0]	fpga_to_hps_awaddr;	// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	fpga_to_hps_awburst;	// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	fpga_to_hps_awcache;	// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	fpga_to_hps_awid;	// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	fpga_to_hps_awlen;	// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	fpga_to_hps_awlock;	// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	fpga_to_hps_awprot;	// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	fpga_to_hps_awsize;	// From mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_awvalid;	// From mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_bready;	// From mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_rready;	// From mkTopLevel_i of mkTopLevel.v
	wire [31:0]	fpga_to_hps_wdata;	// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	fpga_to_hps_wid;	// From mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_wlast;	// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	fpga_to_hps_wstrb;	// From mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_wvalid;	// From mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_arready;	// From mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_awready;	// From mkTopLevel_i of mkTopLevel.v
	wire [11:0]	hps_to_fpga_lw_bid;	// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	hps_to_fpga_lw_bresp;	// From mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_bvalid;	// From mkTopLevel_i of mkTopLevel.v
	wire [31:0]	hps_to_fpga_lw_rdata;	// From mkTopLevel_i of mkTopLevel.v
	wire [11:0]	hps_to_fpga_lw_rid;	// From mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_rlast;	// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	hps_to_fpga_lw_rresp;	// From mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_rvalid;	// From mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_wready;	// From mkTopLevel_i of mkTopLevel.v
	// End of automatics

	wire		fpga_to_hps_arready;	// To mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_awready;	// To mkTopLevel_i of mkTopLevel.v
	wire [7:0]	fpga_to_hps_bid;	// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	fpga_to_hps_bresp;	// To mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_bvalid;	// To mkTopLevel_i of mkTopLevel.v
	wire [31:0]	fpga_to_hps_rdata;	// To mkTopLevel_i of mkTopLevel.v
	wire [7:0]	fpga_to_hps_rid;	// To mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_rlast;	// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	fpga_to_hps_rresp;	// To mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_rvalid;	// To mkTopLevel_i of mkTopLevel.v
	wire		fpga_to_hps_wready;	// To mkTopLevel_i of mkTopLevel.v

	wire [31:0]	hps_to_fpga_lw_araddr;	// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	hps_to_fpga_lw_arburst;	// To mkTopLevel_i of mkTopLevel.v
	wire [3:0]	hps_to_fpga_lw_arcache;	// To mkTopLevel_i of mkTopLevel.v
	wire [11:0]	hps_to_fpga_lw_arid;	// To mkTopLevel_i of mkTopLevel.v
	wire [3:0]	hps_to_fpga_lw_arlen;	// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	hps_to_fpga_lw_arlock;	// To mkTopLevel_i of mkTopLevel.v
	wire [2:0]	hps_to_fpga_lw_arprot;	// To mkTopLevel_i of mkTopLevel.v
	wire [2:0]	hps_to_fpga_lw_arsize;	// To mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_arvalid;	// To mkTopLevel_i of mkTopLevel.v
	wire [31:0]	hps_to_fpga_lw_awaddr;	// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	hps_to_fpga_lw_awburst;	// To mkTopLevel_i of mkTopLevel.v
	wire [3:0]	hps_to_fpga_lw_awcache;	// To mkTopLevel_i of mkTopLevel.v
	wire [11:0]	hps_to_fpga_lw_awid;	// To mkTopLevel_i of mkTopLevel.v
	wire [3:0]	hps_to_fpga_lw_awlen;	// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	hps_to_fpga_lw_awlock;	// To mkTopLevel_i of mkTopLevel.v
	wire [2:0]	hps_to_fpga_lw_awprot;	// To mkTopLevel_i of mkTopLevel.v
	wire [2:0]	hps_to_fpga_lw_awsize;	// To mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_awvalid;	// To mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_bready;	// To mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_rready;	// To mkTopLevel_i of mkTopLevel.v
	wire [31:0]	hps_to_fpga_lw_wdata;	// To mkTopLevel_i of mkTopLevel.v
	wire [11:0]	hps_to_fpga_lw_wid;	// To mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_wlast;	// To mkTopLevel_i of mkTopLevel.v
	wire [3:0]	hps_to_fpga_lw_wstrb;	// To mkTopLevel_i of mkTopLevel.v
	wire		hps_to_fpga_lw_wvalid;	// To mkTopLevel_i of mkTopLevel.v

	reg rst_n;
	initial rst_n = 1'b0;
	always @(posedge clk) rst_n <= 1'b1;

	wire sda_in, sda_out;

	assign sda_in = sda;
	assign sda = sda_out == 1'b0 ? 1'b0 : 1'bz;

	assign debug = {sda_out, sda, scl};

	mkTopLevel mkTopLevel_i(
		.CLK(clk),
		.RST_N(rst_n),
		.ext_i2c_scl_out_get(scl),
		.ext_i2c_sda_out_get(sda_out),
		.ext_i2c_sda_in_put(sda_in),
		/*AUTOINST*/
				// Outputs
				.led		(led[7:0]),
				.video_clk	(video_clk),
				.video_hsync	(video_hsync),
				.video_vsync	(video_vsync),
				.video_de	(video_de),
				.video_data	(video_data[23:0]),
				.fpga_to_hps_awvalid(fpga_to_hps_awvalid),
				.fpga_to_hps_awid(fpga_to_hps_awid[7:0]),
				.fpga_to_hps_awaddr(fpga_to_hps_awaddr[31:0]),
				.fpga_to_hps_awlen(fpga_to_hps_awlen[3:0]),
				.fpga_to_hps_awsize(fpga_to_hps_awsize[2:0]),
				.fpga_to_hps_awburst(fpga_to_hps_awburst[1:0]),
				.fpga_to_hps_awcache(fpga_to_hps_awcache[3:0]),
				.fpga_to_hps_awprot(fpga_to_hps_awprot[2:0]),
				.fpga_to_hps_awlock(fpga_to_hps_awlock[1:0]),
				.fpga_to_hps_wvalid(fpga_to_hps_wvalid),
				.fpga_to_hps_wid(fpga_to_hps_wid[7:0]),
				.fpga_to_hps_wdata(fpga_to_hps_wdata[31:0]),
				.fpga_to_hps_wstrb(fpga_to_hps_wstrb[3:0]),
				.fpga_to_hps_wlast(fpga_to_hps_wlast),
				.fpga_to_hps_bready(fpga_to_hps_bready),
				.fpga_to_hps_arvalid(fpga_to_hps_arvalid),
				.fpga_to_hps_arid(fpga_to_hps_arid[7:0]),
				.fpga_to_hps_araddr(fpga_to_hps_araddr[31:0]),
				.fpga_to_hps_arlen(fpga_to_hps_arlen[3:0]),
				.fpga_to_hps_arsize(fpga_to_hps_arsize[2:0]),
				.fpga_to_hps_arburst(fpga_to_hps_arburst[1:0]),
				.fpga_to_hps_arcache(fpga_to_hps_arcache[3:0]),
				.fpga_to_hps_arprot(fpga_to_hps_arprot[2:0]),
				.fpga_to_hps_arlock(fpga_to_hps_arlock[1:0]),
				.fpga_to_hps_rready(fpga_to_hps_rready),
				.hps_to_fpga_lw_awready(hps_to_fpga_lw_awready),
				.hps_to_fpga_lw_wready(hps_to_fpga_lw_wready),
				.hps_to_fpga_lw_bvalid(hps_to_fpga_lw_bvalid),
				.hps_to_fpga_lw_bid(hps_to_fpga_lw_bid[11:0]),
				.hps_to_fpga_lw_bresp(hps_to_fpga_lw_bresp[1:0]),
				.hps_to_fpga_lw_arready(hps_to_fpga_lw_arready),
				.hps_to_fpga_lw_rvalid(hps_to_fpga_lw_rvalid),
				.hps_to_fpga_lw_rid(hps_to_fpga_lw_rid[11:0]),
				.hps_to_fpga_lw_rresp(hps_to_fpga_lw_rresp[1:0]),
				.hps_to_fpga_lw_rdata(hps_to_fpga_lw_rdata[31:0]),
				.hps_to_fpga_lw_rlast(hps_to_fpga_lw_rlast),
				// Inputs
				.hdmi_int	(hdmi_int),
				.fpga_to_hps_awready(fpga_to_hps_awready),
				.fpga_to_hps_wready(fpga_to_hps_wready),
				.fpga_to_hps_bvalid(fpga_to_hps_bvalid),
				.fpga_to_hps_bid(fpga_to_hps_bid[7:0]),
				.fpga_to_hps_bresp(fpga_to_hps_bresp[1:0]),
				.fpga_to_hps_arready(fpga_to_hps_arready),
				.fpga_to_hps_rvalid(fpga_to_hps_rvalid),
				.fpga_to_hps_rid(fpga_to_hps_rid[7:0]),
				.fpga_to_hps_rresp(fpga_to_hps_rresp[1:0]),
				.fpga_to_hps_rdata(fpga_to_hps_rdata[31:0]),
				.fpga_to_hps_rlast(fpga_to_hps_rlast),
				.hps_to_fpga_lw_awvalid(hps_to_fpga_lw_awvalid),
				.hps_to_fpga_lw_awid(hps_to_fpga_lw_awid[11:0]),
				.hps_to_fpga_lw_awaddr(hps_to_fpga_lw_awaddr[31:0]),
				.hps_to_fpga_lw_awlen(hps_to_fpga_lw_awlen[3:0]),
				.hps_to_fpga_lw_awsize(hps_to_fpga_lw_awsize[2:0]),
				.hps_to_fpga_lw_awburst(hps_to_fpga_lw_awburst[1:0]),
				.hps_to_fpga_lw_awcache(hps_to_fpga_lw_awcache[3:0]),
				.hps_to_fpga_lw_awprot(hps_to_fpga_lw_awprot[2:0]),
				.hps_to_fpga_lw_awlock(hps_to_fpga_lw_awlock[1:0]),
				.hps_to_fpga_lw_wvalid(hps_to_fpga_lw_wvalid),
				.hps_to_fpga_lw_wid(hps_to_fpga_lw_wid[11:0]),
				.hps_to_fpga_lw_wdata(hps_to_fpga_lw_wdata[31:0]),
				.hps_to_fpga_lw_wstrb(hps_to_fpga_lw_wstrb[3:0]),
				.hps_to_fpga_lw_wlast(hps_to_fpga_lw_wlast),
				.hps_to_fpga_lw_bready(hps_to_fpga_lw_bready),
				.hps_to_fpga_lw_arvalid(hps_to_fpga_lw_arvalid),
				.hps_to_fpga_lw_arid(hps_to_fpga_lw_arid[11:0]),
				.hps_to_fpga_lw_araddr(hps_to_fpga_lw_araddr[31:0]),
				.hps_to_fpga_lw_arlen(hps_to_fpga_lw_arlen[3:0]),
				.hps_to_fpga_lw_arsize(hps_to_fpga_lw_arsize[2:0]),
				.hps_to_fpga_lw_arburst(hps_to_fpga_lw_arburst[1:0]),
				.hps_to_fpga_lw_arcache(hps_to_fpga_lw_arcache[3:0]),
				.hps_to_fpga_lw_arprot(hps_to_fpga_lw_arprot[2:0]),
				.hps_to_fpga_lw_arlock(hps_to_fpga_lw_arlock[1:0]),
				.hps_to_fpga_lw_rready(hps_to_fpga_lw_rready));

	cyclonev_hps_interface_hps2fpga_light_weight hps2fpga_light_weight(
		.clk(clk),
		.awready(hps_to_fpga_lw_awready),
		.wready(hps_to_fpga_lw_wready),
		.bvalid(hps_to_fpga_lw_bvalid),
		.bid(hps_to_fpga_lw_bid[11:0]),
		.bresp(hps_to_fpga_lw_bresp[1:0]),
		.arready(hps_to_fpga_lw_arready),
		.rvalid(hps_to_fpga_lw_rvalid),
		.rid(hps_to_fpga_lw_rid[11:0]),
		.rresp(hps_to_fpga_lw_rresp[1:0]),
		.rdata(hps_to_fpga_lw_rdata[31:0]),
		.rlast(hps_to_fpga_lw_rlast),
		.awvalid(hps_to_fpga_lw_awvalid),
		.awid(hps_to_fpga_lw_awid[11:0]),
		.awaddr(hps_to_fpga_lw_awaddr[31:0]),
		.awlen(hps_to_fpga_lw_awlen[3:0]),
		.awsize(hps_to_fpga_lw_awsize[2:0]),
		.awburst(hps_to_fpga_lw_awburst[1:0]),
		.awcache(hps_to_fpga_lw_awcache[3:0]),
		.awprot(hps_to_fpga_lw_awprot[2:0]),
		.awlock(hps_to_fpga_lw_awlock[1:0]),
		.wvalid(hps_to_fpga_lw_wvalid),
		.wid(hps_to_fpga_lw_wid[11:0]),
		.wdata(hps_to_fpga_lw_wdata[31:0]),
		.wstrb(hps_to_fpga_lw_wstrb[3:0]),
		.wlast(hps_to_fpga_lw_wlast),
		.bready(hps_to_fpga_lw_bready),
		.arvalid(hps_to_fpga_lw_arvalid),
		.arid(hps_to_fpga_lw_arid[11:0]),
		.araddr(hps_to_fpga_lw_araddr[31:0]),
		.arlen(hps_to_fpga_lw_arlen[3:0]),
		.arsize(hps_to_fpga_lw_arsize[2:0]),
		.arburst(hps_to_fpga_lw_arburst[1:0]),
		.arcache(hps_to_fpga_lw_arcache[3:0]),
		.arprot(hps_to_fpga_lw_arprot[2:0]),
		.arlock(hps_to_fpga_lw_arlock[1:0]),
		.rready(hps_to_fpga_lw_rready)
	);

	cyclonev_hps_interface_fpga2hps fpga2hps(
		.clk(clk),
		.port_size_config(2'b00),
		.awvalid(fpga_to_hps_awvalid),
		.awid(fpga_to_hps_awid[7:0]),
		.awaddr(fpga_to_hps_awaddr[31:0]),
		.awlen(fpga_to_hps_awlen[3:0]),
		.awsize(fpga_to_hps_awsize[2:0]),
		.awburst(fpga_to_hps_awburst[1:0]),
		.awcache(fpga_to_hps_awcache[3:0]),
		.awprot(fpga_to_hps_awprot[2:0]),
		.awlock(fpga_to_hps_awlock[1:0]),
		.wvalid(fpga_to_hps_wvalid),
		.wid(fpga_to_hps_wid[7:0]),
		.wdata(fpga_to_hps_wdata[31:0]),
		.wstrb(fpga_to_hps_wstrb[3:0]),
		.wlast(fpga_to_hps_wlast),
		.bready(fpga_to_hps_bready),
		.arvalid(fpga_to_hps_arvalid),
		.arid(fpga_to_hps_arid[7:0]),
		.araddr(fpga_to_hps_araddr[31:0]),
		.arlen(fpga_to_hps_arlen[3:0]),
		.arsize(fpga_to_hps_arsize[2:0]),
		.arburst(fpga_to_hps_arburst[1:0]),
		.arcache(fpga_to_hps_arcache[3:0]),
		.arprot(fpga_to_hps_arprot[2:0]),
		.arlock(fpga_to_hps_arlock[1:0]),
		.rready(fpga_to_hps_rready),
		.awready(fpga_to_hps_awready),
		.wready(fpga_to_hps_wready),
		.bvalid(fpga_to_hps_bvalid),
		.bid(fpga_to_hps_bid[7:0]),
		.bresp(fpga_to_hps_bresp[1:0]),
		.arready(fpga_to_hps_arready),
		.rvalid(fpga_to_hps_rvalid),
		.rid(fpga_to_hps_rid[7:0]),
		.rresp(fpga_to_hps_rresp[1:0]),
		.rdata(fpga_to_hps_rdata[31:0]),
		.rlast(fpga_to_hps_rlast)
	);

endmodule

// Local Variables:
// verilog-library-directories:("." "build")
// End:
