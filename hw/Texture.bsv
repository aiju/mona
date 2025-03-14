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
    import Defs :: *;
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
        Reg #(Bit #(3)) texture_width <- mkCBRegRW(cfg_texture_width, 0);
        Reg #(Bit #(3)) texture_height <- mkCBRegRW(cfg_texture_height, 0);
        Reg #(Bit #(3)) texture_stride <- mkCBRegRW(cfg_texture_stride, 0);
        Reg #(TexWrapMode) texture_wrap_mode <- mkCBRegRW(cfg_texture_wrap_mode, WRAP_MODE_WRAP);
        Reg #(Bit #(32)) texture_border <- mkCBRegRW(cfg_texture_border, 32'h0000_0000);

        function Tuple2 #(Bit #(10), Bool) clamp_or_wrap(TexCoord u, Bit #(3) size);
            Bit #(16) u0 = u[17:2];
            Bit #(16) mask = (~16'h0 << 3) << size;
            let clamp = texture_wrap_mode != WRAP_MODE_WRAP;
            let uover = (u0 & mask) > 16'h8000;
            let uunder = (u0 & mask) < 16'h8000;
            let border = texture_wrap_mode == WRAP_MODE_CLAMP_TO_BORDER && (uover || uunder);
            return tuple2(truncate(
                    clamp && uover  ? 16'h7fff & ~mask :
                    clamp && uunder ? 0 :
                    u0 & ~mask
            ), border);
        endfunction

        function Maybe #(Bit #(32)) calc_addr(TexCoord u, TexCoord v);
            match {.u0, .uborder} = clamp_or_wrap(u, texture_width);
            match {.v0, .vborder} = clamp_or_wrap(v, texture_height);
            let addr = texture_addr + 4 * (extend(u0) + (extend(v0) << 3 << texture_stride));
            if(uborder || vborder)
                return tagged Invalid;
            else
                return tagged Valid (addr);
        endfunction

        FIFOF #(Tuple2 #(UVInterpOut, Bool)) bypass <- mkSizedFIFOF (20);

        rule rl_front_no (!texture_en);
            let data <- pop(f_in);
            case(data) matches
                tagged Flush : begin
                    f_out.enq(tagged Flush);
                end
                tagged Pixel .pixel : begin
                    f_out.enq(tagged Pixel {
                            rgba: {8'h00, pack(pixel.per_vertex.rgb)},
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
                    bypass.enq(tuple2(data, False));
                end
                tagged Pixel .pixel : begin
                    let addr = calc_addr(pixel.per_vertex.uv[0], pixel.per_vertex.uv[1]);
                    if(addr matches tagged Valid .a)
                        f_req.enq(DMA_Req {
                            addr: a,
                            len: 4
                        });
                    bypass.enq(tuple2(data, !isValid(addr)));
                end
            endcase
        endrule

        rule rl_back (texture_en);
            match {.in, .is_border} <- pop(bypass);
            case(in) matches
                tagged Flush : begin
                    f_out.enq(tagged Flush);
                end
                tagged Pixel .p : begin
                    let t_data;
                    if(is_border)
                        t_data = texture_border;
                    else
                        t_data <- pop(f_data);
                    Bit #(16) x0 = extend(t_data[7:0]) * extend(p.per_vertex.rgb[0]);
                    Bit #(16) x1 = extend(t_data[15:8]) * extend(p.per_vertex.rgb[1]);
                    Bit #(16) x2 = extend(t_data[23:16]) * extend(p.per_vertex.rgb[2]);
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