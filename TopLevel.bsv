package TopLevel;

    import I2C :: *;
    import Video :: *;
    import Connectable::*;
    import StmtFSM::*;
    import AXI :: *;
    import DMA :: *;
    import HdmiCtrl :: *;
    import CBus :: *;
    import ConfigDefs :: *;
    import AxiToSimple :: *;
    import Semi_FIFOF::*;

interface TopLevel;
    (* always_ready *) method Bit #(8) led;
    (* always_ready, always_enabled, prefix="" *) method Action hdmi_int(Bool hdmi_int);
    interface ExtI2C ext_i2c;
    (*prefix="video"*) interface Ext_Video ext_video;
    interface AXI3_Master_IFC #(32, 32, 8) fpga_to_hps;
endinterface

interface TopLevelWithAxiSlave;
    (*prefix=""*) interface TopLevel top_level;
    interface AXI3_Slave_IFC #(32, 32, 12) hps_to_fpga_lw;
endinterface

module [ModWithConfig] mkInternals(TopLevel);
    HdmiCtrl hdmi_ctrl <- mkHdmiCtrl;
    Video video <- mkVideo;
    DMA #(2, 1) dma <- mkDMA;
    LedDriver led_driver <- mkLedDriver;

    //mkConnection(video.dma_req, dma.rd_req);
    //mkConnection(dma.rd_data, video.dma_resp);

    Reg #(Bool) dma_start <- mkCBRegRW(CRAddr { a: 12'h4, o : 0}, False);
    Reg #(Bit #(1)) got_resp <- mkCBRegR(CRAddr { a: 12'hC, o : 0}, 0);

    rule rl_start;
        if(dma_start) begin
            dma.rd_req[0].enq(DMA_Req { addr: 32'h1000_0000, len: 4096 });
            dma.wr_req[0].enq(DMA_Req { addr: 32'h1000_0000, len: 4096 });
            dma_start <= False;
        end
    endrule

    rule rl_data;
        let d = dma.rd_data[0].first;
        dma.rd_data[0].deq;
        dma.wr_data[0].enq(d * 3);
    endrule

    rule rl_resp;
        dma.wr_resp[0].deq;
        got_resp <= 1;
    endrule

    interface ExtI2C ext_i2c = hdmi_ctrl.ext_i2c;
    interface Video ext_video = video.ext;
    interface AXI3_Master_IFC fpga_to_hps = dma.mem_ifc;
    method hdmi_int = hdmi_ctrl.hdmi_int;

    /*method Bit #(8) led;
        return {1'b1, 6'b0, hdmi_ctrl.hdmi_active ? 1'b1 : 1'b0};
    endmethod*/

    method led = led_driver.led;
endmodule

(* synthesize *)
module mkTopLevel(TopLevelWithAxiSlave);
    AxiToSimple reg_if <- mkAxiToSimple;
    IWithCBus #(ConfigBus, TopLevel) top <- exposeCBusIFC(mkInternals);
    mkConnection(reg_if.simple, top.cbus_ifc);

    interface top_level = top.device_ifc;
    interface hps_to_fpga_lw = reg_if.axi;

endmodule

interface LedDriver;
    method Bit #(8) led;
endinterface
module [ModWithConfig] mkLedDriver(LedDriver);
    Reg#(Bit#(8)) led_reg <- mkCBRegRW(cfg_led, 0);
    method Bit #(8) led = led_reg;
endmodule

endpackage