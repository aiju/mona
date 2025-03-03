package Clear;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import DMA :: *;
    import CBus :: *;
    import ConfigDefs :: *;
    import SpecialFIFOs :: *;
    import Util :: *;
    `include "Util.defines"

    interface Clear;
        interface FIFOF_O #(DMA_Req) dma_req;
        interface FIFOF_O #(Bit #(128)) dma_data;
        interface FIFOF_I #(Bool) dma_resp;
    endinterface

    `SynthBoundary(mkClear, mkClearInternal, Clear)

    module [ModWithConfig] mkClearInternal(Clear);
        FIFOF #(DMA_Req) f_req <- mkBypassFIFOF;
        FIFOF #(Bit #(128)) f_data <- mkBypassFIFOF;

        Reg #(Bit #(10)) outstanding[2] <- mkCReg (2, 0);

        Reg #(Bool) issue_clear <- mkCBRegRW(cfg_control_clear, False);
        Reg #(Bit #(32)) start_addr <- mkCBRegRW(cfg_clear_addr, 32'h1000_0000);
        Reg #(Bit #(32)) data <- mkCBRegRW(cfg_clear_data, 32'h0000_0000);
        Reg #(Bit #(16)) stride <- mkCBRegRW(cfg_clear_stride, 16'h0000);
        Reg #(Bit #(16)) width <- mkCBRegRW(cfg_clear_width, 16'h0000);
        Reg #(Bit #(16)) height <- mkCBRegRW(cfg_clear_height, 16'h0000);
        Reg #(Bool) busy <- mkCBRegR (cfg_status_clear_busy, False);

        Reg #(Bit #(32)) addr <- mkRegU;
        Reg #(Bit #(16)) x <- mkRegU;
        Reg #(Bit #(16)) y <- mkRegU;

        rule rl_start (issue_clear && !busy);
            if(width != 0 && height != 0) begin
                busy <= True;
                addr <= start_addr;
                x <= width;
                y <= height;
            end
            issue_clear <= False;
        endrule

        rule rl_clearing (busy && outstanding[0] != ~0);
            if(y != 0) begin
                if(x == width) begin
                    f_req.enq(DMA_Req {
                        addr: addr,
                        len: extend(width) * 16
                    });
                    addr <= addr + extend(stride) * 16;
                    outstanding[0] <= outstanding[0] + 1;
                end
                f_data.enq({data, data, data, data});
                if(x == 1) begin
                    y <= y - 1;
                    x <= width;
                end else
                    x <= x - 1;
            end else begin
                if(outstanding[0] == 0)
                    busy <= False;
            end
        endrule

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