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
    import PixelSplit :: *;
    import UVInterp :: *;
    import Clear :: *;
    import Texture :: *;
    import TextOverlay :: *;
    import Stats :: *;
    `include "Util.defines"

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

`Parametrize2(mkFabric0, mkFabric, Fabric, 3, 3)
`Parametrize2(mkFabric1, mkFabric, Fabric, 1, 0)
`Parametrize1(mkDMARdChannel32, mkDMARdChannel, DMARdChannel, 32)
`Parametrize1(mkDMARdChannel128, mkDMARdChannel, DMARdChannel, 128)
`Parametrize1(mkDMAWrChannel32, mkDMAWrChannel, DMAWrChannel, 32)
`Parametrize1(mkDMAWrChannel128, mkDMAWrChannel, DMAWrChannel, 128)
`Parametrize1(mkDMAWrChannel_SingleWord32, mkDMAWrChannel_SingleWord, DMAWrChannel, 32)


module [ModWithConfig] mkInternals(TopLevel);
    let fabric0 <- mkFabric0;
    let fabric1 <- mkFabric1;

    DMARdChannel #(32) dma_video <- mkDMARdChannel32;
    DMARdChannel #(32) dma_starter <- mkDMARdChannel32;
    DMARdChannel #(128) dma_depth_rd <- mkDMARdChannel128;
    DMAWrChannel #(128) dma_depth_wr <- mkDMAWrChannel128;
    DMARdChannel #(32) dma_texture <- mkDMARdChannel32;
    DMAWrChannel #(32) dma_pixel_out <- mkDMAWrChannel_SingleWord32;
    DMAWrChannel #(128) dma_clear <- mkDMAWrChannel128;

    mkConnection(dma_starter.axi, fabric0.rd[0]);
    mkConnection(dma_depth_rd.axi, fabric0.rd[1]);
    mkConnection(dma_texture.axi, fabric0.rd[2]);

    mkConnection(dma_depth_wr.axi, fabric0.wr[0]);
    mkConnection(dma_pixel_out.axi, fabric0.wr[1]);
    mkConnection(dma_clear.axi, fabric0.wr[2]);

    mkConnection(dma_video.axi, fabric1.rd[0]);

    HdmiCtrl hdmi_ctrl <- mkHdmiCtrl;
    Video video <- mkVideo;
    Clear clear <- mkClear;

    Starter starter <- mkStarter;
    CoarseRaster coarse_raster <- mkCoarseRaster;
    FineRaster fine_raster <- mkFineRaster;
    DepthTest depth_test <- mkDepthTest;
    PixelSplit pixel_split <- mkPixelSplit;
    UVInterp uv_interp <- mkUVInterp;
    Texture texture <- mkTexture;
    PixelOut pixel_out <- mkPixelOut;

    mkConnection(video.dma_req, dma_video.req);
    mkConnection(video.dma_resp, dma_video.data);

    mkConnection(clear.dma_req, dma_clear.req);
    mkConnection(clear.dma_data, dma_clear.data);
    mkConnection(clear.dma_resp, dma_clear.resp);

    mkConnection(starter.dma_req, dma_starter.req);
    mkConnection(starter.dma_resp, dma_starter.data);

    mkConnection(texture.dma_req, dma_texture.req);
    mkConnection(texture.dma_data, dma_texture.data);

    mkConnectionStats(cfg_stats_starter, starter.out, coarse_raster.in);
    mkConnectionStats(cfg_stats_coarse, coarse_raster.out, fine_raster.in);
    mkConnectionStats(cfg_stats_fine, fine_raster.out, depth_test.in);
    mkConnectionStats(cfg_stats_depth, depth_test.out, pixel_split.in);
    mkConnectionStats(cfg_stats_pixel, pixel_split.out, uv_interp.in);
    mkConnectionStats(cfg_stats_uv, uv_interp.out, texture.in);
    mkConnectionStats(cfg_stats_texture, texture.out, pixel_out.in);

    mkConnection(depth_test.rd_req, dma_depth_rd.req);
    mkConnection(depth_test.rd_data, dma_depth_rd.data);
    mkConnection(depth_test.wr_req, dma_depth_wr.req);
    mkConnection(depth_test.wr_data, dma_depth_wr.data);
    mkConnection(depth_test.wr_resp, dma_depth_wr.resp);

    mkConnection(pixel_out.dma_req, dma_pixel_out.req);
    mkConnection(pixel_out.dma_data, dma_pixel_out.data);
    mkConnection(pixel_out.dma_resp, dma_pixel_out.resp);

    interface ExtI2C ext_i2c = hdmi_ctrl.ext_i2c;
    interface Video ext_video = video.ext;
    interface AXI3_Master_IFC sdram0 = fabric0.mem_ifc;
    interface AXI3_Master_IFC sdram1 = fabric1.mem_ifc;
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
    Reg #(Vector #(3, Vector #(3, Bit #(8)))) rgb <- mkRegU;

    let fsm <- mkFSM (seq
        f_dma_req.enq(DMA_Req { addr: 32'h1020_0000, len: 80 * extend(ctr) });
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
            action let x <- pop(f_dma_resp); rgb[0] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); rgb[1] <= unpack(truncate(x)); endaction
            action let x <- pop(f_dma_resp); rgb[2] <= unpack(truncate(x)); endaction
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
                rgb: rgb,
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