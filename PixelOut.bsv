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

    interface PixelOut;
        interface FIFOF_I #(UVInterpOut) in;
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
        FIFOF #(UVInterpOut) f_in <- mkPipelineFIFOF;
        FIFOF #(DMA_Req) f_req <- mkBypassFIFOF;
        FIFOF #(Bit #(32)) f_data <- mkBypassFIFOF;

        Reg #(Bit #(10)) outstanding[2] <- mkCReg (2, 0);

        Reg #(Bit #(32)) framebuffer <- mkCBRegRW(cfg_render_target, 32'h1000_0000);
        Reg #(Bool) flushed <- mkCBRegRC(cfg_status_flushed, False);

        Reg #(Bool) issue_clear <- mkCBRegRW(cfg_control_clear, False);
        Reg #(Bit #(32)) clear_start_addr <- mkCBRegRW(cfg_clear_addr, 32'h1000_0000);
        Reg #(Bit #(32)) clear_data <- mkCBRegRW(cfg_clear_data, 32'h0000_0000);
        Reg #(Bit #(16)) clear_stride <- mkCBRegRW(cfg_clear_stride, 16'h0000);
        Reg #(Bit #(16)) clear_width <- mkCBRegRW(cfg_clear_width, 16'h0000);
        Reg #(Bit #(16)) clear_height <- mkCBRegRW(cfg_clear_height, 16'h0000);
        Reg #(Bool) clear_busy <- mkCBRegR (cfg_status_clear_busy, False);

        Reg #(Bit #(32)) clear_addr <- mkRegU;
        Reg #(Bit #(16)) clear_x <- mkRegU;
        Reg #(Bit #(16)) clear_y <- mkRegU;

        rule rl_clear_start (issue_clear && !clear_busy);
            if(clear_width != 0 && clear_height != 0) begin
                clear_busy <= True;
                clear_addr <= clear_start_addr;
                clear_x <= clear_width;
                clear_y <= clear_height;
            end
            issue_clear <= False;
        endrule

        rule rl_clearing (clear_busy && outstanding[0] != ~0);
            if(clear_y != 0) begin
                if(clear_x == clear_width) begin
                    f_req.enq(DMA_Req {
                        addr: clear_addr,
                        len: extend(clear_width) * 4
                    });
                    clear_addr <= clear_addr + extend(clear_stride) * 4;
                    outstanding[0] <= outstanding[0] + 1;
                end
                f_data.enq(clear_data);
                if(clear_x == 1) begin
                    clear_y <= clear_y - 1;
                    clear_x <= clear_width;
                end else
                    clear_x <= clear_x - 1;
            end else begin
                if(outstanding[0] == 0)
                    clear_busy <= False;
            end
        endrule

        rule rl_process (!clear_busy && outstanding[0] != ~0);
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
                    f_data.enq({16'b0, pack(pixel.u)[25:18], pack(pixel.v)[25:18]});
                    outstanding[0] <= outstanding[0] + 1;
                end
            endcase
        endrule

        interface in = to_FIFOF_I(f_in);
        interface dma_req = to_FIFOF_O(f_req);
        interface dma_data = to_FIFOF_O(f_data);
        interface dma_resp = interface FIFOF_I;
            method notFull = True;
            method Action enq(x);
                outstanding[1] <= outstanding[1] - 1;
            endmethod
        endinterface;
    endmodule

endpackage