package DMA;

    import AXI::*;
    import FIFOF::*;
    import Semi_FIFOF::*;
    import Vector::*;
    import Util::*;
    import SpecialFIFOs::*;

    typedef 32 WdAddr;
    typedef Bit #(32) Addr;
    typedef 128 WdAxiData;
    typedef 8 WdId;
    typedef 16 Burst;

    Integer n_burst = valueOf(Burst);
    Integer beat_bytes = valueOf(WdAxiData) / 8;
    Integer burst_bytes = n_burst * beat_bytes;

    typedef struct {
        Addr addr;
        Addr len;
    } DMA_Req
    deriving (Bits, FShow);

    typedef struct {
        Bit #(TLog #(TDiv #(WdAxiData, n))) first;
        Bit #(TLog #(TDiv #(WdAxiData, n))) last;
        Addr len;
    } Transfer_Info #(numeric type n)
    deriving (Bits, FShow);

    interface DMARdChannel #(numeric type wd_data);
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_O #(Bit #(wd_data)) data;
        interface AXI3_Rd_Master #(WdAddr, WdAxiData, WdId) axi;
    endinterface

    module mkDMARdChannel(DMARdChannel #(wd_data))
            provisos (Add#(a__, wd_data, WdAxiData), Add#(b__, TLog#(TDiv#(WdAxiData, wd_data)), 32));
        let resp_fifo_size = 4 * n_burst;

        let f_req <- mkPipelineFIFOF;
        let f_rd_addr <- mkBypassFIFOF;

        FIFOF #(Bit #(WdAxiData)) f_data <- mkSizedFIFOF (resp_fifo_size);
        FIFOF #(Transfer_Info #(wd_data)) f_transfers <- mkSizedFIFOF (4);

        Reg #(Bool) active <- mkReg (False);
        Reg #(Addr) addr_ctr <- mkRegU;
        Reg #(Addr) len_ctr <- mkRegU;
        Reg #(Bit #(8)) credits[2] <- mkCReg (2, 0);

        function Bit #(8) burst_len;
            return len_ctr < fromInteger(n_burst) ?
                truncate(len_ctr)
                : fromInteger(n_burst);
        endfunction

        rule rl_start (!active);
            active <= True;
            let req <- pop(f_req);
            let first_addr = req.addr & -fromInteger(beat_bytes);
            let last_addr = (req.addr + req.len - 1) & -fromInteger(beat_bytes);
            let len = (last_addr - first_addr + fromInteger(beat_bytes)) / fromInteger(beat_bytes);
            addr_ctr <= first_addr;
            len_ctr <= len;
            f_transfers.enq(Transfer_Info {
                first: truncate(req.addr / fromInteger(valueOf(wd_data) / 8)),
                last: truncate((req.addr + req.len - 1) / fromInteger(valueOf(wd_data) / 8)),
                len: len
            });
        endrule

        Reg #(Addr) beat_ctr <- mkReg (0);
        Reg #(Bit #(TLog #(TDiv #(WdAxiData, wd_data)))) word_ctr <- mkReg (0);

        rule rl_issue if(active && credits[0] + burst_len <= fromInteger(resp_fifo_size));
            f_rd_addr.enq(AXI3_Rd_Addr {
                araddr: addr_ctr,
                arlen: truncate(burst_len - 1),
                arprot: 3'b000,
                arcache: 4'b0011,
                arlock: 2'b00,
                arburst: 2'b01,
                arsize: fromInteger(valueOf(TLog#(WdAxiData))),
                arid: ?
            });
            if (len_ctr <= fromInteger(n_burst)) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + fromInteger(burst_bytes);
                len_ctr <= len_ctr - fromInteger(n_burst);
            end
            credits[0] <= credits[0] + burst_len;
        endrule

        interface axi = interface AXI3_Rd_Master;
            interface rd_addr = to_FIFOF_O(f_rd_addr);
            interface rd_data = interface FIFOF_I;
                method notFull = f_data.notFull;
                method Action enq(x) = f_data.enq(x.rdata);
            endinterface;
        endinterface;

        interface req = to_FIFOF_I(f_req);
        interface data = interface FIFOF_O;
            method Bit #(wd_data) first;
                let data = f_data.first;
                Bit #(32) offset = extend(word_ctr + (beat_ctr == 0 ? f_transfers.first.first : 0));
                return truncate(data >> (offset * fromInteger(valueOf(wd_data))));
            endmethod
            method Bool notEmpty = f_data.notEmpty;
            method Action deq;
                let ctr = word_ctr + (beat_ctr == 0 ? f_transfers.first.first : 0);
                let last_beat = beat_ctr == f_transfers.first.len - 1;
                let last = last_beat ? f_transfers.first.last : ~0;
                if(ctr == last) begin
                    word_ctr <= 0;
                    if(last_beat) begin
                        beat_ctr <= 0;
                        f_transfers.deq;
                    end else
                        beat_ctr <= beat_ctr + 1;
                    f_data.deq;
                    credits[1] <= credits[1] - 1;
                end else
                    word_ctr <= word_ctr + 1;
            endmethod
        endinterface;
    endmodule

    interface DMAWrChannel #(numeric type wd_data);
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_I #(Bit #(wd_data)) data;
        interface FIFOF_O #(Bool) resp;
        interface AXI3_Wr_Master #(WdAddr, WdAxiData, WdId) axi;
    endinterface

    module mkDMAWrChannel(DMAWrChannel #(wd_data))
            provisos (Add#(a__, wd_data, WdAxiData),
                Add#(b__, TLog#(TDiv#(WdAxiData, wd_data)), 32),
                Mul#(TDiv#(WdAxiData, wd_data), wd_data, WdAxiData),
                Add#(c__, TLog#(TDiv#(128, wd_data)), 4));
        let data_fifo_size = 4 * n_burst;

        let f_req <- mkPipelineFIFOF;
        FIFOF #(AXI3_Wr_Data #(WdAxiData, WdId)) f_data <- mkSizedFIFOF (data_fifo_size);
        FIFOF #(Bool) f_outstanding <- mkSizedFIFOF (4);
        FIFOF #(Transfer_Info #(wd_data)) f_transfers <- mkSizedFIFOF (4);
        let f_resp <- mkFIFOF;

        let f_wr_addr <- mkBypassFIFOF;
        let f_wr_resp <- mkPipelineFIFOF;

        Reg #(Bool) active <- mkReg (False);
        Reg #(Addr) addr_ctr <- mkRegU;
        Reg #(Addr) len_ctr <- mkRegU;
        Reg #(Bit #(8)) credits[2] <- mkCReg (2, 0);

        Reg #(Bool) ok <- mkReg(True);

        function Bit #(8) burst_len;
            return len_ctr < fromInteger(n_burst) ?
                truncate(len_ctr)
                : fromInteger(n_burst);
        endfunction

        rule rl_start (!active);
            active <= True;
            let req <- pop(f_req);
            let first_addr = req.addr & -fromInteger(beat_bytes);
            let last_addr = (req.addr + req.len - 1) & -fromInteger(beat_bytes);
            let len = (last_addr - first_addr + fromInteger(beat_bytes)) / fromInteger(beat_bytes);
            addr_ctr <= first_addr;
            len_ctr <= len;
            f_transfers.enq(Transfer_Info {
                first: truncate(req.addr / fromInteger(valueOf(wd_data) / 8)),
                last: truncate((req.addr + req.len - 1) / fromInteger(valueOf(wd_data) / 8)),
                len: len
            });
        endrule

        Reg #(Addr) beat_ctr <- mkReg (0);
        Reg #(Bit #(TLog #(TDiv #(WdAxiData, wd_data)))) word_ctr <- mkReg (0);
        Reg #(Vector #(TDiv #(WdAxiData, wd_data), Bit #(wd_data))) word_buf <- mkRegU;

        rule rl_issue if(active && credits[0] >= burst_len);
            f_wr_addr.enq(AXI3_Wr_Addr {
                awaddr: addr_ctr,
                awlen: truncate(burst_len - 1),
                awprot: 3'b000,
                awcache: 4'b0011,
                awlock: 2'b00,
                awburst: 2'b01,
                awsize: fromInteger(valueOf(TLog#(WdAxiData))),
                awid: ?
            });
            f_outstanding.enq(len_ctr <= fromInteger(n_burst));
            if (len_ctr <= fromInteger(n_burst)) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + fromInteger(burst_bytes);
                len_ctr <= len_ctr - fromInteger(n_burst);
            end
            credits[0] <= credits[0] - burst_len;
        endrule

        rule rl_response;
            let r <- pop(f_wr_resp);
            let last = f_outstanding.first;
            f_outstanding.deq;
            if(last) begin
                f_resp.enq(ok && r.bresp[1] == 1'b0);
                ok <= True;
            end else begin
                if(r.bresp[1] != 1'b0)
                    ok <= False;
            end
        endrule

        interface req = to_FIFOF_I(f_req);
        interface resp = to_FIFOF_O(f_resp);
        interface data = interface FIFOF_I;
            method notFull = f_data.notFull;
            method Action enq(Bit #(wd_data) value);
                let first_beat = beat_ctr == 0;
                let last_beat = beat_ctr == f_transfers.first.len - 1;
                let ctr = word_ctr + (first_beat ? f_transfers.first.first : 0);
                let last = last_beat ? f_transfers.first.last : ~0;
                let new_word = word_buf;
                new_word[ctr] = value;
                if(ctr == last) begin
                    Bit #(TDiv #(WdAxiData, 8)) wstrb = -1;
                    if(valueOf(wd_data) < valueOf(WdAxiData)) begin
                        Bit #(TLog #(TDiv #(WdAxiData, 8))) first_byte = extend(f_transfers.first.first) * fromInteger(valueOf(wd_data) / 8);
                        Bit #(TLog #(TDiv #(WdAxiData, 8))) last_byte = (extend(f_transfers.first.last) + 1) * fromInteger(valueOf(wd_data) / 8) - 1;
                        if(first_beat) wstrb = wstrb << first_byte;
                        if(last_beat) wstrb = wstrb & ~((-1) << last_byte << 1);
                    end
                    f_data.enq(AXI3_Wr_Data {
                        wdata: pack(new_word),
                        wstrb: wstrb,
                        wlast: last_beat || (beat_ctr + 1) % fromInteger(n_burst) == 0,
                        wid: ?
                    });
                    credits[1] <= credits[1] + 1;
                    word_ctr <= 0;
                    word_buf <= replicate(0);/*FIXME*/
                    if(last_beat) begin
                        beat_ctr <= 0;
                        f_transfers.deq;
                    end else
                        beat_ctr <= beat_ctr + 1;
                end else begin
                    word_ctr <= word_ctr + 1;
                    word_buf <= new_word;
                end
            endmethod
        endinterface;
        interface axi = interface AXI3_Wr_Master;
            interface wr_addr = to_FIFOF_O(f_wr_addr);
            interface wr_data = to_FIFOF_O(f_data);
            interface wr_resp = to_FIFOF_I(f_wr_resp);
        endinterface;
    endmodule

    interface Fabric #(numeric type num_rd, numeric type num_wr);
        interface AXI3_Master_IFC #(WdAddr, WdAxiData, WdId) mem_ifc;
        interface Vector #(num_rd, AXI3_Rd_Slave #(WdAddr, WdAxiData, WdId)) rd;
        interface Vector #(num_wr, AXI3_Wr_Slave #(WdAddr, WdAxiData, WdId)) wr;
    endinterface

    module mkFabric (Fabric #(num_rd, num_wr));
        let xactor <- mkAXI3_Master_Xactor;

        Vector #(num_rd, FIFOF #(AXI3_Rd_Addr #(WdAddr, WdId))) rd_addr <- replicateM (mkPipelineFIFOF);
        Vector #(num_rd, FIFOF #(AXI3_Rd_Data #(WdAxiData, WdId))) rd_data <- replicateM (mkBypassFIFOF);
        Vector #(num_wr, FIFOF #(AXI3_Wr_Addr #(WdAddr, WdId))) wr_addr <- replicateM (mkPipelineFIFOF);
        Vector #(num_wr, FIFOF #(AXI3_Wr_Data #(WdAxiData, WdId))) wr_data <- replicateM (mkPipelineFIFOF);
        Vector #(num_wr, FIFOF #(AXI3_Wr_Resp #(WdId))) wr_resp <- replicateM (mkBypassFIFOF);

        Reg #(Bool) current_wr_active[2] <- mkCReg (2, False);
        Reg #(Bit #(WdId)) current_wr_id[2] <- mkCRegU (2);

        for(Integer i = 0; i < valueOf(num_rd); i = i + 1)
            rule rl_rd_issue;
                let r <- pop(rd_addr[i]);
                r.arid = fromInteger(i);
                xactor.rd_addr.enq(r);
            endrule

        for(Integer i = 0; i < valueOf(num_wr); i = i + 1)
            rule rl_wr_issue (!current_wr_active[1]);
                let r <- pop(wr_addr[i]);
                r.awid = fromInteger(i);
                current_wr_active[1] <= True;
                current_wr_id[1] <= fromInteger(i);
                xactor.wr_addr.enq(r);
            endrule

        if(valueOf(num_rd) > 0)
            rule rl_rd_data;
                let r <- pop_o(xactor.rd_data);
                rd_data[r.rid].enq(r);
            endrule

        if(valueOf(num_wr) > 0) begin
            rule rl_wr_data (current_wr_active[0]);
                let r <- pop(wr_data[current_wr_id[0]]);
                xactor.wr_data.enq(AXI3_Wr_Data {
                    wdata: r.wdata,
                    wid: current_wr_id[0],
                    wlast: r.wlast,
                    wstrb: r.wstrb
                });
                if(r.wlast)
                    current_wr_active[0] <= False;
            endrule

            rule rl_wr_resp;
                let r <- pop_o(xactor.wr_resp);
                wr_resp[r.bid].enq(r);
            endrule
        end

        function AXI3_Rd_Slave #(WdAddr, WdAxiData, WdId) get_rd(Integer i) = 
            interface AXI3_Rd_Slave;
                interface rd_addr = to_FIFOF_I(rd_addr[i]);
                interface rd_data = to_FIFOF_O(rd_data[i]);
            endinterface;
        function AXI3_Wr_Slave #(WdAddr, WdAxiData, WdId) get_wr(Integer i) = 
            interface AXI3_Wr_Slave;
                interface wr_addr = to_FIFOF_I(wr_addr[i]);
                interface wr_data = to_FIFOF_I(wr_data[i]);
                interface wr_resp = to_FIFOF_O(wr_resp[i]);
            endinterface;

        interface mem_ifc = xactor.axi_side;
        interface rd = genWith(get_rd);
        interface wr = genWith(get_wr);

    endmodule

endpackage