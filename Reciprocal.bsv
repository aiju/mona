package Reciprocal;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import Util :: *;

    typedef struct {
        Bit #(n) value;
        Bit #(TLog#(n)) shift;
    } ReciprocalResult #(numeric type n)
    deriving (Bits, FShow);

    interface Reciprocal #(numeric type n);
        interface FIFOF_I #(Bit #(n)) in;
        interface FIFOF_O #(ReciprocalResult #(n)) out;
    endinterface

    typedef struct {
        Bit #(n) x;
        Bit #(n) y;
        Bit #(n) z;
        Bit #(TLog#(n)) sh;
    } RData#(numeric type n)
    deriving (Bits, FShow);

    typedef 4 N_Iter;

    module mkReciprocal (Reciprocal #(n)) provisos (
        Add#(a__, TLog#(n), TLog#(TAdd#(1, n))),
        Add#(b__, n, TMul#(2, n))
    );

        FIFOF #(Bit #(n)) f_in <- mkFIFOF;
        FIFOF #(RData #(n)) s0 <- mkFIFOF;
        Vector #(TAdd #(N_Iter, 1), FIFOF #(RData #(n))) s1 <- replicateM (mkFIFOF);
        Vector #(N_Iter, FIFOF #(RData #(n))) s2 <- replicateM (mkFIFOF);

        rule rl_s0;
            let x <- pop(f_in);
            Bit #(TLog#(n)) sh = pack(truncate(countZerosMSB(x)));
            s0.enq(RData {x: x << sh, y: ?, z: ?, sh: sh});
            //x is Q0.n
        endrule

        rule rl_s1;
            let d <- pop(s0);
            Bit #(TMul#(2,n)) y0 = extend(d.x) * ((32<<(valueOf(n)-1)) / 17);
            //y0 is Q1.2n-1
            Bit #(TMul#(2,n)) y1 = (48<<(valueOf(n) - 1)) / 17 - (y0 >> (valueOf(n)));
            //y1 is Qn+1.n-1
            Bit #(n) y = truncate(y1);
            //y is Q1.n-1
            s1[0].enq(RData {x: d.x, y: y, z: ?, sh: d.sh});
        endrule

        for(Integer i = 0; i < valueOf(N_Iter); i = i + 1) begin
            rule rl_s2;
                let d <- pop(s1[i]);
                Bit #(TMul#(2,n)) z0 = (1<<(valueOf(n)*2-1)) - extend(d.x) * extend(d.y);
                //z is Q1.2n-1
                Bit #(n) z = truncate(z0 >> valueOf(n));
                //z is Q1.n-1
                s2[i].enq(RData {x: d.x, y: d.y, z: z, sh: d.sh});
            endrule

            rule rl_s3;
                let d <- pop(s2[i]);
                Bit #(TMul#(2,n)) y0 = (extend(d.y) << valueOf(n) - 1) + extend(d.y) * signExtend(d.z);
                //y0 is Q2.2n-2
                Bit #(n) y = truncate(y0 >> valueOf(n) - 1);
                //y is Q1.n-1
                s1[i+1].enq(RData {x: d.x, y: y, z: ?, sh: d.sh});
            endrule
        end

        interface in = to_FIFOF_I(f_in);
        interface out = interface FIFOF_O;
            method notEmpty = s1[valueOf(N_Iter)].notEmpty;
            method deq = s1[valueOf(N_Iter)].deq;
            method first;
                let d = s1[valueOf(N_Iter)].first;
                return ReciprocalResult { value: d.y, shift: d.sh };
            endmethod
        endinterface;
    endmodule

endpackage