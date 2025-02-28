package TopLevel;

    import I2C :: *;
    import Video :: *;
    import Connectable::*;
    import StmtFSM::*;
    import AXI :: *;
    import DMA :: *;
    import HdmiCtrl :: *;
    import CBus :: *;
    import ConfigDefs :: *;
    import AxiToSimple :: *;
    import Semi_FIFOF::*;
    import CoarseRaster::*;
    import FineRaster::*;
    import PixelOut::*;
    import Vector::*;

interface TopLevel;
    (* always_ready *) method Bit #(8) led;
    (* always_ready, always_enabled, prefix="" *) method Action hdmi_int(Bool hdmi_int);
    interface ExtI2C ext_i2c;
    (*prefix="video"*) interface Ext_Video ext_video;
    interface AXI3_Master_IFC #(32, 32, 8) fpga_to_hps;
endinterface

interface TopLevelWithAxiSlave;
    (*prefix=""*) interface TopLevel top_level;
    interface AXI3_Slave_IFC #(32, 32, 12) hps_to_fpga_lw;
endinterface

module [ModWithConfig] mkInternals(TopLevel);
    HdmiCtrl hdmi_ctrl <- mkHdmiCtrl;
    Video video <- mkVideo;
    DMA #(2, 1) dma <- mkDMA;
    CoarseRaster coarse_raster <- mkCoarseRaster;
    FineRaster fine_raster <- mkFineRaster;
    PixelOut pixel_out <- mkPixelOut;

    mkConnection(video.dma_req, dma.rd_req[1]);
    mkConnection(dma.rd_data[1], video.dma_resp);

    mkConnection(coarse_raster.out, fine_raster.in);
    mkConnection(fine_raster.out, pixel_out.in);

    mkConnection(pixel_out.dma_req, dma.wr_req[0]);
    mkConnection(pixel_out.dma_data, dma.wr_data[0]);
    mkConnection(pixel_out.dma_resp, dma.wr_resp[0]);

    Reg #(Bool) dma_start <- mkCBRegRW(CRAddr { a: 12'h4, o : 0}, False);

    Reg #(CoarseRasterIn) data <- mkRegU;

    let fsm <- mkFSM (seq
        dma.rd_req[0].enq(DMA_Req { addr: 32'h1020_0000, len: 68 });
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[0].x <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[0].y <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[0].a <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[1].x <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[1].y <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[1].a <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[2].x <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[2].y <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.edge_fns[2].a <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.uv[0][0] <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.uv[0][1] <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.uv[1][0] <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.uv[1][1] <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.uv[2][0] <= unpack(truncate(x)); endaction
        action let x <- pop_o(dma.rd_data[0]); data.uv[2][1] <= unpack(truncate(x)); endaction
        action 
            let x <- pop_o(dma.rd_data[0]);
            let d = data;
            d.min_y = unpack(x[24:16]);
            d.min_x = unpack(x[8:0]);
            data <= d;
        endaction
        action 
            let x <- pop_o(dma.rd_data[0]);
            let d = data;
            d.max_y = unpack(x[24:16]);
            d.max_x = unpack(x[8:0]);
            data <= d;
        endaction
        coarse_raster.in.enq(data);
    endseq);

    rule rl_start;
        if(dma_start) begin
            fsm.start;
            dma_start <= False;
        end
    endrule

    interface ExtI2C ext_i2c = hdmi_ctrl.ext_i2c;
    interface Video ext_video = video.ext;
    interface AXI3_Master_IFC fpga_to_hps = dma.mem_ifc;
    method hdmi_int = hdmi_ctrl.hdmi_int;

    method Bit #(8) led;
        return {1'b1, 6'b0, hdmi_ctrl.hdmi_active ? 1'b1 : 1'b0};
    endmethod

endmodule

(* synthesize *)
module mkTopLevel(TopLevelWithAxiSlave);
    AxiToSimple reg_if <- mkAxiToSimple;
    IWithCBus #(ConfigBus, TopLevel) top <- exposeCBusIFC(mkInternals);
    mkConnection(reg_if.simple, top.cbus_ifc);

    interface top_level = top.device_ifc;
    interface hps_to_fpga_lw = reg_if.axi;

endmodule

endpackage