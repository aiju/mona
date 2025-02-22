package foo;

    import I2C :: *;
    import Video :: *;
    import Connectable::*;
    import StmtFSM::*;
    import AXI :: *;

interface TopLevel;
    method Bit #(8) led();
    interface ExtI2C ext_i2c;
    interface Video ext_video;
    interface AXI3_Master_IFC #(32, 32, 8) fpga_to_hps;
endinterface


(* synthesize, always_ready = "led" *)
module mkTopLevel(TopLevel);

    Video video <- mkVideo;

    I2C i2c <- mkI2C;

    function Stmt set_reg(Bit #(8) addr, Bit #(8) data);
        seq
            i2c.start();
            i2c.write(8'h72);
            i2c.write(addr);
            i2c.write(data);
            i2c.stop();
        endseq;
    endfunction
    
    mkAutoFSM(seq
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
    endseq);

    interface ExtI2C ext_i2c = i2c.external;
    interface Video ext_video = video;

endmodule

endpackage