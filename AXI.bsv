package AXI;

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

        (* always_ready, always_enabled, prefix="" *) method Action m_awready (
            (*port="awready"*) Bool awready);
    
        (* always_ready, result="wvalid" *) method Bool m_wvalid;
        (* always_ready, result="wid" *) method Bit #(wd_id) m_wid;
        (* always_ready, result="wdata" *) method Bit #(wd_data) m_wdata;
        (* always_ready, result="wstrb" *) method Bit #(TDiv #(wd_data, 8)) m_wstrb;
        (* always_ready, result="wlast" *) method Bit #(1) m_wlast;

        (* always_ready, always_enabled, prefix="" *) method Action m_wready (
            (*port="wready"*) Bool awready);

        (* always_ready, result="bready" *) method Bool m_bready;
        (* always_ready, always_enabled, prefix="" *) method Action m_bvalid (
            (*port="bvalid"*) Bool bvalid,
            (*port="bid"*) Bit #(wd_id) m_bid,
            (*port="bresp"*) Bit #(2) m_bresp);

        (* always_ready, result="arvalid" *) method Bool m_arvalid;
        (* always_ready, result="arid" *) method Bit #(wd_id) m_arid;
        (* always_ready, result="araddr" *) method Bit #(wd_addr) m_araddr;
        (* always_ready, result="arlen" *) method Bit #(4) m_arlen;
        (* always_ready, result="arsize" *) method Bit #(3) m_arsize;
        (* always_ready, result="arburst" *) method Bit #(2) m_arburst;
        (* always_ready, result="arcache" *) method Bit #(4) m_arcache;
        (* always_ready, result="arprot" *) method Bit #(3) m_arprot;

        (* always_ready, always_enabled, prefix="" *) method Action m_arready (
            (*port="arready"*) Bool arready);

        (* always_ready, result="rready" *) method Bool m_rready;
        (* always_ready, always_enabled, prefix="" *) method Action m_rvalid (
            (*port="rvalid"*) Bool rvalid,
            (*port="rid"*) Bit #(wd_id) m_rid,
            (*port="rresp"*) Bit #(2) m_rresp,
            (*port="rdata"*) Bit #(wd_data) m_rdata,
            (*port="rlast"*) Bit #(1) m_rlast);

    endinterface

endpackage