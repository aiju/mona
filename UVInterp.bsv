package UVInterp;

    import Semi_FIFOF :: *;
    import FIFOF :: *;
    import Reciprocal :: *;
    import Vector :: *;
    import Util :: *;
    import SpecialFIFOs::*;
    import Defs :: *;

    typedef union tagged {
        void Flush;
        struct {
            UInt #(11) x;
            UInt #(11) y;
            Vector #(3, Int #(27)) edge_vec;
            PerVertexData per_vertex_data;
        } Pixel;
    } UVInterpIn
    deriving (Bits, FShow);

    typedef union tagged {
        void Flush;
        struct {
            UInt #(11) x;
            UInt #(11) y;
            PerVertex per_vertex;
        } Pixel;
    } UVInterpOut
    deriving (Bits, FShow);

    typedef struct {
        UVInterpIn in;
        ReciprocalResult #(27) w;
        Vector #(3, Bit #(54)) e0;
        Vector #(3, Bit #(27)) e;
        Vector #(3, Vector #(2, Bit #(36))) u0;
        Vector #(2, Bit #(36)) u1;
        Vector #(2, Bit #(18)) u2;
        Vector #(3, Vector #(3, Bit #(18))) rgb0;
        Vector #(3, Bit #(18)) rgb1;
        Vector #(3, Bit #(8)) rgb2;
    } UVInterpInternal
    deriving (Bits, FShow);

    interface UVInterp;
        interface FIFOF_I #(UVInterpIn) in;
        interface FIFOF_O #(UVInterpOut) out;
    endinterface

    (* synthesize *)
    module mkUVInterp(UVInterp);
        FIFOF #(UVInterpIn) f_in <- mkPipelineFIFOF;
        FIFOF #(UVInterpOut) f_out <- mkSizedFIFOF(8);

        Reciprocal #(27) reciprocal <- mkReciprocal;

        FIFOF #(UVInterpIn) s0 <- mkSizedFIFOF(12);
        FIFOF #(UVInterpInternal) s1 <- mkPipelineFIFOF;
        FIFOF #(UVInterpInternal) s2 <- mkPipelineFIFOF;
        FIFOF #(UVInterpInternal) s3 <- mkPipelineFIFOF;
        FIFOF #(UVInterpInternal) s4 <- mkPipelineFIFOF;

        rule rl_s0;
            let d <- pop(f_in);
            let e = d.Pixel.edge_vec;
            let s = e[0] + e[1] + e[2];
            reciprocal.in.enq(pack(s));
            s0.enq(d);
        endrule

        rule rl_s1;
            let in <- pop(s0);
            let w <- pop_o(reciprocal.out);
            UVInterpInternal d = ?;
            d.in = in;
            d.w = w;
            for(Integer i = 0; i < 3; i = i + 1) begin
                d.e0[i] = extend(pack(d.in.Pixel.edge_vec[i])) * zeroExtend(w.value);
            end
            s1.enq(d);
        endrule

        rule rl_s2;
            UVInterpInternal d <- pop(s1);
            // TODO: can delete one of those multiplications, since the sum must be 1.0
            for(Integer i = 0; i < 3; i = i + 1)
                d.e[i] = truncate(d.e0[i] >> 26 - d.w.shift);
            s2.enq(d);
        endrule

        rule rl_s3;
            UVInterpInternal d <- pop(s2);
            for(Integer i = 0; i < 3; i = i + 1)
                for(Integer j = 0; j < 2; j = j + 1)
                    d.u0[i][j] = extend(d.in.Pixel.per_vertex_data[i].uv[j]) * zeroExtend(unpack(d.e[i][26:9]));
            for(Integer i = 0; i < 3; i = i + 1)
                for(Integer j = 0; j < 3; j = j + 1)
                    d.rgb0[i][j] = extend(d.in.Pixel.per_vertex_data[i].rgb[j]) * zeroExtend(unpack(d.e[i][26:17]));
            s3.enq(d);
        endrule

        rule rl_s4;
            UVInterpInternal d <- pop(s3);
            d.u1 = replicate(0);
            // TODO: can we fold this accumulation into the DSP somehow?
            for(Integer i = 0; i < 3; i = i + 1)
                for(Integer j = 0; j < 2; j = j + 1)
                    d.u1[j] = d.u1[j] + d.u0[i][j];
            d.rgb1 = replicate(0);
            for(Integer i = 0; i < 3; i = i + 1)
                for(Integer j = 0; j < 3; j = j + 1)
                    d.rgb1[j] = d.rgb1[j] + d.rgb0[i][j];
            s4.enq(d);
        endrule

        rule rl_s5;
            UVInterpInternal d <- pop(s4);
            for(Integer j = 0; j < 2; j = j + 1)
                d.u2[j] = truncate(d.u1[j] >> 18);
            for(Integer j = 0; j < 3; j = j + 1)
                d.rgb2[j] = truncate(d.rgb1[j] >> 10);
            case(d.in) matches
                tagged Flush: f_out.enq(tagged Flush);
                tagged Pixel .p: f_out.enq(tagged Pixel {
                    x: p.x,
                    y: p.y,
                    per_vertex: PerVertex {
                        uv: d.u2,
                        rgb: d.rgb2
                    }
                });
            endcase
        endrule

        interface in = to_FIFOF_I(f_in);
        interface out = to_FIFOF_O(f_out);
    endmodule

endpackage