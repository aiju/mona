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
    CAddr cfg_status_clear_busy = CAddr { a: 12'h000, o: 3 };

    CAddr cfg_control = full(12'h004);
    CAddr cfg_control_start = CAddr { a: 12'h004, o: 0 };
    CAddr cfg_control_flush = CAddr { a: 12'h004, o: 1 };
    CAddr cfg_control_invalidate_depth = CAddr { a: 12'h004, o: 2 };
    CAddr cfg_control_clear = CAddr { a: 12'h004, o: 3 };
    CAddr cfg_control_len = CAddr { a: 12'h004, o: 16 };

    CAddr cfg_display_framebuffer = full(12'h008);
    CAddr cfg_render_target = full(12'h00C);
    CAddr cfg_depth_buffer = full(12'h010);

    CAddr cfg_depth_mode = CAddr { a: 12'h014, o: 0 };
    typedef enum {
        DEPTH_MODE_ALWAYS = 3'b000,
        DEPTH_MODE_NEVER = 3'b001,
        DEPTH_MODE_LT = 3'b010,
        DEPTH_MODE_LE = 3'b011,
        DEPTH_MODE_GT = 3'b100,
        DEPTH_MODE_GE = 3'b101,
        DEPTH_MODE_EQ = 3'b110,
        DEPTH_MODE_NE = 3'b111
    } DepthMode
    deriving (Bits, Eq, FShow);

    CAddr cfg_clear_addr = full(12'h018);
    CAddr cfg_clear_stride = full(12'h01C); // 16 bits
    CAddr cfg_clear_width = CAddr { a : 12'h020, o: 0 }; // 16 bits
    CAddr cfg_clear_height = CAddr { a : 12'h020, o: 16 }; // 16 bits
    CAddr cfg_clear_data = full(12'h024);

    CAddr cfg_texture_en = CAddr {a: 12'h028, o: 0};
    CAddr cfg_texture_wrap_mode = CAddr {a: 12'h028, o: 1};
    CAddr cfg_texture_width = CAddr {a: 12'h028, o: 4};
    CAddr cfg_texture_height = CAddr {a: 12'h028, o: 8};
    CAddr cfg_texture_stride = CAddr {a: 12'h028, o: 12};
    CAddr cfg_texture_addr = full(12'h02C);
    CAddr cfg_texture_border = full(12'h034);
    typedef enum {
        WRAP_MODE_WRAP = 2'b00,
        WRAP_MODE_CLAMP_TO_EDGE = 2'b01,
        WRAP_MODE_CLAMP_TO_BORDER = 2'b10
    } TexWrapMode
    deriving (Bits, Eq, FShow);

    CAddr cfg_text_en = CAddr {a: 12'h080, o: 0};
    CAddr cfg_text_access = CAddr {a: 12'h084, o: 0};
    CAddr cfg_text_transparent = CAddr {a: 12'h088, o: 0};

    CAddr cfg_stats_running = CAddr { a: 12'h030, o: 0 };

    Bit #(12) cfg_stats_starter = 12'h800;
    Bit #(12) cfg_stats_coarse = 12'h810;
    Bit #(12) cfg_stats_fine = 12'h820;
    Bit #(12) cfg_stats_depth = 12'h830;
    Bit #(12) cfg_stats_pixel = 12'h840;
    Bit #(12) cfg_stats_uv = 12'h850;
    Bit #(12) cfg_stats_texture = 12'h860;

endpackage