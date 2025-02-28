package Reciprocal_tb;

    import Reciprocal :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;

    module mkReciprocal_tb();

        Reciprocal #(32) reciprocal <- mkReciprocal;

        mkAutoFSM(seq
            reciprocal.in.enq(99999);
            $display("%x", reciprocal.out.first.value);
            $display("%d", reciprocal.out.first.shift);
        endseq);

    endmodule

endpackage