package ConfigDefs;
    import CBus :: *;

    typedef 12 ConfigBusAddrWidth;
    typedef 32 ConfigBusDataWidth;

    typedef CBus #(ConfigBusAddrWidth, ConfigBusDataWidth) ConfigBus;
    typedef CRAddr #(ConfigBusAddrWidth, ConfigBusDataWidth) CAddr;
    typedef ModWithCBus #(ConfigBusAddrWidth, ConfigBusDataWidth, i) ModWithConfig #(type i);

    function CAddr full(Bit #(ConfigBusAddrWidth) value);
        return CAddr { a: value, o: 0 };
    endfunction

    CAddr cfg_status = full(12'h000);
    CAddr cfg_status_vsync = CAddr { a: 12'h000, o: 0 };
    CAddr cfg_status_in_vsync = CAddr { a: 12'h000, o: 1 };
    CAddr cfg_status_flushed = CAddr { a: 12'h000, o: 2 };

    CAddr cfg_control = full(12'h004);
    CAddr cfg_control_start = CAddr { a: 12'h004, o: 0 };
    CAddr cfg_control_flush = CAddr { a: 12'h004, o: 1 };
    CAddr cfg_control_invalidate_depth = CAddr { a: 12'h004, o: 2 };
    CAddr cfg_control_len = CAddr { a: 12'h004, o: 16 };

    CAddr cfg_display_framebuffer = full(12'h008);
    CAddr cfg_render_target = full(12'h00C);
    CAddr cfg_depth_buffer = full(12'h010);

endpackage