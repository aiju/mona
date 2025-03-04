package Full_tb;

    import CoarseRaster::*;
    import FineRaster::*;
    import PixelOut::*;
    import Connectable::*;
    import StmtFSM::*;
    import Semi_FIFOF::*;
    import Vector::*;
    import CBus::*;
    import DMA::*;
    import PixelSplit::*;
    import Texture::*;
    import UVInterp::*;
    import AXI::*;
    import FIFOF::*;
    import SpecialFIFOs::*;
    `include "Util.defines"

`Parametrize1(mkDMARdChannel32, mkDMARdChannel, DMARdChannel, 32)
`Parametrize1(mkDMAWrChannel_SingleWord32, mkDMAWrChannel_SingleWord, DMAWrChannel, 32)

    typedef 10 MemLatency;

    module mkFull_tb();

        let coarse_raster <- mkCoarseRaster;
        let fine_raster <- mkFineRaster;
        let pixel_split <- mkPixelSplit;
        let uv_interp <- mkUVInterp;
        let texture <- exposeCBusIFC (mkTexture);
        let pixel_out_f <- exposeCBusIFC (mkPixelOut);
        let pixel_out = pixel_out_f.device_ifc;
        DMAWrChannel #(32) dma <- mkDMAWrChannel_SingleWord32;
        DMARdChannel #(32) dma_texture <- mkDMARdChannel32;

        mkConnection(coarse_raster.out, fine_raster.in);
        mkConnection(fine_raster.out, pixel_split.in);
        mkConnection(pixel_split.out, uv_interp.in);
        mkConnection(uv_interp.out, texture.device_ifc.in);
        mkConnection(texture.device_ifc.out, pixel_out.in);
        mkConnection(pixel_out.dma_req, dma.req);
        mkConnection(pixel_out.dma_resp, dma.resp);
        mkConnection(pixel_out.dma_data, dma.data);

        mkConnection(texture.device_ifc.dma_req, dma_texture.req);
        mkConnection(texture.device_ifc.dma_data, dma_texture.data);


//Triangle { edge_vec: [[-10485, -10485, 1048576], [10485, 0, 0], [0, 10485, 0]], uv: [[0, 0], [33554432, 0], [33554432, 0]], min_x: 25, min_y: 25, max_x: 50, max_y: 50 }

        let triangle = tagged Triangle {
                edge_fns: unpack({
                    -27'd10485,
                    -27'd10485,
                    27'd1048576,
                    27'd10485,
                    27'd0,
                    27'd0,
                    27'd0,
                    27'd10485,
                    27'd0
                }),
                uv: unpack({
                    27'd0,
                    27'd0,
                    27'd33554432,
                    27'd0,
                    27'd33554432,
                    27'd0
                }),
                min_x: 25,
                max_x: 50,
                min_y: 25,
                max_y: 50
            };

        mkAutoFSM(seq
            texture.cbus_ifc.write(cfg_texture_en.a, 1);
            coarse_raster.in.enq(triangle);
            delay(10000);
        endseq);

        rule rl_addr;
            let v <- pop_o(dma.axi.wr_addr);
            dma.axi.wr_resp.enq(AXI3_Wr_Resp {bid: v.awid, bresp: 2'b00});
        endrule
        rule rl_data;
            let v <- pop_o(dma.axi.wr_data);
        endrule

        Vector #(MemLatency, FIFOF #(AXI3_Rd_Data #(128, 8))) fifos <- replicateM (mkPipelineFIFOF);

        for(Integer i = 0; i < valueOf(MemLatency) - 1; i = i + 1)
            mkConnection(to_FIFOF_O(fifos[i]), to_FIFOF_I(fifos[i + 1]));

        rule rl_texture_memory;
            let v <- pop_o(dma_texture.axi.rd_addr);
            fifos[0].enq(AXI3_Rd_Data {rid: v.arid, rdata: ?, rlast: True, rresp: 2'b00});
        endrule

        mkConnection(fifos[valueOf(MemLatency) - 1], dma_texture.axi.rd_data);

    endmodule

endpackage