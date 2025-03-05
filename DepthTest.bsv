package DepthTest;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import FineRaster :: *;
    import DMA :: *;
    import ConfigDefs :: *;
    import Util :: *;
    import RegFile :: *;
    import Vector :: *;
    import CoarseRaster :: *;
    import CBus :: *;
    import SpecialFIFOs::*;
    `include "Util.defines"

    typedef 128 WdLineSize;
    typedef 8 WdCacheAddr;
    typedef Bit #(WdCacheAddr) CacheAddr;
    typedef Bit #(WdLineSize) Line;
    typedef 10 WdTag;
    typedef Bit #(WdTag) Tag;

    interface DepthTest;
        interface FIFOF_I #(FineRasterOut) in;
        interface FIFOF_O #(FineRasterOut) out;
        interface FIFOF_O #(DMA_Req) rd_req;
        interface FIFOF_I #(Bit #(128)) rd_data;
        interface FIFOF_O #(DMA_Req) wr_req;
        interface FIFOF_O #(Bit #(128)) wr_data;
        interface FIFOF_I #(Bool) wr_resp;
    endinterface

    typedef struct {
        FineRasterOut tile;
        Vector #(16, Int #(16)) z;
        Bool need0;
        Bool need1;
        Bool yeet0;
        Bool yeet1;
        Tag old0;
        Tag old1;
    } Cache_Req
    deriving (Bits, FShow);

    `SynthBoundary(mkDepthTest, mkDepthTestInternal, DepthTest)

    module [ModWithConfig] mkDepthTestInternal(DepthTest);
        let f_in <- mkPipelineFIFOF;
        FIFOF #(FineRasterOut) f_out <- mkSizedFIFOF(256);
        let f_rd_req <- mkBypassFIFOF;
        let f_rd_data <- mkPipelineFIFOF;
        let f_wr_req <- mkBypassFIFOF;
        let f_wr_data <- mkBypassFIFOF;
        let f_wr_resp <- mkPipelineFIFOF;

        Reg #(Bit #(32)) depth_buffer <- mkCBRegRW(cfg_depth_buffer, 32'h1080_0000);
        Reg #(DepthMode) depth_mode <- mkCBRegRW(cfg_depth_mode, DEPTH_MODE_GT);

        RegFile #(CacheAddr, Tag) tag0 <- mkRegFileFull;
        RegFile #(CacheAddr, Tag) tag1 <- mkRegFileFull;
        RegFile #(CacheAddr, Line) data0 <- mkRegFileWCF(0, ~0);
        RegFile #(CacheAddr, Line) data1 <- mkRegFileWCF(0, ~0);

        Tag invalid_tag = ~0;

        function CacheAddr cache_addr(UInt #(9) tx, UInt #(9) ty)
            = {pack(ty)[3:0], pack(tx)[3:0]};
        function Bit #(32) mem_addr(CacheAddr ca, Tag tag, Bool lower_half)
            = depth_buffer
            + (extend(tag) * 256 + extend(ca)) * 32
            + (lower_half ? 16 : 0);
        function Tag valid_tag(UInt #(9) tx, UInt #(9) ty)
            = {pack(ty)[8:4], pack(tx)[8:4]};
        function Bool is_need(UInt #(9) tx, UInt #(9) ty, Tag tag)
            = tag != valid_tag(tx, ty);
        function Bool is_yeet(UInt #(9) tx, UInt #(9) ty, Tag tag)
            = tag != invalid_tag && tag != valid_tag(tx, ty);

        FIFOF #(Cache_Req) s0 <- mkPipelineFIFOF;
        FIFOF #(Cache_Req) s1 <- mkPipelineFIFOF;
        FIFOF #(Cache_Req) s2 <- mkPipelineFIFOF;
        FIFOF #(Cache_Req) s3 <- mkPipelineFIFOF;

        Reg #(Bool) issue_invalidate <- mkCBRegRW(cfg_control_invalidate_depth, False);
        Reg #(Bool) invalidating <- mkReg (True);
        Reg #(CacheAddr) invalidating_addr <- mkReg (0);

        rule rl_start_invalidate (issue_invalidate && !invalidating);
            invalidating <= True;
            issue_invalidate <= False;
        endrule

        (* preempts="rl_invalidating, rl_s0" *)
        rule rl_invalidating (invalidating);
            tag0.upd(invalidating_addr, invalid_tag);
            tag1.upd(invalidating_addr, invalid_tag);
            invalidating_addr <= invalidating_addr + 1;
            if(invalidating_addr == ~0)
                invalidating <= False;
        endrule

        rule rl_s0;
            FineRasterOut req <- pop(f_in);
            case(req) matches
                tagged Flush: s0.enq(Cache_Req {
                    tile: req,
                    z: ?,
                    need0: False,
                    need1: False,
                    yeet0: False,
                    yeet1: False,
                    old0: ?,
                    old1: ?
                });
                tagged Tile .tile: begin
                    let ca = cache_addr(tile.tx, tile.ty);
                    let tag = valid_tag(tile.tx, tile.ty);
                    let t0 = tag0.sub(ca);
                    let t1 = tag1.sub(ca);
                    let need0 = tile.pixels[7:0] != 0 && is_need(tile.tx, tile.ty, t0);
                    let need1 = tile.pixels[15:8] != 0 && is_need(tile.tx, tile.ty, t1);
                    let yeet0 = tile.pixels[7:0] != 0 && is_yeet(tile.tx, tile.ty, t0);
                    let yeet1 = tile.pixels[15:8] != 0 && is_yeet(tile.tx, tile.ty, t1);
                    if(need0) tag0.upd(ca, tag);
                    if(need1) tag1.upd(ca, tag);
                    if(need0 || need1)
                        f_rd_req.enq(DMA_Req {
                            addr: mem_addr(ca, tag, !need0),
                            len: need0 && need1 ? 32 : 16
                        });
                    s0.enq(Cache_Req {
                        tile: req,
                        z: ?,
                        need0: need0,
                        need1: need1,
                        yeet0: yeet0,
                        yeet1: yeet1,
                        old0: t0,
                        old1: t1
                    });
                end
            endcase
        endrule

        rule rl_s1;
            Cache_Req req <- pop(s0);
            let e = req.tile.Tile.edge_fns;
            for(Integer y = 0; y < 4; y = y + 1)
                for(Integer x = 0; x < 4; x = x + 1) begin
                    Int #(27) z = 0;
                    for(Integer i = 0; i < 3; i = i + 1)
                        z = z + e[i].a + e[i].x * fromInteger(x) + e[i].y * fromInteger(y);
                    req.z[y * 4 + x] = truncate(z >> 11);
                end
            s1.enq(req);
        endrule

        function Action yeet(CacheAddr ca, Tag tag, Bool lower_half, Line line);
            action
                f_wr_req.enq(DMA_Req {
                    addr: mem_addr(ca, tag, lower_half),
                    len: 16
                });
                f_wr_data.enq(line);
            endaction
        endfunction

        function Bit #(16) get_pixels(Cache_Req req);
            case(req.tile) matches
                tagged Flush: return 0;
                tagged Tile .tile: return tile.pixels;
            endcase
        endfunction

        function FineRasterOut update_pixels(Cache_Req req, Bit #(16) pixels);
            case(req.tile) matches
                tagged Flush: return req.tile;
                tagged Tile .tile: return tagged Tile {
                    tx: tile.tx,
                    ty: tile.ty,
                    edge_fns: tile.edge_fns,
                    uv: tile.uv,
                    pixels: pixels
                };
            endcase
        endfunction

        function Bool depth_test(Int #(16) frag, Int #(16) buffer);
            let lt = frag < buffer;
            let eq = frag == buffer;
            case(depth_mode)
                DEPTH_MODE_ALWAYS: return True;
                DEPTH_MODE_NEVER: return False;
                DEPTH_MODE_LT: return lt;
                DEPTH_MODE_LE: return lt || eq;
                DEPTH_MODE_GT: return !lt && !eq;
                DEPTH_MODE_GE: return !lt;
                DEPTH_MODE_EQ: return eq;
                DEPTH_MODE_NE: return !eq;
            endcase
        endfunction

        function Tuple3 #(Line, Line, FineRasterOut) update_z(Line line0, Line line1, Cache_Req req);
            Bit #(16) pixels = get_pixels(req);
            Vector #(16, Int #(16)) z_vec = unpack({line1, line0});
            for(Integer i = 0; i < 16; i = i + 1) begin
                if(pixels[i] != 0 && depth_test(req.z[i], z_vec[i]))
                    z_vec[i] = req.z[i];
                else
                    pixels[i] = 0;
            end
            return tuple3(pack(z_vec)[127:0], pack(z_vec)[255:128], update_pixels(req, pixels));
        endfunction

        function Bool valid_output(FineRasterOut data);
            case(data) matches
                tagged Flush: return True;
                tagged Tile .tile: return tile.pixels != 0;
            endcase
        endfunction

        function Bool conflicts(Cache_Req req1, Maybe #(Cache_Req) req2);
            if(req2 matches tagged Valid .req)
                return get_pixels(req1) != 0 && get_pixels(req) != 0 &&
                    req1.tile.Tile.tx == req.tile.Tile.tx && req1.tile.Tile.ty == req.tile.Tile.ty;
            else
                return False;
        endfunction

        FIFOF #(Line) line0 <- mkPipelineFIFOF;
        FIFOF #(Line) line1 <- mkPipelineFIFOF;
        Reg #(Bool) second_word <- mkReg (False);

        RWire #(Cache_Req) interlock0 <- mkRWire;
        RWire #(Cache_Req) interlock1 <- mkRWire;

        rule rl_s2 (!conflicts(s1.first, interlock0.wget()) && !conflicts(s1.first, interlock1.wget()));
            let req = s1.first;
            let ca = cache_addr(req.tile.Tile.tx, req.tile.Tile.ty);
            let l0 = data0.sub(ca);
            let l1 = data1.sub(ca);
            if(!req.need0 && !req.need1) begin
                line0.enq(l0);
                line1.enq(l1);
                s2.enq(req);
                s1.deq;
            end else if(req.need0 && !req.need1) begin
                let fetched <- pop(f_rd_data);
                line0.enq(fetched);
                line1.enq(l1);
                if(req.yeet0) yeet(ca, req.old0, False, l0);
                s2.enq(req);
                s1.deq;
            end else if(!req.need0 && req.need1) begin
                let fetched <- pop(f_rd_data);
                line0.enq(l0);
                line1.enq(fetched);
                if(req.yeet1) yeet(ca, req.old1, True, l1);
                s2.enq(req);
                s1.deq;
            end else if(req.need0 && req.need1 && !second_word) begin
                let fetched <- pop(f_rd_data);
                line0.enq(fetched);
                if(req.yeet0) yeet(ca, req.old0, False, l0);
                second_word <= True;
            end else if(req.need0 && req.need1 && second_word) begin
                let fetched <- pop(f_rd_data);
                line1.enq(fetched);
                if(req.yeet1) yeet(ca, req.old1, True, l1);
                second_word <= False;
                s1.deq;
                s2.enq(req);
            end
        endrule

        FIFOF #(Line) line0_ <- mkPipelineFIFOF;
        FIFOF #(Line) line1_ <- mkPipelineFIFOF;
        FIFOF #(FineRasterOut) out_ <- mkPipelineFIFOF;

        rule rl_s3_interlock;
            interlock0.wset(s2.first);
        endrule

        rule rl_s3;
            let req <- pop(s2);
            let l0 <- pop(line0);
            let l1 <- pop(line1);
            match {.new0, .new1, .out} = update_z(l0, l1, req);
            s3.enq(req);
            line0_.enq(new0);
            line1_.enq(new1);
            out_.enq(out);
        endrule

        rule rl_s4_interlock;
            interlock1.wset(s3.first);
        endrule

        rule rl_s4;
            let req <- pop(s3);
            let ca = cache_addr(req.tile.Tile.tx, req.tile.Tile.ty);
            let new0 <- pop(line0_);
            let new1 <- pop(line1_);
            let out <- pop(out_);
            if(get_pixels(req)[7:0] != 0) data0.upd(ca, new0);
            if(get_pixels(req)[15:8] != 0) data1.upd(ca, new1);
            if(valid_output(out))
                f_out.enq(out);
        endrule

        // flush should probably wait for all responses to have arrived
        rule rl_wr_resp;
            f_wr_resp.deq;
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
        interface rd_req = to_FIFOF_O(f_rd_req);
        interface rd_data = to_FIFOF_I(f_rd_data);
        interface wr_req = to_FIFOF_O(f_wr_req);
        interface wr_data = to_FIFOF_O(f_wr_data);
        interface wr_resp = to_FIFOF_I(f_wr_resp);
    endmodule

    module mkDummyDepthTest(DepthTest);
        FIFOF #(FineRasterOut) data <- mkPipelineFIFOF;
        interface in = to_FIFOF_I(data);
        interface out = to_FIFOF_O(data);
        interface rd_req = dummy_FIFOF_O;
        interface rd_data = dummy_FIFOF_I;
        interface wr_req = dummy_FIFOF_O;
        interface wr_data = dummy_FIFOF_O;
        interface wr_resp = dummy_FIFOF_I;
    endmodule

endpackage