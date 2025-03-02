package Texture;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import FineRaster :: *;
    import DMA :: *;
    import UVInterp :: *;
    import CoarseRaster :: *;
    import CBus :: *;
    import ConfigDefs :: *;
    import SpecialFIFOs :: *;
    import Util :: *;

    typedef union tagged {
        void Flush;
        struct {
            Bit #(32) rgba;
            UInt #(11) x;
            UInt #(11) y;
        } Pixel;
    } TextureOut
    deriving (Bits, FShow);

    interface Texture;
        interface FIFOF_I #(UVInterpOut) in;
        interface FIFOF_O #(TextureOut) out;
        interface FIFOF_O #(DMA_Req) dma_req;
        interface FIFOF_I #(Bit #(32)) dma_data;
    endinterface

    module [ModWithConfig] mkTexture(Texture);
        let ifc <- collectCBusIFC(mkTextureExposed);
        return ifc;
    endmodule

    (* synthesize *)
    module mkTextureExposed(IWithCBus#(ConfigBus, Texture));
        let ifc <- exposeCBusIFC(mkTextureInternal);
        return ifc;
    endmodule

    module [ModWithConfig] mkTextureInternal(Texture);
        FIFOF #(UVInterpOut) f_in <- mkPipelineFIFOF;
        FIFOF #(TextureOut) f_out <- mkBypassFIFOF;
        FIFOF #(DMA_Req) f_req <- mkBypassFIFOF;
        FIFOF #(Bit #(32)) f_data <- mkBypassFIFOF;

        Reg #(Bool) texture_en <- mkCBRegRW(cfg_texture_en, False);

        Reg #(Bit #(32)) texture_addr <- mkCBRegRW(cfg_texture_addr, 32'h0000_0000);

        FIFOF #(Tuple3 #(Bool, UInt #(11), UInt #(11))) coords <- mkSizedFIFOF (4);

        rule rl_front_no (!texture_en);
            let data <- pop(f_in);
            case(data) matches
                tagged Flush : begin
                    f_out.enq(tagged Flush);
                end
                tagged Pixel .pixel : begin
                    f_out.enq(tagged Pixel {
                            rgba: {16'b0, pack(pixel.u)[25:18], pack(pixel.v)[25:18]},
                            x: pixel.x,
                            y: pixel.y
                        });
                end
            endcase
        endrule

        rule rl_front_yes (texture_en);
            let data <- pop(f_in);
            case(data) matches
                tagged Flush : begin
                    coords.enq(tuple3(True, ?, ?));
                end
                tagged Pixel .pixel : begin
                    f_req.enq(DMA_Req {
                        addr: texture_addr
                            + 512 * 4 * extend(pack(pixel.v)[25:17])
                            + 4 * extend(pack(pixel.u)[25:17]),
                        len: 4
                    });
                    coords.enq(tuple3(False, pixel.x, pixel.y));
                end
            endcase
        endrule

        rule rl_back (texture_en);
            match {.flush, .x, .y} <- pop(coords);
            if(flush) begin
                f_out.enq(tagged Flush);
            end else begin
                let data <- pop(f_data);
                f_out.enq(tagged Pixel {
                    x: x,
                    y: y,
                    rgba: data
                });
            end
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
        interface dma_req = to_FIFOF_O(f_req);
        interface dma_data = to_FIFOF_I(f_data);
    endmodule

endpackage