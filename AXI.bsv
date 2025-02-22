package AXI;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Connectable :: *;

    interface AXI3_Master_IFC #(
        numeric type wd_addr,
        numeric type wd_data,
        numeric type wd_id);
    
        (* always_ready, result="awvalid" *) method Bool m_awvalid;
        (* always_ready, result="awid" *) method Bit #(wd_id) m_awid;
        (* always_ready, result="awaddr" *) method Bit #(wd_addr) m_awaddr;
        (* always_ready, result="awlen" *) method Bit #(4) m_awlen;
        (* always_ready, result="awsize" *) method Bit #(3) m_awsize;
        (* always_ready, result="awburst" *) method Bit #(2) m_awburst;
        (* always_ready, result="awcache" *) method Bit #(4) m_awcache;
        (* always_ready, result="awprot" *) method Bit #(3) m_awprot;
        (* always_ready, result="awlock" *) method Bit #(2) m_awlock;

        (* always_ready, always_enabled, prefix="" *) method Action m_awready (
            (*port="awready"*) Bool awready);
    
        (* always_ready, result="wvalid" *) method Bool m_wvalid;
        (* always_ready, result="wid" *) method Bit #(wd_id) m_wid;
        (* always_ready, result="wdata" *) method Bit #(wd_data) m_wdata;
        (* always_ready, result="wstrb" *) method Bit #(TDiv #(wd_data, 8)) m_wstrb;
        (* always_ready, result="wlast" *) method Bool m_wlast;

        (* always_ready, always_enabled, prefix="" *) method Action m_wready (
            (*port="wready"*) Bool awready);

        (* always_ready, result="bready" *) method Bool m_bready;
        (* always_ready, always_enabled, prefix="" *) method Action m_bvalid (
            (*port="bvalid"*) Bool bvalid,
            (*port="bid"*) Bit #(wd_id) bid,
            (*port="bresp"*) Bit #(2) bresp);

        (* always_ready, result="arvalid" *) method Bool m_arvalid;
        (* always_ready, result="arid" *) method Bit #(wd_id) m_arid;
        (* always_ready, result="araddr" *) method Bit #(wd_addr) m_araddr;
        (* always_ready, result="arlen" *) method Bit #(4) m_arlen;
        (* always_ready, result="arsize" *) method Bit #(3) m_arsize;
        (* always_ready, result="arburst" *) method Bit #(2) m_arburst;
        (* always_ready, result="arcache" *) method Bit #(4) m_arcache;
        (* always_ready, result="arprot" *) method Bit #(3) m_arprot;
        (* always_ready, result="arlock" *) method Bit #(2) m_arlock;

        (* always_ready, always_enabled, prefix="" *) method Action m_arready (
            (*port="arready"*) Bool arready);

        (* always_ready, result="rready" *) method Bool m_rready;
        (* always_ready, always_enabled, prefix="" *) method Action m_rvalid (
            (*port="rvalid"*) Bool rvalid,
            (*port="rid"*) Bit #(wd_id) rid,
            (*port="rresp"*) Bit #(2) rresp,
            (*port="rdata"*) Bit #(wd_data) rdata,
            (*port="rlast"*) Bool rlast);

    endinterface

    interface AXI3_Slave_IFC #(
        numeric type wd_addr,
        numeric type wd_data,
        numeric type wd_id);
    
        (* always_ready, always_enabled, prefix="" *) method Action m_awvalid (
            (* port="awvalid" *) Bool awvalid,
            (* port="awid" *) Bit #(wd_id) awid,
            (* port="awaddr" *) Bit #(wd_addr) awaddr,
            (* port="awlen" *) Bit #(4) awlen,
            (* port="awsize" *) Bit #(3) awsize,
            (* port="awburst" *) Bit #(2) awburst,
            (* port="awcache" *) Bit #(4) awcache,
            (* port="awprot" *) Bit #(3) awprot,
            (* port="awlock" *) Bit #(2) awlock);
        (* always_ready, result="awready" *) method Bool m_awready;
    
        (* always_ready, always_enabled, prefix="" *) method Action m_wvalid (
            (* port="wvalid" *) Bool m_wvalid,
            (* port="wid" *) Bit #(wd_id) m_wid,
            (* port="wdata" *) Bit #(wd_data) m_wdata,
            (* port="wstrb" *) Bit #(TDiv #(wd_data, 8)) m_wstrb,
            (* port="wlast" *) Bool m_wlast);
        (* always_ready, result="wready" *) method Bool m_wready;

        (* always_ready, result="bvalid" *) method Bool m_bvalid;
        (* always_ready, result="bid" *) method Bit #(wd_id) m_bid;
        (* always_ready, result="bresp" *) method Bit #(2) m_bresp;
        (* always_ready, always_enabled, prefix="" *) method Action m_bready (
            (*port="bready"*) Bool bready);

        (* always_ready, always_enabled, prefix="" *) method Action m_arvalid (
            (* port="arvalid" *) Bool arvalid,
            (* port="arid" *) Bit #(wd_id) arid,
            (* port="araddr" *) Bit #(wd_addr) araddr,
            (* port="arlen" *) Bit #(4) arlen,
            (* port="arsize" *) Bit #(3) arsize,
            (* port="arburst" *) Bit #(2) arburst,
            (* port="arcache" *) Bit #(4) arcache,
            (* port="arprot" *) Bit #(3) arprot,
            (* port="arlock" *) Bit #(2) arlock);
        (* always_ready, result="arready" *) method Bool m_arready;

        (* always_ready, result="rvalid" *) method Bool m_rvalid;
        (* always_ready, result="rid" *) method Bit #(wd_id) m_rid;
        (* always_ready, result="rresp" *) method Bit #(2) m_rresp;
        (* always_ready, result="rdata" *) method Bit #(wd_data) m_rdata;
        (* always_ready, result="rlast" *) method Bool m_rlast;
        (* always_ready, always_enabled, prefix="" *) method Action m_rready (
            (*port="bready"*) Bool rready);

    endinterface

    instance Connectable #(
        AXI3_Master_IFC #(wd_addr, wd_data, wd_id),
        AXI3_Slave_IFC  #(wd_addr, wd_data, wd_id));

        module mkConnection #(
            AXI3_Master_IFC #(wd_addr, wd_data, wd_id) axim,
			AXI3_Slave_IFC  #(wd_addr, wd_data, wd_id) axis)
            (Empty);

            (* fire_when_enabled, no_implicit_conditions *)
            rule rl_aw;
                axis.m_awvalid(
                    axim.m_awvalid,
                    axim.m_awid,
                    axim.m_awaddr,
                    axim.m_awlen,
                    axim.m_awsize,
                    axim.m_awburst,
                    axim.m_awcache,
                    axim.m_awprot,
                    axim.m_awlock
                );
                axim.m_awready(axis.m_awready);
            endrule

            (* fire_when_enabled, no_implicit_conditions *)
            rule rl_w;
                axis.m_wvalid(
                    axim.m_wvalid,
                    axim.m_wid,
                    axim.m_wdata,
                    axim.m_wstrb,
                    axim.m_wlast
                );
                axim.m_wready(axis.m_wready);
            endrule

            (* fire_when_enabled, no_implicit_conditions *)
            rule rl_b;
                axim.m_bvalid(
                    axis.m_bvalid,
                    axis.m_bid,
                    axis.m_bresp
                );
                axis.m_bready(axim.m_bready);
            endrule

            (* fire_when_enabled, no_implicit_conditions *)
            rule rl_ar;
                axis.m_arvalid(
                    axim.m_arvalid,
                    axim.m_arid,
                    axim.m_araddr,
                    axim.m_arlen,
                    axim.m_arsize,
                    axim.m_arburst,
                    axim.m_arcache,
                    axim.m_arprot,
                    axim.m_arlock
                );
                axim.m_arready(axis.m_arready);
            endrule

            (* fire_when_enabled, no_implicit_conditions *)
            rule rl_r;
                axim.m_rvalid(
                    axis.m_rvalid,
                    axis.m_rid,
                    axis.m_rresp,
                    axis.m_rdata,
                    axis.m_rlast
                );
                axis.m_rready(axim.m_rready);
            endrule

        endmodule
    endinstance

    typedef struct {
        Bit #(wd_id) awid;
        Bit #(wd_addr) awaddr;
        Bit #(4) awlen;
        Bit #(3) awsize;
        Bit #(2) awburst;
        Bit #(2) awlock;
        Bit #(4) awcache;
        Bit #(3) awprot;
    } AXI3_Wr_Addr #(numeric type wd_addr, numeric type wd_id)
    deriving (Bits, FShow);

    typedef struct {
        Bit #(wd_id) wid;
        Bit #(wd_data) wdata;
        Bit #(TDiv #(wd_data, 8)) wstrb;
        Bool wlast;
    } AXI3_Wr_Data #(numeric type wd_data, numeric type wd_id)
    deriving (Bits, FShow);

    typedef struct {
        Bit #(wd_id) bid;
        Bit #(2) bresp;
    } AXI3_Wr_Resp #(numeric type wd_id)
    deriving (Bits, FShow);

    typedef struct {
        Bit #(wd_id) arid;
        Bit #(wd_addr) araddr;
        Bit #(4) arlen;
        Bit #(3) arsize;
        Bit #(2) arburst;
        Bit #(2) arlock;
        Bit #(4) arcache;
        Bit #(3) arprot;
    } AXI3_Rd_Addr #(numeric type wd_addr, numeric type wd_id)
    deriving (Bits, FShow);

    typedef struct {
        Bit #(wd_id) rid;
        Bit #(wd_data) rdata;
        Bit #(2) rresp;
        Bool rlast;
    } AXI3_Rd_Data #(numeric type wd_data, numeric type wd_id)
    deriving (Bits, FShow);

    interface AXI3_Master_Xactor_IFC #(numeric type wd_addr, numeric type wd_data, numeric type wd_id);
    
        interface AXI3_Master_IFC #(wd_addr, wd_data, wd_id) axi_side;

        interface FIFOF_I #(AXI3_Wr_Addr #(wd_addr, wd_id)) wr_addr;
        interface FIFOF_I #(AXI3_Wr_Data #(wd_data, wd_id)) wr_data;
        interface FIFOF_O #(AXI3_Wr_Resp #(wd_id)) wr_resp;

        interface FIFOF_I #(AXI3_Rd_Addr #(wd_addr, wd_id)) rd_addr;
        interface FIFOF_O #(AXI3_Rd_Data #(wd_data, wd_id)) rd_data;
    
    endinterface

    module mkAXI3_Master_Xactor (AXI3_Master_Xactor_IFC #(wd_addr, wd_data, wd_id));

        Bool unguarded = True;
        Bool guarded = False;

        FIFOF #(AXI3_Wr_Addr #(wd_addr, wd_id)) f_wr_addr <- mkGFIFOF (False, True);
        FIFOF #(AXI3_Wr_Data #(wd_data, wd_id)) f_wr_data <- mkGFIFOF (False, True);
        FIFOF #(AXI3_Wr_Resp #(wd_id)) f_wr_resp <- mkGFIFOF (True, False);
        FIFOF #(AXI3_Rd_Addr #(wd_addr, wd_id)) f_rd_addr <- mkGFIFOF (False, True);
        FIFOF #(AXI3_Rd_Data #(wd_data, wd_id)) f_rd_data <- mkGFIFOF (True, False);

        interface axi_side = interface AXI3_Master_IFC;
            method m_awvalid = f_wr_addr.notEmpty;
            method Bit #(wd_id) m_awid = f_wr_addr.first.awid;
            method Bit #(wd_addr) m_awaddr = f_wr_addr.first.awaddr;
            method Bit #(4) m_awlen = f_wr_addr.first.awlen;
            method Bit #(2) m_awburst = f_wr_addr.first.awburst;
            method Bit #(3) m_awsize = f_wr_addr.first.awsize;
            method Bit #(4) m_awcache = f_wr_addr.first.awcache;
            method Bit #(3) m_awprot = f_wr_addr.first.awprot;
            method Bit #(2) m_awlock = f_wr_addr.first.awlock;
            method Action m_awready(Bool awready);
                if(f_wr_addr.notEmpty && awready)
                    f_wr_addr.deq();
            endmethod

            method m_wvalid = f_wr_data.notEmpty;
            method Bit #(wd_id) m_wid = f_wr_data.first.wid;
            method Bit #(wd_data) m_wdata = f_wr_data.first.wdata;
            method Bit #(TDiv #(wd_data, 8)) m_wstrb = f_wr_data.first.wstrb;
            method Bool m_wlast = f_wr_data.first.wlast;
            method Action m_wready(Bool wready);
                if(f_wr_data.notEmpty && wready)
                    f_wr_data.deq();
            endmethod

            method m_bready = f_wr_resp.notFull;
            method Action m_bvalid(Bool bvalid, Bit #(wd_id) bid, Bit #(2) bresp);
                if(f_wr_resp.notFull && bvalid)
                    f_wr_resp.enq(AXI3_Wr_Resp {bid: bid, bresp: bresp});
            endmethod

            method m_arvalid = f_rd_addr.notEmpty;
            method Bit #(wd_id) m_arid = f_rd_addr.first.arid;
            method Bit #(wd_addr) m_araddr = f_rd_addr.first.araddr;
            method Bit #(4) m_arlen = f_rd_addr.first.arlen;
            method Bit #(2) m_arburst = f_rd_addr.first.arburst;
            method Bit #(3) m_arsize = f_rd_addr.first.arsize;
            method Bit #(4) m_arcache = f_rd_addr.first.arcache;
            method Bit #(3) m_arprot = f_rd_addr.first.arprot;
            method Bit #(2) m_arlock = f_rd_addr.first.arlock;
            method Action m_arready(Bool arready);
                if(f_rd_addr.notEmpty && arready)
                    f_rd_addr.deq();
            endmethod

            method m_rready = f_rd_data.notFull;
            method Action m_rvalid(Bool rvalid, Bit #(wd_id) rid, Bit #(2) rresp, Bit #(wd_data) rdata, Bool rlast);
                if(f_rd_data.notFull && rvalid)
                    f_rd_data.enq(AXI3_Rd_Data {rid: rid, rresp: rresp, rdata: rdata, rlast: rlast});
            endmethod
        endinterface;

        interface wr_addr = to_FIFOF_I(f_wr_addr);
        interface wr_data = to_FIFOF_I(f_wr_data);
        interface wr_resp = to_FIFOF_O(f_wr_resp);
        interface rd_addr = to_FIFOF_I(f_rd_addr);
        interface rd_data = to_FIFOF_O(f_rd_data);

    endmodule

    interface AXI3_Slave_Xactor_IFC #(numeric type wd_addr, numeric type wd_data, numeric type wd_id);
    
        interface AXI3_Slave_IFC #(wd_addr, wd_data, wd_id) axi_side;

        interface FIFOF_O #(AXI3_Wr_Addr #(wd_addr, wd_id)) wr_addr;
        interface FIFOF_O #(AXI3_Wr_Data #(wd_data, wd_id)) wr_data;
        interface FIFOF_I #(AXI3_Wr_Resp #(wd_id)) wr_resp;

        interface FIFOF_O #(AXI3_Rd_Addr #(wd_addr, wd_id)) rd_addr;
        interface FIFOF_I #(AXI3_Rd_Data #(wd_data, wd_id)) rd_data;
    
    endinterface

    module mkAXI3_Slave_Xactor (AXI3_Slave_Xactor_IFC #(wd_addr, wd_data, wd_id));

        Bool unguarded = True;
        Bool guarded = False;

        FIFOF #(AXI3_Wr_Addr #(wd_addr, wd_id)) f_wr_addr <- mkGFIFOF (True, False);
        FIFOF #(AXI3_Wr_Data #(wd_data, wd_id)) f_wr_data <- mkGFIFOF (True, False);
        FIFOF #(AXI3_Wr_Resp #(wd_id)) f_wr_resp <- mkGFIFOF (False, True);
        FIFOF #(AXI3_Rd_Addr #(wd_addr, wd_id)) f_rd_addr <- mkGFIFOF (True, False);
        FIFOF #(AXI3_Rd_Data #(wd_data, wd_id)) f_rd_data <- mkGFIFOF (False, True);

        interface axi_side = interface AXI3_Slave_IFC;
            method Action m_awvalid (
                Bool awvalid,
                Bit #(wd_id) awid,
                Bit #(wd_addr) awaddr,
                Bit #(4) awlen,
                Bit #(3) awsize,
                Bit #(2) awburst,
                Bit #(4) awcache,
                Bit #(3) awprot,
                Bit #(2) awlock);
                if(f_wr_addr.notFull && awvalid)
                    f_wr_addr.enq(AXI3_Wr_Addr {
                        awid: awid,
                        awaddr: awaddr,
                        awlen: awlen,
                        awsize: awsize,
                        awburst: awburst,
                        awcache: awcache,
                        awprot: awprot,
                        awlock: awlock
                    });
            endmethod
            method m_awready = f_wr_addr.notFull;

            method Action m_wvalid (
                Bool wvalid,
                Bit #(wd_id) wid,
                Bit #(wd_data) wdata,
                Bit #(TDiv #(wd_data, 8)) wstrb,
                Bool wlast);
                if(f_wr_data.notFull && wvalid)
                    f_wr_data.enq(AXI3_Wr_Data {
                        wid: wid,
                        wdata: wdata,
                        wstrb: wstrb,
                        wlast: wlast
                    });
            endmethod
            method m_wready = f_wr_data.notFull;

            method m_bvalid = f_wr_resp.notEmpty;
            method m_bid = f_wr_resp.first.bid;
            method m_bresp = f_wr_resp.first.bresp;
            method Action m_bready(Bool bready);
                if(f_wr_resp.notEmpty && bready)
                    f_wr_resp.deq;
            endmethod

            method Action m_arvalid (
                Bool arvalid,
                Bit #(wd_id) arid,
                Bit #(wd_addr) araddr,
                Bit #(4) arlen,
                Bit #(3) arsize,
                Bit #(2) arburst,
                Bit #(4) arcache,
                Bit #(3) arprot,
                Bit #(2) arlock);
                if(f_rd_addr.notFull && arvalid)
                    f_rd_addr.enq(AXI3_Rd_Addr {
                        arid: arid,
                        araddr: araddr,
                        arlen: arlen,
                        arsize: arsize,
                        arburst: arburst,
                        arcache: arcache,
                        arprot: arprot,
                        arlock: arlock
                    });
            endmethod
            method m_arready = f_rd_addr.notFull;

            method m_rvalid = f_rd_data.notEmpty;
            method m_rid = f_rd_data.first.rid;
            method m_rresp = f_rd_data.first.rresp;
            method m_rdata = f_rd_data.first.rdata;
            method m_rlast = f_rd_data.first.rlast;
            method Action m_rready(Bool rready);
                if(f_rd_data.notEmpty && rready)
                    f_rd_data.deq;
            endmethod
        endinterface;

        interface wr_addr = to_FIFOF_O(f_wr_addr);
        interface wr_data = to_FIFOF_O(f_wr_data);
        interface wr_resp = to_FIFOF_I(f_wr_resp);
        interface rd_addr = to_FIFOF_O(f_rd_addr);
        interface rd_data = to_FIFOF_I(f_rd_data);

    endmodule

    function Bit #(8) axi3_decode_size(Bit #(3) size);
        case(size)
        0: return 1;
        1: return 2;
        2: return 4;
        3: return 8;
        4: return 16;
        5: return 32;
        6: return 64;
        7: return 128;
        endcase
    endfunction

    function Bit #(wd_addr) axi3_next_address(Bit #(wd_addr) addr, Bit #(3) size, Bit #(2) burst, Bit #(4) len)
        provisos(Add#(ignore, 4, wd_addr), Add#(ignore2, 8, wd_addr));
        case(burst)
        2'b00, 2'b11: return addr;
        2'b01: return addr + extend(axi3_decode_size(size));
        2'b10: begin
            let a = addr + extend(axi3_decode_size(size));
            let m = extend(len)<<size;
            return (a & m) | (addr & ~m);
        end
        endcase
    endfunction

endpackage