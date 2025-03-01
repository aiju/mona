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
	wire [31:0]	sdram0_araddr;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram0_arburst;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram0_arcache;		// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram0_arid;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram0_arlen;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram0_arlock;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram0_arprot;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram0_arsize;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram0_arvalid;		// From mkTopLevel_i of mkTopLevel.v
	wire [31:0]	sdram0_awaddr;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram0_awburst;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram0_awcache;		// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram0_awid;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram0_awlen;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram0_awlock;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram0_awprot;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram0_awsize;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram0_awvalid;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram0_bready;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram0_rready;		// From mkTopLevel_i of mkTopLevel.v
	wire [127:0]	sdram0_wdata;		// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram0_wid;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram0_wlast;		// From mkTopLevel_i of mkTopLevel.v
	wire [15:0]	sdram0_wstrb;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram0_wvalid;		// From mkTopLevel_i of mkTopLevel.v
	wire [31:0]	sdram1_araddr;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram1_arburst;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram1_arcache;		// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram1_arid;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram1_arlen;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram1_arlock;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram1_arprot;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram1_arsize;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram1_arvalid;		// From mkTopLevel_i of mkTopLevel.v
	wire [31:0]	sdram1_awaddr;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram1_awburst;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram1_awcache;		// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram1_awid;		// From mkTopLevel_i of mkTopLevel.v
	wire [3:0]	sdram1_awlen;		// From mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram1_awlock;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram1_awprot;		// From mkTopLevel_i of mkTopLevel.v
	wire [2:0]	sdram1_awsize;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram1_awvalid;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram1_bready;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram1_rready;		// From mkTopLevel_i of mkTopLevel.v
	wire [127:0]	sdram1_wdata;		// From mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram1_wid;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram1_wlast;		// From mkTopLevel_i of mkTopLevel.v
	wire [15:0]	sdram1_wstrb;		// From mkTopLevel_i of mkTopLevel.v
	wire		sdram1_wvalid;		// From mkTopLevel_i of mkTopLevel.v
	// End of automatics

	wire		sdram0_arready;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram0_awready;		// To mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram0_bid;		// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram0_bresp;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram0_bvalid;		// To mkTopLevel_i of mkTopLevel.v
	wire [127:0]	sdram0_rdata;		// To mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram0_rid;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram0_rlast;		// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram0_rresp;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram0_rvalid;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram0_wready;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram1_arready;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram1_awready;		// To mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram1_bid;		// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram1_bresp;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram1_bvalid;		// To mkTopLevel_i of mkTopLevel.v
	wire [127:0]	sdram1_rdata;		// To mkTopLevel_i of mkTopLevel.v
	wire [7:0]	sdram1_rid;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram1_rlast;		// To mkTopLevel_i of mkTopLevel.v
	wire [1:0]	sdram1_rresp;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram1_rvalid;		// To mkTopLevel_i of mkTopLevel.v
	wire		sdram1_wready;		// To mkTopLevel_i of mkTopLevel.v

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
				.sdram0_awvalid	(sdram0_awvalid),
				.sdram0_awid	(sdram0_awid[7:0]),
				.sdram0_awaddr	(sdram0_awaddr[31:0]),
				.sdram0_awlen	(sdram0_awlen[3:0]),
				.sdram0_awsize	(sdram0_awsize[2:0]),
				.sdram0_awburst	(sdram0_awburst[1:0]),
				.sdram0_awcache	(sdram0_awcache[3:0]),
				.sdram0_awprot	(sdram0_awprot[2:0]),
				.sdram0_awlock	(sdram0_awlock[1:0]),
				.sdram0_wvalid	(sdram0_wvalid),
				.sdram0_wid	(sdram0_wid[7:0]),
				.sdram0_wdata	(sdram0_wdata[127:0]),
				.sdram0_wstrb	(sdram0_wstrb[15:0]),
				.sdram0_wlast	(sdram0_wlast),
				.sdram0_bready	(sdram0_bready),
				.sdram0_arvalid	(sdram0_arvalid),
				.sdram0_arid	(sdram0_arid[7:0]),
				.sdram0_araddr	(sdram0_araddr[31:0]),
				.sdram0_arlen	(sdram0_arlen[3:0]),
				.sdram0_arsize	(sdram0_arsize[2:0]),
				.sdram0_arburst	(sdram0_arburst[1:0]),
				.sdram0_arcache	(sdram0_arcache[3:0]),
				.sdram0_arprot	(sdram0_arprot[2:0]),
				.sdram0_arlock	(sdram0_arlock[1:0]),
				.sdram0_rready	(sdram0_rready),
				.sdram1_awvalid	(sdram1_awvalid),
				.sdram1_awid	(sdram1_awid[7:0]),
				.sdram1_awaddr	(sdram1_awaddr[31:0]),
				.sdram1_awlen	(sdram1_awlen[3:0]),
				.sdram1_awsize	(sdram1_awsize[2:0]),
				.sdram1_awburst	(sdram1_awburst[1:0]),
				.sdram1_awcache	(sdram1_awcache[3:0]),
				.sdram1_awprot	(sdram1_awprot[2:0]),
				.sdram1_awlock	(sdram1_awlock[1:0]),
				.sdram1_wvalid	(sdram1_wvalid),
				.sdram1_wid	(sdram1_wid[7:0]),
				.sdram1_wdata	(sdram1_wdata[127:0]),
				.sdram1_wstrb	(sdram1_wstrb[15:0]),
				.sdram1_wlast	(sdram1_wlast),
				.sdram1_bready	(sdram1_bready),
				.sdram1_arvalid	(sdram1_arvalid),
				.sdram1_arid	(sdram1_arid[7:0]),
				.sdram1_araddr	(sdram1_araddr[31:0]),
				.sdram1_arlen	(sdram1_arlen[3:0]),
				.sdram1_arsize	(sdram1_arsize[2:0]),
				.sdram1_arburst	(sdram1_arburst[1:0]),
				.sdram1_arcache	(sdram1_arcache[3:0]),
				.sdram1_arprot	(sdram1_arprot[2:0]),
				.sdram1_arlock	(sdram1_arlock[1:0]),
				.sdram1_rready	(sdram1_rready),
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
				.sdram0_awready	(sdram0_awready),
				.sdram0_wready	(sdram0_wready),
				.sdram0_bvalid	(sdram0_bvalid),
				.sdram0_bid	(sdram0_bid[7:0]),
				.sdram0_bresp	(sdram0_bresp[1:0]),
				.sdram0_arready	(sdram0_arready),
				.sdram0_rvalid	(sdram0_rvalid),
				.sdram0_rid	(sdram0_rid[7:0]),
				.sdram0_rresp	(sdram0_rresp[1:0]),
				.sdram0_rdata	(sdram0_rdata[127:0]),
				.sdram0_rlast	(sdram0_rlast),
				.sdram1_awready	(sdram1_awready),
				.sdram1_wready	(sdram1_wready),
				.sdram1_bvalid	(sdram1_bvalid),
				.sdram1_bid	(sdram1_bid[7:0]),
				.sdram1_bresp	(sdram1_bresp[1:0]),
				.sdram1_arready	(sdram1_arready),
				.sdram1_rvalid	(sdram1_rvalid),
				.sdram1_rid	(sdram1_rid[7:0]),
				.sdram1_rresp	(sdram1_rresp[1:0]),
				.sdram1_rdata	(sdram1_rdata[127:0]),
				.sdram1_rlast	(sdram1_rlast),
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

	cyclonev_hps_interface_fpga2sdram f2sdram(
		.cfg_port_width(12'b000010101010),
		.cfg_cport_type(12'b000001100110),
		.cfg_rfifo_cport_map(16'b0010001000000000),
		.cfg_wfifo_cport_map(16'b0011001100010001),
		.cfg_cport_rfifo_map(18'b000000010010000000),
		.cfg_cport_wfifo_map(18'b000000010010000000),
		.cfg_axi_mm_select(6'b001111),

		.cmd_port_clk_0(clk),
		.cmd_data_0({
			sdram0_arprot[1:0],
			sdram0_arlock[1:0],
			sdram0_arburst[1:0],
			sdram0_arsize[2:0],
			sdram0_arid[7:0],
			4'b0000,
			sdram0_arlen[3:0],
			sdram0_araddr[31:0],
			3'b001
		}),
		.cmd_valid_0(sdram0_arvalid),
		.cmd_ready_0(sdram0_arready),

		.cmd_port_clk_1(clk),
		.cmd_data_1({
			sdram0_awprot[1:0],
			sdram0_awlock[1:0],
			sdram0_awburst[1:0],
			sdram0_awsize[2:0],
			sdram0_awid[7:0],
			4'b0000,
			sdram0_awlen[3:0],
			sdram0_awaddr[31:0],
			3'b010
		}),
		.cmd_valid_1(sdram0_awvalid),
		.cmd_ready_1(sdram0_awready),

		.cmd_port_clk_2(clk),
		.cmd_data_2({
			sdram1_arprot[1:0],
			sdram1_arlock[1:0],
			sdram1_arburst[1:0],
			sdram1_arsize[2:0],
			sdram1_arid[7:0],
			4'b0000,
			sdram1_arlen[3:0],
			sdram1_araddr[31:0],
			3'b001
		}),
		.cmd_valid_2(sdram1_arvalid),
		.cmd_ready_2(sdram1_arready),

		.cmd_port_clk_3(clk),
		.cmd_data_3({
			sdram1_awprot[1:0],
			sdram1_awlock[1:0],
			sdram1_awburst[1:0],
			sdram1_awsize[2:0],
			sdram1_awid[7:0],
			4'b0000,
			sdram1_awlen[3:0],
			sdram1_awaddr[31:0],
			3'b010
		}),
		.cmd_valid_3(sdram1_awvalid),
		.cmd_ready_3(sdram1_awready),

		.rd_clk_0(clk),
		.rd_clk_1(clk),
		.rd_valid_0(sdram0_rvalid),
		.rd_ready_0(sdram0_rready),
		.rd_ready_1(sdram0_rready),
		.rd_data_0(sdram0_rdata[63:0]),
		.rd_data_1({
			sdram0_rid[7:0],
			sdram0_rlast,
			sdram0_rresp,
			sdram0_rdata[127:64]
		}),

		.rd_clk_2(clk),
		.rd_clk_3(clk),
		.rd_valid_2(sdram1_rvalid),
		.rd_ready_2(sdram1_rready),
		.rd_ready_3(sdram1_rready),
		.rd_data_2(sdram1_rdata[63:0]),
		.rd_data_3({
			sdram1_rid[7:0],
			sdram1_rlast,
			sdram1_rresp,
			sdram1_rdata[127:64]
		}),

		.wr_clk_0(clk),
		.wr_clk_1(clk),
		.wr_valid_0(sdram0_wvalid),
		.wr_valid_1(sdram0_wvalid),
		.wr_ready_0(sdram0_wready),
		.wr_data_0({sdram0_wlast, sdram0_wstrb[7:0], sdram0_wdata[63:0]}),
		.wr_data_1({sdram0_wlast, sdram0_wstrb[15:8], sdram0_wdata[127:64]}),

		.wr_clk_2(clk),
		.wr_clk_3(clk),
		.wr_valid_2(sdram1_wvalid),
		.wr_valid_3(sdram1_wvalid),
		.wr_ready_2(sdram1_wready),
		.wr_data_2({sdram1_wlast, sdram1_wstrb[7:0], sdram1_wdata[63:0]}),
		.wr_data_3({sdram1_wlast, sdram1_wstrb[15:8], sdram1_wdata[127:64]}),

		.wrack_data_1({sdram0_bid[7:0], sdram0_bresp[1:0]}),
		.wrack_ready_1(sdram0_bready),
		.wrack_valid_1(sdram0_bvalid),

		.wrack_data_3({sdram1_bid[7:0], sdram1_bresp[1:0]}),
		.wrack_ready_3(sdram1_bready),
		.wrack_valid_3(sdram1_bvalid),
	);

endmodule

// Local Variables:
// verilog-library-directories:("." "build")
// End:
