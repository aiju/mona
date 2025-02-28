package CoarseRaster;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;

    typedef struct {
        Int #(27) x;
        Int #(27) y;
        Int #(27) a;
    } EdgeFn
    deriving (Bits, FShow);

    typedef union tagged {
        void Flush;
        struct {
            Vector #(3, EdgeFn) edge_fns;
            Vector #(3, Vector #(2, Int #(27))) uv;
            UInt #(9) min_x;
            UInt #(9) max_x;
            UInt #(9) min_y;
            UInt #(9) max_y;
        } Triangle;
    } CoarseRasterIn
    deriving (Bits, FShow);

    typedef union tagged {
        void Flush;
        struct {
            Vector #(3, EdgeFn) edge_fns;
            Vector #(3, Vector #(2, Int #(27))) uv;
            UInt #(9) tx;
            UInt #(9) ty;
        } Tile;
    } CoarseRasterOut
    deriving (Bits, FShow);

    interface CoarseRaster;
        interface FIFOF_I #(CoarseRasterIn) in;
        interface FIFOF_O #(CoarseRasterOut) out;
    endinterface

    (* synthesize *)
    module mkCoarseRaster(CoarseRaster);
        FIFOF #(CoarseRasterIn) f_in <- mkFIFOF;
        FIFOF #(CoarseRasterOut) f_out <- mkFIFOF;

        Reg #(Bool) active <- mkReg (False);
        Reg #(Vector #(3, EdgeFn)) edge_fns <- mkRegU;
        Reg #(Vector #(3, Int #(27))) edge_fns_left <- mkRegU;
        Reg #(UInt #(9)) min_x <- mkRegU;
        Reg #(UInt #(9)) max_x <- mkRegU;
        Reg #(UInt #(9)) min_y <- mkRegU;
        Reg #(UInt #(9)) max_y <- mkRegU;
        Reg #(UInt #(9)) tx <- mkRegU;
        Reg #(UInt #(9)) ty <- mkRegU;
        Reg #(Vector #(3, Vector #(2, Int #(27)))) uv <- mkRegU;

        function Int #(27) edge_fn(Integer i, Integer corner)
            = edge_fns[i].a
            + edge_fns[i].x * (corner == 1 || corner == 3 ? 3 : 0)
            + edge_fns[i].y * (corner == 2 || corner == 3 ? 3 : 0);
        function Bool viable;
            Bool can_pos = True;
            for(Integer i = 0; i < 3; i = i + 1) begin
                Bool any_pos = False;
                for(Integer c = 0; c < 4; c = c + 1) begin
                    any_pos = any_pos || edge_fn(i, c) >= 0;
                end
                can_pos = can_pos && any_pos;
            end
            return can_pos;
        endfunction

        rule rl_start (!active);
            f_in.deq;
            case(f_in.first) matches
                tagged Flush : begin
                    f_out.enq(tagged Flush);
                end
                tagged Triangle .p : begin
                    active <= True;
                    edge_fns <= p.edge_fns;
                    Vector #(3, Int #(27)) new_edge_fns_left = newVector;
                    for(Integer i = 0; i < 3; i = i + 1)
                        new_edge_fns_left[i] = p.edge_fns[i].a;
                    edge_fns_left <= new_edge_fns_left;
                    min_x <= p.min_x;
                    max_x <= p.max_x;
                    min_y <= p.min_y;
                    max_y <= p.max_y;
                    tx <= p.min_x;
                    ty <= p.min_y;
                    uv <= p.uv;
                end
            endcase
        endrule

        rule rl_step (active);
            if(viable) begin
                f_out.enq(tagged Tile {
                    edge_fns: edge_fns,
                    tx: tx,
                    ty: ty,
                    uv: uv
                });
            end
            if(tx == max_x) begin
                if(ty == max_y)
                    active <= False;
                tx <= min_x;
                ty <= ty + 1;
                Vector #(3, EdgeFn) new_edge_fns = edge_fns;
                Vector #(3, Int #(27)) new_edge_fns_left = newVector;
                for(Integer i = 0; i < 3; i = i + 1) begin
                    new_edge_fns[i].a = edge_fns_left[i] + edge_fns[i].y * 4;
                    new_edge_fns_left[i] = edge_fns_left[i] + edge_fns[i].y * 4;
                end
                edge_fns <= new_edge_fns;
                edge_fns_left <= new_edge_fns_left;
            end else begin
                tx <= tx + 1;
                Vector #(3, EdgeFn) new_edge_fns = edge_fns;
                for(Integer i = 0; i < 3; i = i + 1)
                    new_edge_fns[i].a = edge_fns[i].a + edge_fns[i].x * 4;
                edge_fns <= new_edge_fns;
            end
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
    endmodule

endpackage