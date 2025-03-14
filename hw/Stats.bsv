package Stats;

    import Semi_FIFOF::*;
    import ConfigDefs::*;
    import CBus::*;
    import Vector::*;
    import Connectable::*;

    module [ModWithConfig] mkConnectionStats #(Bit #(12) stat_base, FIFOF_O #(t) left, FIFOF_I #(t) right) (Empty);

        Reg #(Bool) counting <- mkCBRegRW (cfg_stats_running, False);

        Reg #(Bit #(32)) ctr0 <- mkCBRegR (CAddr {a: stat_base, o: 0}, 0);
        Reg #(Bit #(32)) ctr1 <- mkCBRegR (CAddr {a: stat_base + 4, o: 0}, 0);
        Reg #(Bit #(32)) ctr2 <- mkCBRegR (CAddr {a: stat_base + 8, o: 0}, 0);
        Reg #(Bit #(32)) ctr3 <- mkCBRegR (CAddr {a: stat_base + 12, o: 0}, 0);

        rule rl_ctr;
            if(counting) begin
                if(left.notEmpty) begin
                    if(right.notFull)
                        ctr3 <= ctr3 + 1;
                    else
                        ctr2 <= ctr2 + 1;
                end else begin
                    if(right.notFull)
                        ctr1 <= ctr1 + 1;
                    else
                        ctr0 <= ctr0 + 1;
                end
            end
        endrule

        mkConnection(left, right);

    endmodule

endpackage