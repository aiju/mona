package TextOverlay;

    import RegFile :: *;
    import ConfigDefs :: *;
    import CBus :: *;
    import FIFOF :: *;
    import Semi_FIFOF::*;
    import SpecialFIFOs::*;
    import Util :: *;
    `include "Util.defines"

    interface TextOverlay;
        (* always_ready *)
        method Action start(Bit #(16) y);
        (* always_ready *)
        interface FIFOF_O #(Maybe #(Bit #(24))) out;
    endinterface

    `SynthBoundary(mkTextOverlay, mkTextOverlayInternal, TextOverlay)

    module [ModWithConfig] mkTextOverlayInternal(TextOverlay);

        RegFile #(Bit #(12), Bit #(16)) text <- mkRegFileFull;
        RegFile #(Bit #(12), Bit #(8)) font <- mkRegFileFull;

        Reg #(Bool) enable <- mkCBRegRW (cfg_text_en, False);
        Reg #(Bit #(16)) access_data <- mkCBRegRW (CAddr {a: cfg_text_access.a, o: 0}, 0);
        Reg #(Bit #(12)) access_addr <- mkCBRegRW (CAddr {a: cfg_text_access.a, o: 16}, 0);
        Reg #(Bool) access_font <- mkCBRegRW (CAddr {a: cfg_text_access.a, o: 30}, False);
        Reg #(Bool) access_go <- mkCBRegRW (CAddr {a: cfg_text_access.a, o: 31}, False);
        Reg #(Bit #(4)) transparent <- mkCBRegRW (cfg_text_transparent, 0);

        Reg #(Bool) active <- mkReg (False);
        Reg #(Bit #(12)) text_addr <- mkRegU;
        Reg #(Bit #(7)) ctr <- mkRegU;
        Reg #(Bit #(4)) y <- mkRegU;

        FIFOF #(Bit #(16)) char_attr <- mkPipelineFIFOF;
        FIFOF #(Bit #(8)) attr <- mkSizedFIFOF (3);
        FIFOF #(Bit #(12)) char_addr <- mkPipelineFIFOF;
        FIFOF #(Bit #(8)) row <- mkPipelineFIFOF;
        FIFOF #(Maybe #(Bit #(24))) f_out <- mkGFIFOF(False, True);

        rule rl_access (access_go);
            if(access_font)
                font.upd(access_addr, access_data[7:0]);
            else
                text.upd(access_addr, access_data);
            access_go <= False;
        endrule

        rule rl_fetch_char (active);
            char_attr.enq(text.sub(text_addr));
            text_addr <= text_addr + 1;

            ctr <= ctr - 1;
            active <= ctr > 0;
        endrule

        rule rl_char_addr;
            let ch <- pop(char_attr);
            char_addr.enq(14 * extend(ch[7:0]) + extend(y));
            attr.enq(ch[15:8]);
        endrule

        rule rl_fetch_font;
            let addr <- pop(char_addr);
            row.enq(font.sub(addr));
        endrule

        Reg #(Bit #(3)) bit_ctr <- mkReg (0);

        function Bit #(24) color(Bit #(4) a);
            case(a)
            0: return 24'h000000;
            1: return 24'h0000AA;
            2: return 24'h00AA00;
            3: return 24'h00AAAA;
            4: return 24'hAA0000;
            5: return 24'hAA00AA;
            6: return 24'hAA5500;
            7: return 24'hAAAAAA;
            8: return 24'h555555;
            9: return 24'h5555FF;
            10: return 24'h55FF55;
            11: return 24'h55FFFF;
            12: return 24'hFF5555;
            13: return 24'hFF55FF;
            14: return 24'hFFFF55;
            15: return 24'hFFFFFF;
            endcase
        endfunction

        function Action output_pixel(Bit #(1) b, Bit #(8) a);
            action
                let is_fg = y < 14 && b != 0;
                let colindex = is_fg ? a[3:0] : a[7:4];
                if(enable && colindex != transparent)
                    f_out.enq(tagged Valid color(colindex));
                else
                    f_out.enq(tagged Invalid);
            endaction
        endfunction

        rule rl_pixel_fetch;
            output_pixel(row.first[7 - bit_ctr], attr.first);
            if(bit_ctr == 7) begin
                row.deq;
                attr.deq;
            end
            bit_ctr <= bit_ctr + 1;
        endrule

        method Action start(Bit #(16) y_);
            active <= True;
            text_addr <= 80 * extend(y_[9:4]);
            y <= truncate(y_);
            ctr <= 79;
        endmethod

        interface out = to_FIFOF_O(f_out);

    endmodule

endpackage