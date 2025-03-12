package Video;

    import GetPut :: *;
    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import DMA :: *;
    import Vector :: *;
    import CBus :: *;
    import ConfigDefs :: *;
    import TextOverlay :: *;
    `include "Util.defines"

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

    `SynthBoundary(mkVideo, mkVideoInternal, Video)

    module [ModWithConfig] mkVideoInternal(Video);
        FIFOF #(DMA_Req) f_dma_req <- mkFIFOF;
        FIFOF #(Bit #(32)) f_dma_resp <- mkGFIFOF (False, True);

        Reg #(Bool) div <- mkReg (False);
        Reg #(Bit #(16)) x <- mkReg(0);
        Reg #(Bit #(16)) y <- mkReg(480);

        Reg #(Bit #(32)) framebuffer <- mkCBRegRW(cfg_display_framebuffer, 32'h1000_0000);
        Reg #(Bool) seen_vsync <- mkCBRegRC(cfg_status_vsync, False);
        Reg #(Bool) in_vsync <- mkCBRegR(cfg_status_in_vsync, False);

        Reg #(Bit #(32)) addr_ctr <- mkReg(32'h1000_0000);

        let text_overlay <- mkTextOverlay;

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_flip;
            div <= !div;
        endrule

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_inc_x (div && x < 640 + 16 + 96 + 48 - 1);
            x <= x + 1;
        endrule

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_inc_y (div && x == 640 + 16 + 96 + 48 - 1 && y < 480 + 10 + 2 + 33 - 1);
            x <= 0;
            y <= y + 1;
        endrule

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_update_seen_vsync (div && x == 640 + 16 + 96 + 48 - 1 && y == 479);
            seen_vsync <= True;
        endrule

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_update_in_vsync (div && x == 640 + 16 + 96 + 48 - 1);
            if(y == 479)
                in_vsync <= True;
            if(y == 480 + 10 + 2 + 33 - 2)
                in_vsync <= False;
        endrule

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_reset (div && x == 640 + 16 + 96 + 48 - 1 && y == 480 + 10 + 2 + 33 - 1);
            x <= 0;
            y <= 0;
        endrule

        Reg #(Bool) issue <- mkReg (False);

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_issue (div && x == 640 && (y < 479 || y == 480 + 10 + 2 + 33 - 1) && !issue);
            issue <= True;
            text_overlay.start(y < 479 ? y + 1 : 0);
        endrule

        rule rl_issue_go (issue);
            let new_addr = y < 479 ? addr_ctr + 640 * 4 : framebuffer;
            f_dma_req.enq(DMA_Req {addr: new_addr, len: 640 * 4});
            addr_ctr <= new_addr;
            issue <= False;
        endrule

        (* fire_when_enabled, no_implicit_conditions *)
        rule rl_pop (div && x < 640 && y < 480 || (y >= 480 && y < 480 + 10 + 2 + 33 - 1));
            if(f_dma_resp.notEmpty)
                f_dma_resp.deq();
            if(text_overlay.out.notEmpty)
                text_overlay.out.deq();
        endrule

        Reg #(Bool) r_clk <- mkRegU;
        Reg #(Bool) r_hsync <- mkRegU;
        Reg #(Bool) r_vsync <- mkRegU;
        Reg #(Bool) r_de <- mkRegU;
        Reg #(Bit #(24)) r_data <- mkRegU;

        rule rl_output;
            r_clk <= div;
            r_hsync <= x >= 640+16 && x < 640+16+96;
            r_vsync <= y >= 480+10 && y < 480+10+2;
            r_de <= x < 640 && y < 480;
            let r = f_dma_resp.first[7:0];
            let g = f_dma_resp.first[15:8];
            let b = f_dma_resp.first[23:16];
            if(text_overlay.out.notEmpty &&& text_overlay.out.first matches tagged Valid .rgba)
                r_data <= rgba[23:0];
            else
                r_data <= f_dma_resp.notEmpty ? {r, g, b} : 24'hFF;
        endrule

        interface ext = interface Ext_Video;
            method clk = r_clk;
            method hsync = r_hsync;
            method vsync = r_vsync;
            method de = r_de;
            method data = r_data;
        endinterface;

        interface dma_req = to_FIFOF_O(f_dma_req);
        interface dma_resp = to_FIFOF_I(f_dma_resp);
    endmodule

endpackage