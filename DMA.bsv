package DMA;

    import AXI::*;
    import FIFOF::*;
    import Semi_FIFOF::*;
    import Vector::*;
    import Util::*;

    typedef 32 WdAddr;
    typedef Bit #(32) Addr;
    typedef 128 WdAxiData;
    typedef 32 WdRdData;
    typedef 32 WdWrData;
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

    interface DMARdChannel;
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_O #(Bit #(WdRdData)) data;
        method ActionValue #(AXI3_Rd_Addr #(WdAddr, WdId)) issue;
        method Action put_data(Bit #(WdAxiData) data);
    endinterface

    module mkDMARdChannel(DMARdChannel);
        let resp_fifo_size = 4 * n_burst;

        let f_req <- mkFIFOF;
        FIFOF #(Bit #(WdAxiData)) f_data <- mkSizedFIFOF (resp_fifo_size);
        FIFOF #(Transfer_Info #(WdRdData)) f_transfers <- mkSizedFIFOF (4);

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
                first: truncate(req.addr / fromInteger(valueOf(WdRdData) / 8)),
                last: truncate((req.addr + req.len - 1) / fromInteger(valueOf(WdRdData) / 8)),
                len: len
            });
        endrule

        Reg #(Addr) beat_ctr <- mkReg (0);
        Reg #(Bit #(TLog #(TDiv #(WdAxiData, WdRdData)))) word_ctr <- mkReg (0);

        method ActionValue #(AXI3_Rd_Addr #(WdAddr, WdId)) issue if(active && credits[0] + burst_len <= fromInteger(resp_fifo_size));
            let addr = AXI3_Rd_Addr {
                araddr: addr_ctr,
                arlen: truncate(burst_len - 1),
                arprot: 3'b000,
                arcache: 4'b0011,
                arlock: 2'b00,
                arburst: 2'b01,
                arsize: fromInteger(valueOf(TLog#(WdAxiData))),
                arid: ?
            };
            if (len_ctr <= fromInteger(n_burst)) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + fromInteger(burst_bytes);
                len_ctr <= len_ctr - fromInteger(n_burst);
            end
            credits[0] <= credits[0] + burst_len;
            return addr;
        endmethod

        method Action put_data(Bit #(WdAxiData) data);
            f_data.enq(data);
        endmethod

        interface req = to_FIFOF_I(f_req);
        interface data = interface FIFOF_O;
            method Bit #(WdRdData) first;
                let data = f_data.first;
                Bit #(32) offset = extend(word_ctr + (beat_ctr == 0 ? f_transfers.first.first : 0));
                return truncate(data >> (offset * fromInteger(valueOf(WdRdData))));
            endmethod
            method Bool notEmpty = f_data.notEmpty;
            method Action deq;
                let ctr = word_ctr + (beat_ctr == 0 ? f_transfers.first.first : 0);
                let last_beat = beat_ctr == f_transfers.first.len - 1;
                let last = last_beat ? f_transfers.first.last : -1;
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

    interface DMAWrChannel;
        interface FIFOF_I #(DMA_Req) req;
        interface FIFOF_I #(Bit #(WdWrData)) data_in;
        interface FIFOF_O #(Bool) resp;
        method ActionValue #(AXI3_Wr_Addr #(WdAddr, WdId)) issue;
        method Action response(AXI3_Wr_Resp #(WdId) r);
        interface FIFOF_O #(AXI3_Wr_Data #(WdAxiData, 0)) data_out;
    endinterface

    module mkDMAWrChannel(DMAWrChannel);
        let data_fifo_size = 4 * n_burst;

        let f_req <- mkFIFOF;
        FIFOF #(AXI3_Wr_Data #(WdAxiData, 0)) f_data <- mkSizedFIFOF (data_fifo_size);
        FIFOF #(Bool) f_outstanding <- mkSizedFIFOF (4);
        FIFOF #(Transfer_Info #(WdWrData)) f_transfers <- mkSizedFIFOF (4);
        let f_resp <- mkFIFOF;

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
                first: truncate(req.addr / fromInteger(valueOf(WdWrData) / 8)),
                last: truncate((req.addr + req.len - 1) / fromInteger(valueOf(WdWrData) / 8)),
                len: len
            });
        endrule

        Reg #(Addr) beat_ctr <- mkReg (0);
        Reg #(Bit #(TLog #(TDiv #(WdAxiData, WdWrData)))) word_ctr <- mkReg (0);
        Reg #(Vector #(TDiv #(WdAxiData, WdWrData), Bit #(WdWrData))) word_buf <- mkRegU;

        method ActionValue #(AXI3_Wr_Addr #(WdAddr, WdId)) issue if(active && credits[0] >= burst_len);
            let addr = AXI3_Wr_Addr {
                awaddr: addr_ctr,
                awlen: truncate(burst_len - 1),
                awprot: 3'b000,
                awcache: 4'b0011,
                awlock: 2'b00,
                awburst: 2'b01,
                awsize: fromInteger(valueOf(TLog#(WdAxiData))),
                awid: ?
            };
            f_outstanding.enq(len_ctr <= fromInteger(n_burst));
            if (len_ctr <= fromInteger(n_burst)) begin
                active <= False;
            end else begin
                addr_ctr <= addr_ctr + fromInteger(burst_bytes);
                len_ctr <= len_ctr - fromInteger(n_burst);
            end
            credits[0] <= credits[0] - burst_len;
            return addr;
        endmethod

        method Action response(AXI3_Wr_Resp #(WdId) r);
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
            method Action enq(Bit #(WdWrData) value);
                let first_beat = beat_ctr == 0;
                let last_beat = beat_ctr == f_transfers.first.len - 1;
                let ctr = word_ctr + (first_beat ? f_transfers.first.first : 0);
                let last = last_beat ? f_transfers.first.last : -1;
                let new_word = word_buf;
                new_word[ctr] = value;
                if(ctr == last) begin
                    Bit #(TLog #(TDiv #(WdAxiData, 8))) first_byte = extend(f_transfers.first.first) * fromInteger(valueOf(WdWrData) / 8);
                    Bit #(TLog #(TDiv #(WdAxiData, 8))) last_byte = (extend(f_transfers.first.last) + 1) * fromInteger(valueOf(WdWrData) / 8) - 1;
                    Bit #(TDiv #(WdAxiData, 8)) wstrb = -1;
                    if(first_beat) wstrb = wstrb << first_byte;
                    if(last_beat) wstrb = wstrb & ~((-1) << last_byte << 1);
                    f_data.enq(AXI3_Wr_Data {
                        wdata: pack(new_word),
                        wstrb: wstrb,
                        wlast: last_beat || (beat_ctr + 1) % fromInteger(n_burst) == 0,
                        wid: ?
                    });
                    credits[1] <= credits[1] + 1;
                    word_ctr <= 0;
                    word_buf <= replicate(32'hA5A5A5A5);/*FIXME*/
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
        interface data_out = to_FIFOF_O(f_data);

    endmodule

    interface DMA #(numeric type num_rd_channels, numeric type num_wr_channels);
        interface AXI3_Master_IFC #(WdAddr, WdAxiData, WdId) mem_ifc;
        interface Vector #(num_rd_channels, FIFOF_I #(DMA_Req)) rd_req;
        interface Vector #(num_rd_channels, FIFOF_O #(Bit #(WdRdData))) rd_data;
        interface Vector #(num_wr_channels, FIFOF_I #(DMA_Req)) wr_req;
        interface Vector #(num_wr_channels, FIFOF_I #(Bit #(WdWrData))) wr_data;
        interface Vector #(num_wr_channels, FIFOF_O #(Bool)) wr_resp;
    endinterface

    module mkDMA (DMA #(num_rd_channels, num_wr_channels));
        let xactor <- mkAXI3_Master_Xactor;
        Vector #(num_rd_channels, DMARdChannel) rd_channels <- replicateM (mkDMARdChannel);
        Vector #(num_wr_channels, DMAWrChannel) wr_channels <- replicateM (mkDMAWrChannel);

        Reg #(Bool) current_wr_active[2] <- mkCReg (2, False);
        Reg #(Bit #(WdId)) current_wr_id[2] <- mkCRegU (2);

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
                wr_channels[r.bid].response(r);
            endrule
        end

        function FIFOF_I #(DMA_Req) get_rd_req(Integer i) = rd_channels[i].req;
        function FIFOF_O #(Bit #(WdRdData)) get_rd_data(Integer i) = rd_channels[i].data;
        function FIFOF_I #(DMA_Req) get_wr_req(Integer i) = wr_channels[i].req;
        function FIFOF_I #(Bit #(WdWrData)) get_wr_data(Integer i) = wr_channels[i].data_in;
        function FIFOF_O #(Bool) get_wr_resp(Integer i) = wr_channels[i].resp;

        interface mem_ifc = xactor.axi_side;
        interface rd_req = genWith(get_rd_req);
        interface rd_data = genWith(get_rd_data);
        interface wr_req = genWith(get_wr_req);
        interface wr_data = genWith(get_wr_data);
        interface wr_resp = genWith(get_wr_resp);

    endmodule

endpackage