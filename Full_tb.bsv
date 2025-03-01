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

    module mkFull_tb();

        let coarse_raster <- mkCoarseRaster;
        let fine_raster <- mkFineRaster;
        let pixel_out_f <- exposeCBusIFC (mkPixelOut);
        let pixel_out = pixel_out_f.device_ifc;

        mkConnection(coarse_raster.out, fine_raster.in);
        mkConnection(fine_raster.out, pixel_out.in);


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
            coarse_raster.in.enq(triangle);
            delay(10000);
        endseq);

        Reg #(DMA_Req) req <- mkRegU;
        Reg #(Bit #(32)) data <- mkRegU;

        mkAutoFSM(seq
            while(True) seq
                action let v <- pop_o(pixel_out.dma_req); req <= v; endaction
                action let v <- pop_o(pixel_out.dma_data); data <= v; endaction
                $display("%d %d %d", (req.addr - 32'h1000_0000) / 4 % 640, (req.addr - 32'h1000_0000)/ (4 * 640), data);
            endseq
        endseq);

    endmodule

endpackage