package DMA_tb;

    import AXI :: *;
    import DMA :: *;
    import Connectable :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;

    module mkDMA_tb(Empty);

        DMA #(2, 1) dma <- mkDMA;

        AXI3_Slave_Xactor_IFC #(32, 128, 8) xactor <- mkAXI3_Slave_Xactor;

        mkConnection(dma.mem_ifc, xactor.axi_side);

        mkAutoFSM(seq
            dma.rd_req[0].enq(DMA_Req {addr: 32'h1000_000C, len: 4 * 1});
            dma.wr_req[0].enq(DMA_Req {addr: 32'h8000_0004, len: 4 * 4});
            dma.wr_data[0].enq(32'hDEADBEEF);
            dma.wr_data[0].enq(32'hCAFEBABE);
            dma.wr_data[0].enq(32'hFEFEFEFE);
            dma.wr_data[0].enq(32'h12345678);
            dma.wr_req[0].enq(DMA_Req {addr: 32'h8000_001C, len: 4 * 1});
            dma.wr_data[0].enq(32'h00010001);
            dma.wr_req[0].enq(DMA_Req {addr: 32'h8000_0014, len: 4 * 1});
            dma.wr_data[0].enq(32'h00020002);
            //dma.rd_req[1].enq(DMA_Req {addr: 32'h1000_0000, len: 4 * 7});
            delay(10000);
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

        rule rl_waddr;
            let r <- pop_o(xactor.wr_addr);
            $display(fshow(r));
        endrule
        rule rl_wdata;
            let r <- pop_o(xactor.wr_data);
            $display(fshow(r));
        endrule

        rule rl_rdata (active);
            let s = AXI3_Rd_Data {
                rdata: {addr + 12, addr + 8, addr + 4, addr + 0},
                rid: id,
                rresp: 2'b00,
                rlast: len == 1
            };
            $display(fshow(s));
            xactor.rd_data.enq(s);
            addr <= addr + 16;
            active <= len > 1;
            len <= len - 1;
        endrule

        rule rl_out;
            let d <- pop_o(dma.rd_data[0]);
            $display("0", fshow(d));
        endrule

        rule rl_out1;
            let d <- pop_o(dma.rd_data[1]);
            $display("1", fshow(d));
        endrule

    endmodule

endpackage