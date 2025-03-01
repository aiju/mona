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
    import FIFOF::*;
    import Util :: *;
    import DepthTest :: *;

interface TopLevel;
    (* always_ready *) method Bit #(8) led;
    (* always_ready, always_enabled, prefix="" *) method Action hdmi_int(Bool hdmi_int);
    interface ExtI2C ext_i2c;
    (*prefix="video"*) interface Ext_Video ext_video;
    interface AXI3_Master_IFC #(32, 128, 8) sdram0;
    interface AXI3_Master_IFC #(32, 128, 8) sdram1;
endinterface

interface TopLevelWithAxiSlave;
    (*prefix=""*) interface TopLevel top_level;
    interface AXI3_Slave_IFC #(32, 32, 12) hps_to_fpga_lw;
endinterface

(* synthesize *)
module mkDMA0(DMA #(2, 2));
    let m <- mkDMA;
    return m;
endmodule

(* synthesize *)
module mkDMA1(DMA #(1, 0));
    let m <- mkDMA;
    return m;
endmodule

module [ModWithConfig] mkInternals(TopLevel);
    HdmiCtrl hdmi_ctrl <- mkHdmiCtrl;
    Video video <- mkVideo;
    let dma0 <- mkDMA0;
    let dma1 <- mkDMA1;
    Starter starter <- mkStarter;
    CoarseRaster coarse_raster <- mkCoarseRaster;
    FineRaster fine_raster <- mkFineRaster;
    PixelOut pixel_out <- mkPixelOut;
    DepthTest depth_test <- mkDepthTest;

    mkConnection(video.dma_req, dma1.rd_req[0]);
    mkConnection(dma1.rd_data[0], video.dma_resp);

    mkConnection(starter.dma_req, dma0.rd_req[0]);
    mkConnection(starter.dma_resp, dma0.rd_data[0]);
    mkConnection(starter.out, coarse_raster.in);

    mkConnection(coarse_raster.out, fine_raster.in);
    mkConnection(fine_raster.out, depth_test.in);
    mkConnection(depth_test.out, pixel_out.in);

    mkConnection(depth_test.rd_req, dma0.rd_req[1]);
    mkConnection(depth_test.wr_req, dma0.wr_req[1]);
    mkConnection(depth_test.wr_resp, dma0.wr_resp[1]);

    Reg #(Bit #(128)) rdbuf <- mkRegU;
    mkAutoFSM(seq
        while(True) seq
            action let v <- pop_o(dma0.rd_data[1]); rdbuf <= rdbuf << 32 | zeroExtend(v); endaction
            action let v <- pop_o(dma0.rd_data[1]); rdbuf <= rdbuf << 32 | zeroExtend(v); endaction
            action let v <- pop_o(dma0.rd_data[1]); rdbuf <= rdbuf << 32 | zeroExtend(v); endaction
            action let v <- pop_o(dma0.rd_data[1]); rdbuf <= rdbuf << 32 | zeroExtend(v); endaction
            depth_test.rd_data.enq(rdbuf);
        endseq
    endseq);
    Reg #(Bit #(128)) wrbuf <- mkRegU;
    mkAutoFSM(seq
        while(True) seq
            action let v <- pop_o(depth_test.wr_data); wrbuf <= v; endaction
            action dma0.wr_data[1].enq(truncate(wrbuf)); wrbuf <= wrbuf >> 32; endaction
            action dma0.wr_data[1].enq(truncate(wrbuf)); wrbuf <= wrbuf >> 32; endaction
            action dma0.wr_data[1].enq(truncate(wrbuf)); wrbuf <= wrbuf >> 32; endaction
            action dma0.wr_data[1].enq(truncate(wrbuf)); wrbuf <= wrbuf >> 32; endaction
        endseq
    endseq);


    mkConnection(pixel_out.dma_req, dma0.wr_req[0]);
    mkConnection(pixel_out.dma_data, dma0.wr_data[0]);
    mkConnection(pixel_out.dma_resp, dma0.wr_resp[0]);

    interface ExtI2C ext_i2c = hdmi_ctrl.ext_i2c;
    interface Video ext_video = video.ext;
    //interface AXI3_Master_IFC fpga_to_hps = dma.mem_ifc;
    interface AXI3_Master_IFC sdram0 = dma0.mem_ifc;
    interface AXI3_Master_IFC sdram1 = dma1.mem_ifc;
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

interface Starter;
    interface FIFOF_O #(DMA_Req) dma_req;
    interface FIFOF_I #(Bit #(32)) dma_resp;
    interface FIFOF_O #(CoarseRasterIn) out;
endinterface

module [ModWithConfig] mkStarter(Starter);
    FIFOF #(DMA_Req) f_dma_req <- mkFIFOF;
    FIFOF #(Bit #(32)) f_dma_resp <- mkFIFOF;
    FIFOF #(CoarseRasterIn) f_out <- mkFIFOF;

    Reg #(Bool) dma_start <- mkCBRegRW(cfg_control_start, False);
    Reg #(Bool) issue_flush <- mkCBRegRW(cfg_control_flush, False);
    Reg #(Bit #(16)) dma_len <- mkCBRegRW(cfg_control_len, 0);

    Reg #(Bit #(32)) addr <- mkRegU;
    Reg #(Bit #(16)) ctr  <- mkRegU;

    Reg #(Vector #(3, EdgeFn)) edge_fns <- mkRegU;
    Reg #(Vector #(3, Vector #(2, Int #(27)))) uv <- mkRegU;
    Reg #(UInt #(9)) min_x <- mkRegU;
    Reg #(UInt #(9)) min_y <- mkRegU;
    Reg #(UInt #(9)) max_x <- mkRegU;
    Reg #(UInt #(9)) max_y <- mkRegU;

    let fsm <- mkFSM (seq
        f_dma_req.enq(DMA_Req { addr: 32'h1020_0000, len: 68 * extend(ctr) });
        while(ctr > 0) seq
            action let x <- pop(f_dma_resp); edge_fns[0].x <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[0].y <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[0].a <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[1].x <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[1].y <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[1].a <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[2].x <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[2].y <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); edge_fns[2].a <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); uv[0][0] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); uv[0][1] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); uv[1][0] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); uv[1][1] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); uv[2][0] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); uv[2][1] <= unpack(truncate(x)); endaction
            action 
                let x <- pop(f_dma_resp);
                min_y <= unpack(x[24:16]);
                min_x <= unpack(x[8:0]);
            endaction
            action 
                let x <- pop(f_dma_resp);
                max_y <= unpack(x[24:16]);
                max_x <= unpack(x[8:0]);
            endaction
            f_out.enq(tagged Triangle { 
                edge_fns: edge_fns,
                uv: uv,
                min_x: min_x,
                min_y: min_y,
                max_x: max_x,
                max_y: max_y
            });
            ctr <= ctr - 1;
        endseq
    endseq);

    rule rl_start;
        if(dma_start) begin
            if(dma_len > 0) begin
                fsm.start;
                ctr <= dma_len;
            end
            dma_start <= False;
        end
    endrule

    rule rl_flush;
        if(issue_flush) begin
            fsm.waitTillDone;
            f_out.enq(tagged Flush);
            issue_flush <= False;
        end
    endrule

    interface dma_req = to_FIFOF_O(f_dma_req);
    interface dma_resp = to_FIFOF_I(f_dma_resp);
    interface out = to_FIFOF_O(f_out);
endmodule

endpackage