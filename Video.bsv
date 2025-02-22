package Video;

    import GetPut :: *;
    import FIFO :: *;
    
    interface Video;
        (* always_ready *)
        method Bool clk;
        (* always_ready *)
        method Bool hsync;
        (* always_ready *)
        method Bool vsync;
        (* always_ready *)
        method Bool de;
        (* always_ready *)
        method Bit #(24) data;
        interface Put #(Bit #(32)) in_data;
        (* always_ready *)
        method Bool consume_pixel;
    endinterface

    (* synthesize *)
    module mkVideo(Video);
        Reg #(Bool) div <- mkReg (False);
        Reg #(Bit #(16)) x <- mkReg(0);
        Reg #(Bit #(16)) y <- mkReg(0);

        FIFO #(Bit #(32)) fifo <- mkSizedFIFO(64);
        RWire #(Bit #(32)) data_w <- mkRWire;

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

        rule rl_read;
            data_w.wset(fifo.first);
        endrule

        rule rl_pop (div && x < 640 && y < 480);
            fifo.deq();
        endrule

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
            return fromMaybe(0, data_w.wget())[23:0];
        endmethod

        method consume_pixel;
            return div && x < 640 && y < 480;
        endmethod

        interface Put in_data = fifoToPut(fifo);
    endmodule

endpackage