package HdmiCtrl;

    import StmtFSM::*;
    import I2C::*;

    interface HdmiCtrl;
        interface ExtI2C ext_i2c;
        (* always_ready, always_enabled *) method Action hdmi_int(Bool value);
        (* always_ready *) method Bool hdmi_active;
    endinterface

    (* synthesize *)
    module mkHdmiCtrl(HdmiCtrl);
        let i2c <- mkI2C;
        Reg #(Bit #(32)) init_ctr <- mkReg (100_000);
        Reg #(Bool) started <- mkReg (False);
        Reg #(Bit #(8)) read_data <- mkRegU;
        Reg #(Bool) irq <- mkReg (False);
        Reg #(Bool) active <- mkReg (False);

        function Stmt set_reg(Bit #(8) addr, Bit #(8) data);
            seq
                i2c.start();
                i2c.write(8'h72);
                i2c.write(addr);
                i2c.write(data);
                i2c.stop();
            endseq;
        endfunction

        function Stmt get_reg(Bit #(8) addr);
            seq
                i2c.start();
                i2c.write(8'h72);
                i2c.write(addr);
                i2c.start();
                i2c.write(8'h73);
                i2c.read();
                read_data <= i2c.read_data();
                i2c.stop();
            endseq;
        endfunction

        rule rl_init_dec (init_ctr > 0);
            init_ctr <= init_ctr - 1;
        endrule
        
        let fsm <- mkFSM(seq
            // clear interrupts
            set_reg(8'h96, 8'h00);
            // check hot plug and monitor sense status
            get_reg(8'h42);
            active <= read_data[6:5] == 2'b11;
            if(active) seq
                // powerup
                set_reg(8'h41, 8'h00);
                // fixed magic values
                set_reg(8'h98, 8'h03); 
                set_reg(8'h9a, 8'hE0);
                set_reg(8'h9C, 8'h30);
                set_reg(8'h9D, 8'h61);
                set_reg(8'hA2, 8'hA4);
                set_reg(8'hA3, 8'hA4);
                set_reg(8'hE0, 8'hD0);
                set_reg(8'hF9, 8'h00);
                // RGB
                set_reg(8'h15, 8'h00);
                set_reg(8'h16, 8'h30);
                set_reg(8'h17, 8'h00);
            endseq
        endseq);

       rule rl_start (init_ctr == 0 && (!started || irq));
            fsm.start;
            started <= True;
        endrule

        interface ext_i2c = i2c.ext;

        method Action hdmi_int(Bool value);
            irq <= !value;
        endmethod

        method hdmi_active = active;
    endmodule
endpackage