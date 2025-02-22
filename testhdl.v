module testhdl(
	input wire clk,
	output wire [7:0] led,
	output wire scl,
	inout wire sda,
	output wire [2:0] debug,
	output wire video_clk,
	output wire [23:0] video_data,
	output wire video_hsync,
	output wire video_vsync,
	output wire video_de
);

	reg rst_n;
	initial rst_n = 1'b0;
	always @(posedge clk) rst_n <= 1'b1;

	wire sda_in, sda_out;

	assign sda_in = sda;
	assign sda = sda_out == 1'b0 ? 1'b0 : 1'bz;

	assign debug = {sda_out, sda, scl};

	wire [31:0] fifo_data;
	wire fifo_en;
	wire fifo_rdy;
	wire consume_pixel;

	mkTopLevel mkTopLevel_i(
		.CLK(clk),
		.RST_N(rst_n),
		.led(),
		.ext_i2c_scl_out_get(scl),
		.ext_i2c_sda_out_get(sda_out),
		.ext_i2c_sda_in_put(sda_in),
		.ext_video_clk(video_clk),
		.ext_video_data(video_data),
		.ext_video_hsync(video_hsync),
		.ext_video_vsync(video_vsync),
		.ext_video_de(video_de),
		.ext_video_in_data_put(fifo_data),
		.EN_ext_video_in_data_put(fifo_en),
		.RDY_ext_video_in_data_put(fifo_rdy),
		.ext_video_consume_pixel(consume_pixel)
	);

	reg [31:0] ctr;
	always @(posedge clk)
		ctr <= ctr + 1;

	reg got_led = 1'b0;
	wire arvalid, rready;
	reg rvalid;
	assign led = {ctr[25], 6'b0, got_led};
	wire [11:0] arid;
	reg [11:0] rid;

	(* KEEP *)
	cyclonev_hps_interface_hps2fpga_light_weight hps2fpga_light_weight(
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
	);

	always @(posedge clk) begin
		if(arvalid && !rvalid) begin
			rvalid <= 1'b1;
			rid <= arid;
		end
		if(rvalid && rready) begin
			rvalid <= 1'b0;
		end
	end

	reg [31:0] m_araddr;
	wire m_rvalid, m_rready;
	wire [31:0] m_rdata;
	reg m_arvalid = 1'b0;
	wire m_arready;

	cyclonev_hps_interface_fpga2hps fpga2hps(
		.clk(clk),
		.port_size_config(2'b01),
		.arid(8'h00),
		.araddr(m_araddr),
		.arlen(4'b1111),
		.arsize(3'b010),
		.arburst(2'b01),
		.arcache(4'b0011),
		.arprot(3'b000),
		.arvalid(m_arvalid),
		.arready(m_arready),
		.rvalid(m_rvalid),
		.rready(m_rready),
		.rdata(m_rdata),
	);

	localparam [31:0] START = 32'h1000_0000;
	localparam [31:0] END = 32'h1000_0000 + 640 * 480 * 4;

	initial m_araddr = START;

	reg [6:0] credits = 7'b0;

	wire issue = !m_arvalid && credits <= 48;

	wire [6:0] c0 = issue ? credits + 16 : credits;
	wire [6:0] c1 = consume_pixel && c0 > 0 ? c0 - 1 : c0;

	always @(posedge clk) begin
		if(issue) begin
			m_arvalid <= 1'b1;
		end
		credits <= c1;
		if(m_arvalid && m_arready) begin
			m_arvalid <= 1'b0;
			if(m_araddr + 16 * 4 == END)
				m_araddr <= START;
			else
				m_araddr <= m_araddr + 16 * 4;
		end
	end

	assign fifo_data = m_rdata;
	assign fifo_en = m_rvalid;
	assign m_rready = fifo_rdy;


endmodule