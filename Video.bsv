package Video;

    import GetPut :: *;
    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import DMA :: *;
    import Vector :: *;
    import CBus :: *;
    import ConfigDefs :: *;

    (* always_ready *)
    interface Ext_Video;
        method Bool clk;
        method Bool hsync;
        method Bool vsync;
        method Bool de;
        method Bit #(24) data;
    endinterface
    
    interface Video;
        interface Ext_Video ext;
        interface FIFOF_O #(DMA_Req) dma_req;
        interface FIFOF_I #(Bit #(32)) dma_resp;
    endinterface

    module [ModWithConfig] mkVideo(Video);
        FIFOF #(DMA_Req) f_dma_req <- mkFIFOF;
        FIFOF #(Bit #(32)) f_dma_resp <- mkGFIFOF (False, True);

        Reg #(Bool) div <- mkReg (False);
        Reg #(Bit #(16)) x <- mkReg(0);
        Reg #(Bit #(16)) y <- mkReg(480);

        Reg #(Bit #(32)) framebuffer <- mkCBRegRW(cfg_display_framebuffer, 32'h1000_0000);
        Reg #(Bool) seen_vsync <- mkCBRegRC(cfg_status_vsync, False);
        Reg #(Bool) in_vsync <- mkCBRegR(cfg_status_in_vsync, False);

        Reg #(Bit #(32)) addr_ctr <- mkReg(32'h1000_0000);

        rule rl_flip;
            div <= !div;
        endrule

        rule rl_inc_x (div && x < 640 + 16 + 96 + 48 - 1);
            x <= x + 1;
        endrule

        rule rl_inc_y (div && x == 640 + 16 + 96 + 48 - 1 && y < 480 + 10 + 2 + 33 - 1);
            x <= 0;
            y <= y + 1;
        endrule

        rule rl_update_seen_vsync (div && x == 640 + 16 + 96 + 48 - 1 && y == 479);
            seen_vsync <= True;
        endrule

        rule rl_update_in_vsync (div && x == 640 + 16 + 96 + 48 - 1);
            if(y == 479)
                in_vsync <= True;
            if(y == 480 + 10 + 2 + 33 - 2)
                in_vsync <= False;
        endrule

        rule rl_reset (div && x == 640 + 16 + 96 + 48 - 1 && y == 480 + 10 + 2 + 33 - 1);
            x <= 0;
            y <= 0;
        endrule

        Reg #(Bool) issue <- mkReg (False);

        rule rl_issue (div && x == 640 && (y < 479 || y == 480 + 10 + 2 + 33 - 1) && !issue);
            issue <= True;
        endrule

        rule rl_issue_go (issue);
            let new_addr = y < 479 ? addr_ctr + 640 * 4 : framebuffer;
            f_dma_req.enq(DMA_Req {addr: new_addr, len: 640 * 4});
            addr_ctr <= new_addr;
            issue <= False;
        endrule

        rule rl_pop (div && x < 640 && y < 480 || (y >= 480 && y < 480 + 10 + 2 + 33 - 1));
            if(f_dma_resp.notEmpty)
                f_dma_resp.deq();
        endrule

        interface ext = interface Ext_Video;
            method clk;
                return div;
            endmethod

            method hsync;
                return x >= 640+16 && x < 640+16+96;
            endmethod

            method vsync;
                return y >= 480+10 && y < 480+10+2;
            endmethod

            method de;
                return x < 640 && y < 480;
            endmethod

            method data;
                return f_dma_resp.notEmpty ? f_dma_resp.first[23:0] : 24'hFF;
            endmethod
        endinterface;

        interface dma_req = to_FIFOF_O(f_dma_req);
        interface dma_resp = to_FIFOF_I(f_dma_resp);
    endmodule

endpackage