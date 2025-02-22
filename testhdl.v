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
				.fpga_to_hps_rlast(fpga_to_hps_rlast));

	/*cyclonev_hps_interface_hps2fpga_light_weight hps2fpga_light_weight(
		.clk(clk),
		.arvalid(arvalid),
		.arready(!rvalid),
		.arid(arid),
		.rvalid(rvalid),
		.rready(rready),
		.rlast(1'b1),
		.rdata(32'hDEADBEEF),
		.rid(rid),
		.rresp(2'b00)
	);*/

	cyclonev_hps_interface_fpga2hps fpga2hps(
		.clk(clk),
		.port_size_config(2'b01),
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
