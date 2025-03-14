package DepthTest_tb;

    import DMA :: *;
    import DepthTest :: *;
    import CBus :: *;
    import StmtFSM::*;
    import Semi_FIFOF::*;
    import FIFOF::*;
    import FineRaster::*;
    import Vector::*;
    import CoarseRaster::*;

    module mkDepthTest_tb(Empty);
        let dut_ <- exposeCBusIFC(mkDepthTest);
        let dut = dut_.device_ifc;

        mkAutoFSM(seq
            dut.in.enq(tagged Tile {
                tx: 4,
                ty: 4,
                pixels: -1,
                edge_fns: replicate(EdgeFn { a: 1<<11, x: 0, y: 0 }),
                uv: ?
            });
            dut.in.enq(tagged Tile {
                tx: 4,
                ty: 4,
                pixels: 1,
                edge_fns: replicate(EdgeFn { a: 1<<11, x: 0, y: 0 }),
                uv: ?
            });
            dut.in.enq(tagged Tile {
                tx: 4,
                ty: 4,
                pixels: 8,
                edge_fns: replicate(EdgeFn { a: 2<<11, x: 0, y: 0 }),
                uv: ?
            });
            dut.in.enq(tagged Tile {
                tx: 4+16,
                ty: 4+16,
                pixels: -1,
                edge_fns: replicate(EdgeFn { a: 1<<11, x: 0, y: 0 }),
                uv: ?
            });
            delay(1000);
        endseq);

        Reg #(DMA_Req) rreq <- mkRegU;

        mkAutoFSM(seq
            while(True) seq
                action let v <- pop_o(dut.rd_req); rreq <= v; $display(fshow(v)); endaction
                dut.rd_data.enq(0);
                if(rreq.len == 32)
                    dut.rd_data.enq(0);
            endseq
        endseq);

        rule rl_out;
            let v <- pop_o(dut.out);
            $display("out   %d %d %x", v.Tile.tx, v.Tile.ty, v.Tile.pixels);
        endrule

        rule rl_wr_req;
            let v <- pop_o(dut.wr_req);
            $display("write ", fshow(v));
        endrule

        rule rl_wr_data;
            let v <- pop_o(dut.wr_data);
            $display("write ", fshow(v));
        endrule

    endmodule

endpackage