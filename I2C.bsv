package I2C;

    import StmtFSM :: * ;
    import GetPut :: * ;
    import FIFO :: * ;

    (* always_ready, always_enabled *)
    interface ExtI2C;
        interface Get #(Bool) scl_out;
        interface Get #(Bool) sda_out;
        interface Put #(Bool) sda_in;
    endinterface

    interface I2C;
        method Action start();
        method Action stop();
        method Action write(Bit#(8) data);
        method Action read();
        method Bool got_ack();
        method Bit #(8) read_data();
        interface ExtI2C external;
    endinterface
    
    typedef enum {
        START,
        STOP,
        WRITE,
        READ
    } Command
    deriving (Bits, Eq);

    (* synthesize *)
    module mkI2C(I2C);

        Reg #(Bool) scl_out_r <- mkReg (True);
        Reg #(Bool) sda_out_r <- mkReg (True);
        RWire #(Bool) sda_in_w <- mkRWire;

        let sda_high = action sda_out_r <= True; endaction;
        let sda_low = action sda_out_r <= False; endaction;
        let scl_high = action scl_out_r <= True; endaction;
        let scl_low = action scl_out_r <= False; endaction;

        let wait_bit = delay(50000);

        Reg #(Command) cmd <- mkRegU;
        Reg #(Bit#(8)) data_reg <- mkRegU;
        Reg #(Bool) ack_bit <- mkRegU;

        function Stmt write_bit(Bool value);
            seq
                scl_low;
                wait_bit;
                sda_out_r <= value;
                wait_bit;
                scl_high;
                wait_bit;
                wait_bit;
                scl_low;
            endseq;
        endfunction

        function Stmt read_bit;
            seq
                scl_low;
                wait_bit;
                sda_high;
                wait_bit;
                scl_high;
                wait_bit;
                ack_bit <= !fromMaybe(True, sda_in_w.wget());
                wait_bit;
                scl_low;
            endseq;
        endfunction

        Reg #(int) i <- mkRegU;

        let fsm <- mkFSM (seq
                if(cmd == START) seq
                    sda_high;
                    wait_bit;
                    scl_high;
                    wait_bit;
                    sda_low;
                    wait_bit;
                endseq else if(cmd == STOP) seq
                    sda_low;
                    wait_bit;
                    scl_high;
                    wait_bit;
                    sda_high;
                    wait_bit;
                endseq else if(cmd == WRITE) seq
                    for(i <= 0; i < 8; i <= i + 1) seq
                        write_bit(data_reg[7] != 1'b0);
                        data_reg <= data_reg << 1;
                    endseq
                    read_bit;
                endseq else if(cmd == READ) seq
                    for(i <= 0; i < 8; i <= i + 1) seq
                        read_bit;
                        data_reg <= {data_reg[6:0], ack_bit ? 1'b0 : 1'b1};
                    endseq
                    write_bit(True);
                endseq
        endseq);

        method Action start() if(fsm.done);
            cmd <= START;
            fsm.start;
        endmethod

        method Action stop() if(fsm.done);
            cmd <= STOP;
            fsm.start;
        endmethod

        method Action write(Bit#(8) data) if(fsm.done);
            cmd <= WRITE;
            data_reg <= data;
            fsm.start;
        endmethod

        method Action read() if(fsm.done);
            cmd <= READ;
            fsm.start;
        endmethod

        method Bool got_ack() if(fsm.done);
            return ack_bit;
        endmethod
    
        method Bit #(8) read_data() if(fsm.done);
            return data_reg;
        endmethod

        interface ExtI2C external;
            interface Get scl_out = toGet (scl_out_r);
            interface Get sda_out = toGet (sda_out_r);
            interface Put sda_in = toPut (sda_in_w);
        endinterface
    endmodule

endpackage