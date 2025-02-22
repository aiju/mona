package DMA;

    import AXI::*;
    import FIFOF::*;
    import Semi_FIFOF::*;

    typedef struct {
        Bit #(32) addr;
        Bit #(32) len;
    } DMA_Req
    deriving (Bits, FShow);

    interface DMA;
        interface AXI3_Master_IFC #(32, 32, 8) mem_ifc;
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_O #(Bit #(32)) resp;
    endinterface

    (* synthesize *)
    module mkDMA (DMA);
        let resp_fifo_size = 32;

        let xactor <- mkAXI3_Master_Xactor;
        let f_req <- mkFIFOF;
        let f_resp <- mkSizedFIFOF (resp_fifo_size);

        Reg #(Bool) active <- mkReg (False);
        Reg #(Bit #(32)) addr_ctr <- mkRegU;
        Reg #(Bit #(32)) len_ctr <- mkRegU;
        Reg #(Bit #(8)) credits[2] <- mkCReg (2, 0);

        rule rl_start (!active);
            active <= True;
            addr_ctr <= f_req.first.addr;
            len_ctr <= f_req.first.len;
            f_req.deq();
        endrule

        function Bit #(8) burst_len;
            return len_ctr < 64 ? truncate(len_ctr / 4) : 16;
        endfunction

        rule rl_issue (active && credits[0] + burst_len <= resp_fifo_size);
            xactor.rd_addr.enq(AXI3_Rd_Addr {
                araddr: addr_ctr,
                arlen: truncate(burst_len - 1),
                arprot: 3'b000,
                arcache: 4'b0011,
                arlock: 2'b00,
                arburst: 2'b01,
                arsize: 3'b010,
                arid: 8'h00
            });
            if (len_ctr <= 64) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + 64;
                len_ctr <= len_ctr - 64;
            end
            credits[0] <= credits[0] + burst_len;
        endrule

        rule rl_data;
            let r <- pop_o(xactor.rd_data);
            f_resp.enq(r.rdata);
        endrule

        interface mem_ifc = xactor.axi_side;
        interface req = to_FIFOF_I(f_req);
        interface resp = interface FIFOF_O;
            method Bit #(32) first = f_resp.first;
            method Bool notEmpty = f_resp.notEmpty;
            method Action deq;
                credits[1] <= credits[1] - 1;
                f_resp.deq;
            endmethod
        endinterface;
    endmodule

endpackage