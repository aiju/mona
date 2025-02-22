package TopLevel;

    import I2C :: *;
    import Video :: *;
    import Connectable::*;
    import StmtFSM::*;
    import AXI :: *;
    import DMA :: *;
    import HdmiCtrl :: *;

interface TopLevel;
    (* always_ready *) method Bit #(8) led;
    (* always_ready, always_enabled, prefix="" *) method Action hdmi_int(Bool hdmi_int);
    interface ExtI2C ext_i2c;
    (*prefix="video"*) interface Ext_Video ext_video;
    interface AXI3_Master_IFC #(32, 32, 8) fpga_to_hps;
endinterface

(* synthesize *)
module mkTopLevel(TopLevel);
    HdmiCtrl hdmi_ctrl <- mkHdmiCtrl;
    Video video <- mkVideo;
    DMA dma <- mkDMA;

    mkConnection(video.dma_req, dma.req);
    mkConnection(dma.resp, video.dma_resp);

    interface ExtI2C ext_i2c = hdmi_ctrl.ext_i2c;
    interface Video ext_video = video.ext;
    interface AXI3_Master_IFC fpga_to_hps = dma.mem_ifc;
    method hdmi_int = hdmi_ctrl.hdmi_int;

    method Bit #(8) led;
        return {1'b1, 6'b0, hdmi_ctrl.hdmi_active ? 1'b1 : 1'b0};
    endmethod

endmodule

endpackage