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
    import SpecialFIFOs :: *;
    import Util :: *;
    import Texture :: *;

    interface PixelOut;
        interface FIFOF_I #(TextureOut) in;
        interface FIFOF_O #(DMA_Req) dma_req;
        interface FIFOF_O #(Bit #(32)) dma_data;
        interface FIFOF_I #(Bool) dma_resp;
    endinterface

    module [ModWithConfig] mkPixelOut(PixelOut);
        let ifc <- collectCBusIFC(mkPixelOutExposed);
        return ifc;
    endmodule

    (* synthesize *)
    module mkPixelOutExposed(IWithCBus#(ConfigBus, PixelOut));
        let ifc <- exposeCBusIFC(mkPixelOutInternal);
        return ifc;
    endmodule

    module [ModWithConfig] mkPixelOutInternal(PixelOut);
        FIFOF #(TextureOut) f_in <- mkPipelineFIFOF;
        FIFOF #(DMA_Req) f_req <- mkBypassFIFOF;
        FIFOF #(Bit #(32)) f_data <- mkBypassFIFOF;

        Reg #(Bit #(32)) framebuffer <- mkCBRegRW(cfg_render_target, 32'h1000_0000);
        Reg #(Bool) flushed <- mkCBRegRC(cfg_status_flushed, False);

        rule rl_process;
            let data <- pop(f_in);
            case(data) matches
                tagged Flush : flushed <= True;
                tagged Pixel .pixel : begin
                    f_req.enq(DMA_Req {
                        addr: framebuffer
                            + 640 * 4 * extend(pack(pixel.y))
                            + 4 * extend(pack(pixel.x)),
                        len: 4
                    });
                    f_data.enq(pixel.rgba);
                end
            endcase
        endrule

        interface in = to_FIFOF_I(f_in);
        interface dma_req = to_FIFOF_O(f_req);
        interface dma_data = to_FIFOF_O(f_data);
        interface dma_resp = interface FIFOF_I;
            method notFull = True;
            method Action enq(x);
            endmethod
        endinterface;
    endmodule

endpackage