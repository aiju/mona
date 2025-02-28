package FineRaster;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import CoarseRaster :: *;

    typedef union tagged {
        void Flush;
        struct {
            UInt #(9) tx;
            UInt #(9) ty;
            Bit #(16) pixels;
            Vector #(3, EdgeFn) edge_fns;
            Vector #(3, Vector #(2, Int #(27))) uv;
        } Tile;
    } FineRasterOut
    deriving (Bits, FShow);

    interface FineRaster;
        interface FIFOF_I #(CoarseRasterOut) in;
        interface FIFOF_O #(FineRasterOut) out;
    endinterface

    (* synthesize *)
    module mkFineRaster(FineRaster);
        FIFOF #(CoarseRasterOut) f_in <- mkFIFOF;
        FIFOF #(FineRasterOut) f_out <- mkFIFOF;

        function Int #(27) edge_fn(Vector #(3, EdgeFn) edge_fns, Integer pixel, Integer i)
            = edge_fns[i].a
            + edge_fns[i].x * fromInteger(pixel % 4)
            + edge_fns[i].y * fromInteger(pixel / 4);
        function Bool check_inside(Vector #(3, EdgeFn) e, Integer pixel)
            = edge_fn(e, pixel, 0) >= 0
            && edge_fn(e, pixel, 1) >= 0
            && edge_fn(e, pixel, 2) >= 0;

        rule rl_step;
            f_in.deq;
            case(f_in.first) matches
                tagged Flush : f_out.enq(tagged Flush);
                tagged Tile .p : begin
                    Bit #(16) pixels = pack(genWith(check_inside(p.edge_fns)));
                    if(pixels != 0)
                        f_out.enq(tagged Tile {
                            tx: p.tx,
                            ty: p.ty,
                            pixels: pixels,
                            edge_fns: p.edge_fns,
                            uv: p.uv
                        });
                end
            endcase
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
    endmodule

endpackage