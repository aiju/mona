package Full_tb;

    import CoarseRaster::*;
    import FineRaster::*;
    import PixelOut::*;
    import Connectable::*;
    import StmtFSM::*;
    import Semi_FIFOF::*;
    import Vector::*;

    module mkFull_tb();

        let coarse_raster <- mkCoarseRaster;
        let fine_raster <- mkFineRaster;
        let pixel_out <- mkPixelOut;

        mkConnection(coarse_raster.out, fine_raster.in);
        mkConnection(fine_raster.out, pixel_out.in);

        mkAutoFSM(seq
            coarse_raster.in.enq(CoarseRasterIn {
                edge_fns: unpack({
                    -27'd655,
                    27'd0,
                    27'd131072,
                    27'd655,
                    -27'd655,
                    27'd0,
                    27'd0,
                    27'd655,
                    -27'd65536
                }),
                uv: unpack({
                    27'd0,
                    27'd1<<24,
                    27'd1<<24,
                    27'd0,
                    27'd0,
                    27'd0
                }),
                min_x: 0,
                max_x: 50,
                min_y: 0,
                max_y: 50
            });
            delay(10000);
        endseq);

        rule rl_out_req;
            let data <- pop_o(pixel_out.dma_req);
            $display(fshow(data));
        endrule

        rule rl_out_data;
            let data <- pop_o(pixel_out.dma_data);
            $display(fshow(data));
        endrule

    endmodule

endpackage