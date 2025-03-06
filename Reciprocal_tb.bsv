package Reciprocal_tb;

    import Reciprocal :: *;
    import StmtFSM :: *;
    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Randomizable :: *;
    import List :: *;
    import Util :: *;
    
    typedef 32 W;

    module mkReciprocal_tb();

        Randomize #(Bit #(W)) rnd <- mkGenericRandomizer;

        Integer n = 1000;

        FIFOF #(Bit #(W)) inputs <- mkSizedFIFOF(n);
        FIFOF #(ReciprocalResult #(W)) outputs <- mkSizedFIFOF(n);

        Reciprocal #(W) dut <- mkReciprocal;

        Reg #(Bit #(32)) i <- mkRegU;

        function ActionValue #(ReciprocalResult #(W)) golden(Bit #(W) x);
        actionvalue
            Bit #(TAdd#(W, W)) y = (1<<(fromInteger(2*valueOf(W))-1)) / extend(x);
            UInt #(TLog #(TAdd #(TAdd #(W, W), 1))) sh = countZerosMSB(y);
            Bit #(W) value = truncate(y << sh >> fromInteger(valueOf(W)));
            return ReciprocalResult {
                value: value,
                shift: truncate(pack(32 - sh))
            };
        endactionvalue
        endfunction

        function Bit #(W) delta(Bit #(W) a, Bit #(W) b) =
            a > b ? a - b : b - a;
        
        Reg #(Bit #(W)) v <- mkRegU;
        Reg #(Bit #(32)) errors <- mkReg (0);

        mkAutoFSM(seq
            rnd.cntrl.init();
            for(i <= 0; i < fromInteger(n); i <= i + 1) seq
                v <= 0;
                while(v == 0) action
                    let a <- rnd.next();
                    let b <- rnd.next();
                    v <= a >> (b & 31);
                endaction
                action
                    let g <- golden(v);
                    inputs.enq(v);
                    outputs.enq(g);
                endaction
            endseq
            par
                while(inputs.notEmpty) action
                    let v <- pop(inputs);
                    dut.in.enq(v);
                endaction
                seq
                    for(i <= 0; i < fromInteger(n); i <= i + 1) seq
                        action
                            let got <- pop_o(dut.out);
                            let exp <- pop(outputs);
                            if(!(
                                    got.shift == exp.shift && delta(got.value, exp.value) <= 1 ||
                                    exp.shift == got.shift + 1 && got.value == ~0 && exp.value == (1<<(valueOf(W)-1))))
                            begin
                                if(errors < 25)
                                    $display("@FAIL    input %0d: expected %x << %d, got %x << %d", i, exp.value, exp.shift, got.value, got.shift);
                                errors <= errors + 1;
                            end
                        endaction
                    endseq
                    if(errors == 0)
                        $display("@PASS");
                    $finish;
                endseq
                seq
                    delay(100000);
                    $display("@FAIL    timed out");
                    $finish;
                endseq
            endpar
        endseq);

    endmodule

endpackage