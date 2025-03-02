package UVInterp;

    import Semi_FIFOF :: *;
    import FIFOF :: *;
    import Reciprocal :: *;
    import Vector :: *;
    import Util :: *;

    typedef union tagged {
        void Flush;
        struct {
            UInt #(11) x;
            UInt #(11) y;
            Vector #(3, Int #(27)) edge_vec;
            Vector #(3, Vector #(2, Int #(27))) uv;
        } Pixel;
    } UVInterpIn
    deriving (Bits, FShow);

    typedef union tagged {
        void Flush;
        struct {
            UInt #(11) x;
            UInt #(11) y;
            Int #(27) u;
            Int #(27) v;
        } Pixel;
    } UVInterpOut
    deriving (Bits, FShow);

    interface UVInterp;
        interface FIFOF_I #(UVInterpIn) in;
        interface FIFOF_O #(UVInterpOut) out;
    endinterface

    (* synthesize *)
    module mkUVInterp(UVInterp);
        FIFOF #(UVInterpIn) f_in <- mkFIFOF;
        FIFOF #(UVInterpOut) f_out <- mkFIFOF;

        Reciprocal #(27) reciprocal <- mkReciprocal;
        FIFOF #(Tuple5 #(Bool, UInt #(11), UInt #(11), Vector #(3, Int #(27)), Vector #(3, Int #(27)))) s0 <- mkFIFOF;
        FIFOF #(Tuple5 #(Bool, UInt #(11), UInt #(11), Int #(27), Int #(27))) s1 <- mkFIFOF;

        rule rl_s0;
            let d <- pop(f_in);
            let e = d.Pixel.edge_vec;
            let s = e[0] + e[1] + e[2];
            reciprocal.in.enq(pack(s));
            Vector #(3, Int #(27)) u = newVector;
            Vector #(3, Int #(27)) v = newVector;
            for(Integer i = 0; i < 3; i = i + 1) begin
                Int #(54) uu = extend(e[i]) * extend(d.Pixel.uv[i][0]);
                u[i] = truncate(uu >> 27);
                Int #(54) vv = extend(e[i]) * extend(d.Pixel.uv[i][1]);
                v[i] = truncate(vv >> 27);
            end
            let flush = d matches tagged Flush ? True : False;
            s0.enq(tuple5(flush, d.Pixel.x, d.Pixel.y, u, v));
        endrule

        rule rl_s1;
            match {.flush, .x, .y, .u, .v} <- pop(s0);
            let uu = u[0] + u[1] + u[2];
            let vv = v[0] + v[1] + v[2];
            s1.enq(tuple5(flush, x, y, uu, vv));
        endrule

        rule rl_s2;
            match {.flush, .x, .y, .u, .v} <- pop(s1);
            let w <- pop_o(reciprocal.out);
            Int #(54) u0 = extend(u) * unpack(zeroExtend(w.value));
            Int #(54) v0 = extend(v) * unpack(zeroExtend(w.value));
            Int #(27) u1 = truncate(u0 >> 26 - w.shift);
            Int #(27) v1 = truncate(v0 >> 26 - w.shift);
            if(flush)
                f_out.enq(tagged Flush);
            else
                f_out.enq(tagged Pixel {x: x, y: y, u: u1, v: v1});
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
    endmodule

endpackage