package UVInterp_tb;

    import UVInterp :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;

    module mkUVInterp_tb();

        UVInterp dut <- mkUVInterp;

        mkAutoFSM(seq
            action
                Vector #(3, Int #(27)) edge_vec = newVector;
                edge_vec[0] = 1<<10;
                edge_vec[1] = 1<<10;
                edge_vec[2] = 0;
                Vector #(3, Int #(27)) u = newVector;
                u[0] = 2<<20;
                u[1] = 0;
                u[2] = 0;
                Vector #(3, Int #(27)) v = newVector;
                v[0] = 0;
                v[1] = 1<<25;
                v[2] = 1<<25;
                dut.in.enq(UVInterpIn {
                    edge_vec: edge_vec,
                    u: u,
                    v: v
                });
            endaction
            $display(fshow(dut.out.first));
        endseq);

    endmodule

endpackage