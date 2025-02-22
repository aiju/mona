package AxiToSimple;

    import ClientServer :: *;
    import GetPut :: *;
    import AXI :: *;
    import FIFOF :: *;
    import Semi_FIFOF :: *;

    typedef struct {
        Bit #(32) addr;
        Bool wr;
        Bit #(32) wdata;
    } Simple_Req
    deriving (Bits, FShow);

    typedef struct {
        Bool ok;
        Bit #(32) rdata;
    } Simple_Resp
    deriving (Bits, FShow);

    typedef enum {
        IDLE,
        READ_REQ,
        READ_RESP,
        WRITE_REQ,
        WRITE_RESP,
        WRITE_FINISH
    } AxiToSimple_State
    deriving (Bits, Eq, FShow);

    interface AxiToSimple;
        interface AXI3_Slave_IFC #(32, 32, 12) axi;
        interface Server #(Simple_Req, Simple_Resp) simple;
    endinterface
    
    function ActionValue #(t) pop(FIFOF #(t) fifo);
        actionvalue
            fifo.deq;
            return fifo.first;
        endactionvalue
    endfunction

    module mkAxiToSimple(AxiToSimple);
        let xactor <- mkAXI3_Slave_Xactor;

        FIFOF #(Simple_Req) f_req <- mkFIFOF;
        FIFOF #(Simple_Resp) f_resp <- mkFIFOF;

        Reg #(AxiToSimple_State) state <- mkReg(IDLE);

        Reg #(Bit #(32)) addr <- mkRegU;
        Reg #(Bit #(4)) len <- mkRegU;
        Reg #(Bit #(3)) size <- mkRegU;
        Reg #(Bit #(2)) burst <- mkRegU;
        Reg #(Bit #(4)) total_len <- mkRegU;
        Reg #(Bit #(12)) id <- mkRegU;
        Reg #(Bool) error <- mkRegU;

        rule rl_start_read (state == IDLE);
            let r <- pop_o(xactor.rd_addr);
            state <= READ_REQ;
            addr <= r.araddr;
            len <= r.arlen;
            total_len <= r.arlen;
            size <= r.arsize;
            burst <= r.arburst;
            id <= r.arid;
            error <= False;
        endrule

        rule rl_start_write (state == IDLE);
            let r <- pop_o(xactor.wr_addr);
            state <= WRITE_REQ;
            addr <= r.awaddr;
            len <= r.awlen;
            total_len <= r.awlen;
            size <= r.awsize;
            burst <= r.awburst;
            id <= r.awid;
            error <= False;
        endrule

        rule rl_read_out (state == READ_REQ);
            f_req.enq(Simple_Req {
                addr: addr & -4,
                wr: False,
                wdata: ?
            });

            state <= READ_RESP;
        endrule

        rule rl_read_in (state == READ_RESP);
            let r <- pop(f_resp);
            xactor.rd_data.enq(AXI3_Rd_Data {
                rdata: r.rdata,
                rresp: r.ok ? 2'b00 : 2'b10,
                rid: id,
                rlast: len == 0
            });
            if(len > 0) begin
                addr <= axi3_next_address(addr, size, burst, total_len);
                len <= len - 1;
                state <= READ_REQ;
            end else
                state <= IDLE;
        endrule

        rule rl_write_out (state == WRITE_REQ);
            let d <- pop_o(xactor.wr_data);
            f_req.enq(Simple_Req {
                addr: addr & -4,
                wr: True,
                wdata: d.wdata
            });
            state <= WRITE_RESP;
        endrule

        rule rl_write_in (state == WRITE_RESP);
            let r <- pop(f_resp);
            if(!r.ok)
                error <= True;
            if(len > 0) begin
                addr <= axi3_next_address(addr, size, burst, total_len);
                len <= len - 1;
                state <= WRITE_REQ;
            end else
                state <= WRITE_FINISH;
        endrule

        rule rl_write_finish (state == WRITE_FINISH);
            xactor.wr_resp.enq(AXI3_Wr_Resp {
                bid: id,
                bresp: error ? 2'b10 : 2'b00
            });
            state <= IDLE;
        endrule

        interface AXI3_Slave_IFC axi = xactor.axi_side;
        interface Server simple = toGPServer(f_req, f_resp);
    endmodule

endpackage