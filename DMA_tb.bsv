package DMA_tb;

    import AXI :: *;
    import DMA :: *;
    import Connectable :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;

    module mkDMA_tb(Empty);

        DMA #(2) dma <- mkDMA;

        AXI3_Slave_Xactor_IFC #(32, 32, 8) xactor <- mkAXI3_Slave_Xactor;

        mkConnection(dma.mem_ifc, xactor.axi_side);

        mkAutoFSM(seq
            dma.req[0].enq(DMA_Req {addr: 32'h1000_0000, len: 4 * (16 * 2 + 3)});
            dma.req[1].enq(DMA_Req {addr: 32'h1000_0000, len: 4 * 7});
            delay(1000);
        endseq);

        Reg #(Bool) active <- mkReg (False);
        Reg #(Bit #(32)) addr <- mkRegU;
        Reg #(Bit #(5)) len <- mkRegU;
        Reg #(Bit #(8)) id <- mkRegU;

        rule rl_raddr (!active);
            let r <- pop_o(xactor.rd_addr);
            $display(fshow(r));
            active <= True;
            addr <= r.araddr;
            len <= extend(r.arlen) + 1;
            id <= r.arid;
        endrule

        rule rl_rdata (active);
            let s = AXI3_Rd_Data {
                rdata: addr,
                rid: id,
                rresp: 2'b00,
                rlast: len == 1
            };
            $display(fshow(s));
            xactor.rd_data.enq(s);
            addr <= addr + 4;
            active <= len > 1;
            len <= len - 1;
        endrule

        rule rl_out;
            let d <- pop_o(dma.resp[0]);
            $display("0", fshow(d));
        endrule

        rule rl_out1;
            let d <- pop_o(dma.resp[1]);
            $display("1", fshow(d));
        endrule

    endmodule

endpackage