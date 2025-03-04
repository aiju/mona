package UVInterp_tb;

    import UVInterp :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;

    module mkUVInterp_tb();

        UVInterp dut <- mkUVInterp;

        rule rl_input;
            Vector #(3, Int #(27)) edge_vec = newVector;
            edge_vec[0] = 1<<10;
            edge_vec[1] = 1<<10;
            edge_vec[2] = 0;
            Vector #(3, Vector #(2, Int #(27))) uv = newVector;
            uv[0][0] = 2<<20;
            uv[1][0] = 0;
            uv[2][0] = 0;
            uv[0][1] = 0;
            uv[1][1] = 1<<25;
            uv[2][1] = 1<<25;
            dut.in.enq(tagged Pixel {
                edge_vec: edge_vec,
                uv: uv
            });
        endrule

        rule rl_output;
            let v <- pop_o(dut.out); $display("%t ", $time, fshow(v));
        endrule

        mkAutoFSM(delay(1000));

    endmodule

endpackage