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

    CAddr cfg_led = full(12'h000);

endpackage