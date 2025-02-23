package DMA;

    import AXI::*;
    import FIFOF::*;
    import Semi_FIFOF::*;
    import Vector::*;

    typedef struct {
        Bit #(32) addr;
        Bit #(32) len;
    } DMA_Req
    deriving (Bits, FShow);

    interface DMAChannel;
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_O #(Bit #(32)) resp;
        method ActionValue #(AXI3_Rd_Addr #(32, 8)) issue;
        method Action put_data(Bit #(32) data);
    endinterface

    module mkDMAChannel(DMAChannel);
        let resp_fifo_size = 64;

        let f_req <- mkFIFOF;
        let f_resp <- mkSizedFIFOF (resp_fifo_size);

        Reg #(Bool) active <- mkReg (False);
        Reg #(Bit #(32)) addr_ctr <- mkRegU;
        Reg #(Bit #(32)) len_ctr <- mkRegU;
        Reg #(Bit #(8)) credits[2] <- mkCReg (2, 0);

        function Bit #(8) burst_len;
            return len_ctr < 64 ? truncate(len_ctr / 4) : 16;
        endfunction

        rule rl_start (!active);
            active <= True;
            addr_ctr <= f_req.first.addr;
            len_ctr <= f_req.first.len;
            f_req.deq();
        endrule

        method ActionValue #(AXI3_Rd_Addr #(32, 8)) issue if(active && credits[0] + burst_len <= resp_fifo_size);
            let addr = AXI3_Rd_Addr {
                araddr: addr_ctr,
                arlen: truncate(burst_len - 1),
                arprot: 3'b000,
                arcache: 4'b0011,
                arlock: 2'b00,
                arburst: 2'b01,
                arsize: 3'b010,
                arid: 8'h00
            };
            if (len_ctr <= 64) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + 64;
                len_ctr <= len_ctr - 64;
            end
            credits[0] <= credits[0] + burst_len;
            return addr;
        endmethod

        method Action put_data(Bit #(32) data);
            f_resp.enq(data);
        endmethod

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

    interface DMA #(numeric type num_channels);
        interface AXI3_Master_IFC #(32, 32, 8) mem_ifc;
        interface Vector #(num_channels, FIFOF_I #(DMA_Req)) req;
        interface Vector #(num_channels, FIFOF_O #(Bit #(32))) resp;
    endinterface

    module mkDMA (DMA #(num_channels));
        let xactor <- mkAXI3_Master_Xactor;
        Vector #(num_channels, DMAChannel) channels <- replicateM (mkDMAChannel);

        for(Integer i = 0; i < valueOf(num_channels); i = i + 1)
            rule rl_issue;
                let r <- channels[i].issue;
                r.arid = fromInteger(i);
                xactor.rd_addr.enq(r);
            endrule

        rule rl_data;
            let r <- pop_o(xactor.rd_data);
            channels[r.rid].put_data(r.rdata);
        endrule

        function FIFOF_I #(DMA_Req) f1(Integer i) = channels[i].req;
        function FIFOF_O #(Bit #(32)) f2(Integer i) = channels[i].resp;

        interface mem_ifc = xactor.axi_side;
        interface req = genWith(f1);
        interface resp = genWith(f2);

    endmodule

endpackage