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

    interface DMARdChannel;
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_O #(Bit #(32)) data;
        method ActionValue #(AXI3_Rd_Addr #(32, 8)) issue;
        method Action put_data(Bit #(32) data);
    endinterface

    module mkDMARdChannel(DMARdChannel);
        let resp_fifo_size = 64;

        let f_req <- mkFIFOF;
        let f_data <- mkSizedFIFOF (resp_fifo_size);

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
                arid: ?
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
            f_data.enq(data);
        endmethod

        interface req = to_FIFOF_I(f_req);
        interface data = interface FIFOF_O;
            method Bit #(32) first = f_data.first;
            method Bool notEmpty = f_data.notEmpty;
            method Action deq;
                credits[1] <= credits[1] - 1;
                f_data.deq;
            endmethod
        endinterface;
    endmodule

    interface DMAWrChannel;
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_I #(Bit #(32)) data_in;
        interface FIFOF_O #(Bool) resp;
        method ActionValue #(AXI3_Wr_Addr #(32, 8)) issue;
        method Action response(AXI3_Wr_Resp #(8) r);
        interface FIFOF_O #(Bit #(32)) data_out;
    endinterface

    module mkDMAWrChannel(DMAWrChannel);
        let data_fifo_size = 64;

        let f_req <- mkFIFOF;
        let f_data <- mkSizedFIFOF (data_fifo_size);
        FIFOF #(Bool) f_outstanding <- mkFIFOF;
        let f_resp <- mkFIFOF;

        Reg #(Bool) active <- mkReg (False);
        Reg #(Bit #(32)) addr_ctr <- mkRegU;
        Reg #(Bit #(32)) len_ctr <- mkRegU;
        Reg #(Bit #(8)) credits[2] <- mkCReg (2, 0);

        Reg #(Bool) ok <- mkReg(True);

        function Bit #(8) burst_len;
            return len_ctr < 64 ? truncate(len_ctr / 4) : 16;
        endfunction

        rule rl_start (!active);
            active <= True;
            addr_ctr <= f_req.first.addr;
            len_ctr <= f_req.first.len;
            f_req.deq();
        endrule

        method ActionValue #(AXI3_Wr_Addr #(32, 8)) issue if(active && credits[0] >= burst_len);
            let addr = AXI3_Wr_Addr {
                awaddr: addr_ctr,
                awlen: truncate(burst_len - 1),
                awprot: 3'b000,
                awcache: 4'b0011,
                awlock: 2'b00,
                awburst: 2'b01,
                awsize: 3'b010,
                awid: ?
            };
            f_outstanding.enq(len_ctr <= 64);
            if (len_ctr <= 64) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + 64;
                len_ctr <= len_ctr - 64;
            end
            credits[0] <= credits[0] - burst_len;
            return addr;
        endmethod

        method Action response(AXI3_Wr_Resp #(8) r);
            let last = f_outstanding.first;
            f_outstanding.deq;
            if(last) begin
                f_resp.enq(ok && r.bresp[1] == 1'b0);
                ok <= True;
            end else begin
                if(r.bresp[1] != 1'b0)
                    ok <= False;
            end
        endmethod

        interface req = to_FIFOF_I(f_req);
        interface resp = to_FIFOF_O(f_resp);
        interface data_in = interface FIFOF_I;
            method notFull = f_data.notFull;
            method Action enq(Bit #(32) value);
                f_data.enq(value);
                credits[1] <= credits[1] + 1;
            endmethod
        endinterface;
        interface data_out = to_FIFOF_O(f_data);

    endmodule

    interface DMA #(numeric type num_rd_channels, numeric type num_wr_channels);
        interface AXI3_Master_IFC #(32, 32, 8) mem_ifc;
        interface Vector #(num_rd_channels, FIFOF_I #(DMA_Req)) rd_req;
        interface Vector #(num_rd_channels, FIFOF_O #(Bit #(32))) rd_data;
        interface Vector #(num_wr_channels, FIFOF_I #(DMA_Req)) wr_req;
        interface Vector #(num_wr_channels, FIFOF_I #(Bit #(32))) wr_data;
        interface Vector #(num_wr_channels, FIFOF_O #(Bool)) wr_resp;
    endinterface

    module mkDMA (DMA #(num_rd_channels, num_wr_channels));
        let xactor <- mkAXI3_Master_Xactor;
        Vector #(num_rd_channels, DMARdChannel) rd_channels <- replicateM (mkDMARdChannel);
        Vector #(num_wr_channels, DMAWrChannel) wr_channels <- replicateM (mkDMAWrChannel);

        Reg #(Bool) current_wr_active[2] <- mkCReg (2, False);
        Reg #(Bit #(4)) current_wr_len[2] <- mkCRegU (2);
        Reg #(Bit #(8)) current_wr_id[2] <- mkCRegU (2);

        for(Integer i = 0; i < valueOf(num_rd_channels); i = i + 1)
            rule rl_rd_issue;
                let r <- rd_channels[i].issue;
                r.arid = fromInteger(i);
                xactor.rd_addr.enq(r);
            endrule

        for(Integer i = 0; i < valueOf(num_wr_channels); i = i + 1)
            rule rl_wr_issue (!current_wr_active[1]);
                let r <- wr_channels[i].issue;
                r.awid = fromInteger(i);
                current_wr_active[1] <= True;
                current_wr_len[1] <= r.awlen;
                current_wr_id[1] <= fromInteger(i);
                xactor.wr_addr.enq(r);
            endrule

        if(valueOf(num_rd_channels) > 0)
            rule rl_rd_data;
                let r <- pop_o(xactor.rd_data);
                rd_channels[r.rid].put_data(r.rdata);
            endrule

        if(valueOf(num_wr_channels) > 0) begin
            rule rl_wr_data (current_wr_active[0]);
                let r <- pop_o(wr_channels[current_wr_id[0]].data_out);
                xactor.wr_data.enq(AXI3_Wr_Data {
                    wdata: r,
                    wid: current_wr_id[0],
                    wlast: current_wr_len[0] == 0,
                    wstrb: ~0
                });
                if(current_wr_len[0] > 0) 
                    current_wr_len[0] <= current_wr_len[0] - 1;
                else
                    current_wr_active[0] <= False;
            endrule

            rule rl_wr_resp;
                let r <- pop_o(xactor.wr_resp);
                wr_channels[r.bid].response(r);
            endrule
        end

        function FIFOF_I #(DMA_Req) get_rd_req(Integer i) = rd_channels[i].req;
        function FIFOF_O #(Bit #(32)) get_rd_data(Integer i) = rd_channels[i].data;
        function FIFOF_I #(DMA_Req) get_wr_req(Integer i) = wr_channels[i].req;
        function FIFOF_I #(Bit #(32)) get_wr_data(Integer i) = wr_channels[i].data_in;
        function FIFOF_O #(Bool) get_wr_resp(Integer i) = wr_channels[i].resp;

        interface mem_ifc = xactor.axi_side;
        interface rd_req = genWith(get_rd_req);
        interface rd_data = genWith(get_rd_data);
        interface wr_req = genWith(get_wr_req);
        interface wr_data = genWith(get_wr_data);
        interface wr_resp = genWith(get_wr_resp);

    endmodule

endpackage