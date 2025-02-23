package Video;

    import GetPut :: *;
    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import DMA :: *;
    import Vector :: *;

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
        interface Vector #(2, FIFOF_O #(DMA_Req)) dma_req;
        interface Vector #(2, FIFOF_I #(Bit #(32))) dma_resp;
    endinterface

    (* synthesize *)
    module mkVideo(Video);
        Vector #(2, FIFOF #(DMA_Req)) f_dma_req <- replicateM (mkFIFOF);
        Vector #(2, FIFOF #(Bit #(32))) f_dma_resp <- replicateM (mkGFIFOF (False, True));

        Reg #(Bool) div <- mkReg (False);
        Reg #(Bit #(16)) x <- mkReg(0);
        Reg #(Bit #(16)) y <- mkReg(480);

        let framebuffer = 32'h1000_0000;

        Reg #(Bit #(32)) addr_ctr <- mkReg(framebuffer);

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

        rule rl_reset (div && x == 640 + 16 + 96 + 48 - 1 && y == 480 + 10 + 2 + 33 - 1);
            x <= 0;
            y <= 0;
        endrule

        rule rl_issue (div && x == 640 && (y < 479 || y == 480 + 10 + 2 + 33 - 1));
            f_dma_req[0].enq(DMA_Req {addr: addr_ctr, len: 640 * 4});
            f_dma_req[1].enq(DMA_Req {addr: addr_ctr + 640*480*4, len: 640});
            if(y != 478)
                addr_ctr <= addr_ctr + 640 * 4;
            else
                addr_ctr <= framebuffer;
        endrule

        rule rl_pop (div && x < 640 && y < 480 || (y >= 480 && y < 480 + 10 + 2 + 33 - 1));
            if(f_dma_resp[0].notEmpty)
                f_dma_resp[0].deq();
            if(f_dma_resp[1].notEmpty && x[1:0] == 3)
                f_dma_resp[1].deq();
        endrule

        function FIFOF_O #(DMA_Req) f1(Integer i) = to_FIFOF_O(f_dma_req[i]);
        function FIFOF_I #(Bit #(32)) f2(Integer i) = to_FIFOF_I(f_dma_resp[i]);

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
                let a = (f_dma_resp[0].notEmpty ? f_dma_resp[0].first : 32'hFF);
                let b = (f_dma_resp[1].notEmpty ? f_dma_resp[1].first : 32'h00);
                return b[31] != 0 ? b[23:0] : a[23:0];
            endmethod
        endinterface;

        interface dma_req = genWith(f1);
        interface dma_resp = genWith(f2);
    endmodule

endpackage