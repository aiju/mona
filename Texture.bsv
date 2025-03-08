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
    `include "Util.defines"

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

    `SynthBoundary(mkTexture, mkTextureInternal, Texture)

    module [ModWithConfig] mkTextureInternal(Texture);
        FIFOF #(UVInterpOut) f_in <- mkPipelineFIFOF;
        FIFOF #(TextureOut) f_out <- mkBypassFIFOF;
        FIFOF #(DMA_Req) f_req <- mkBypassFIFOF;
        FIFOF #(Bit #(32)) f_data <- mkPipelineFIFOF;

        Reg #(Bool) texture_en <- mkCBRegRW(cfg_texture_en, False);

        Reg #(Bit #(32)) texture_addr <- mkCBRegRW(cfg_texture_addr, 32'h0000_0000);

        FIFOF #(UVInterpOut) bypass <- mkSizedFIFOF (20);

        rule rl_front_no (!texture_en);
            let data <- pop(f_in);
            case(data) matches
                tagged Flush : begin
                    f_out.enq(tagged Flush);
                end
                tagged Pixel .pixel : begin
                    f_out.enq(tagged Pixel {
                            rgba: {8'h00, pack(pixel.rgb)},
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
                    bypass.enq(data);
                end
                tagged Pixel .pixel : begin
                    f_req.enq(DMA_Req {
                        addr: texture_addr
                            + 512 * 4 * extend(pack(pixel.v)[25:17])
                            + 4 * extend(pack(pixel.u)[25:17]),
                        len: 4
                    });
                    bypass.enq(data);
                end
            endcase
        endrule

        rule rl_back (texture_en);
            let in <- pop(bypass);
            case(in) matches
                tagged Flush : begin
                    f_out.enq(tagged Flush);
                end
                tagged Pixel .p : begin
                    let t_data <- pop(f_data);
                    Bit #(16) x0 = extend(t_data[7:0]) * extend(p.rgb[0]);
                    Bit #(16) x1 = extend(t_data[15:8]) * extend(p.rgb[1]);
                    Bit #(16) x2 = extend(t_data[23:16]) * extend(p.rgb[2]);
                    // FIXME: rounding, should divide by 255 rather than 256
                    Bit #(8) y0 = truncate(x0 >> 8);
                    Bit #(8) y1 = truncate(x1 >> 8);
                    Bit #(8) y2 = truncate(x2 >> 8);
                    f_out.enq(tagged Pixel {
                        x: p.x,
                        y: p.y,
                        rgba: {8'b0, y2, y1, y0}
                    });
                end
            endcase
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
        interface dma_req = to_FIFOF_O(f_req);
        interface dma_data = to_FIFOF_I(f_data);
    endmodule

endpackage