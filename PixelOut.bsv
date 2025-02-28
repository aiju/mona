package PixelOut;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import FineRaster :: *;
    import DMA :: *;
    import UVInterp :: *;
    import CoarseRaster :: *;
    import CBus :: *;
    import ConfigDefs :: *;

    interface PixelOut;
        interface FIFOF_I #(FineRasterOut) in;
        interface FIFOF_O #(DMA_Req) dma_req;
        interface FIFOF_O #(Bit #(32)) dma_data;
        interface FIFOF_I #(Bool) dma_resp;
    endinterface

    module [ModWithConfig] mkPixelOut(PixelOut);
        FIFOF #(FineRasterOut) f_in <- mkFIFOF;
        FIFOF #(DMA_Req) f_req <- mkFIFOF;
        FIFOF #(Bit #(32)) f_data <- mkFIFOF;

        Reg #(Bool) active <- mkReg (False);
        Reg #(Bit #(4)) ctr <- mkRegU;
        Reg #(FineRasterOut) data <- mkRegU;

        Reg #(Bit #(32)) framebuffer <- mkCBRegRW(cfg_render_target, 32'h1000_0000);

        Reg #(Bool) flushed <- mkCBRegRC(cfg_status_flushed, False);

        UVInterp uv_interp <- mkUVInterp;

        rule rl_start (!active);
            case(f_in.first) matches
                tagged Flush : flushed <= True;
                tagged Tile .p : begin
                    active <= True;
                    data <= f_in.first;
                    ctr <= 0;
                end
            endcase
            f_in.deq;
        endrule

        rule rl_process (active);
            if(data.Tile.pixels[ctr] != 1'b0) begin
                f_req.enq(DMA_Req {
                    addr: framebuffer
                        + 640 * 4 * extend({pack(data.Tile.ty), ctr[3:2]})
                        + 4 * extend({pack(data.Tile.tx), ctr[1:0]}),
                    len: 4
                });
                Vector #(3, Int #(27)) edge_vec = newVector;
                for(Integer i = 0; i < 3; i = i + 1)
                    edge_vec[i] = data.Tile.edge_fns[i].a
                        + data.Tile.edge_fns[i].x * extend(unpack(ctr[1:0]))
                        + data.Tile.edge_fns[i].y * extend(unpack(ctr[3:2]));
                uv_interp.in.enq(UVInterpIn {
                    edge_vec: edge_vec,
                    uv: data.Tile.uv
                });
            end
            if(ctr == 15)
                active <= False;
            else
                ctr <= ctr + 1;
        endrule

        rule rl_data;
            let d <- pop_o(uv_interp.out);
            f_data.enq({16'b0, pack(d.u)[25:18], pack(d.v)[25:18]});
        endrule

        interface in = to_FIFOF_I(f_in);
        interface dma_req = to_FIFOF_O(f_req);
        interface dma_data = to_FIFOF_O(f_data);
        interface dma_resp = interface FIFOF_I;
            method notFull = True;
            method enq(x) = noAction;
        endinterface;
    endmodule

endpackage