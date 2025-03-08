package UVInterp_tb;

    import UVInterp :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;

    module mkUVInterp_tb();

        UVInterp dut <- mkUVInterp;


        Reg #(Int #(27)) x <- mkReg (0);

        rule rl_input;
            Vector #(3, Int #(27)) edge_vec = newVector;
            edge_vec[0] = x;
            edge_vec[1] = (1<<10) - x;
            edge_vec[2] = 0;
            x <= x + 1;
            Vector #(3, Vector #(2, Int #(27))) uv = newVector;
            uv[0][0] = 2<<20;
            uv[1][0] = 0;
            uv[2][0] = 0;
            uv[0][1] = 0;
            uv[1][1] = 1<<25;
            uv[2][1] = 1<<25;
            Vector #(3, Vector #(3, Bit #(8))) rgb = replicate(replicate(0));
            rgb[0][0] = 255;
            rgb[1][1] = 255;
            rgb[2][2] = 255;
            dut.in.enq(tagged Pixel {
                edge_vec: edge_vec,
                uv: uv,
                rgb: rgb,
                x: 0,
                y: 0
            });
        endrule

        rule rl_output;
            let v <- pop_o(dut.out); $display("%t U  %x V %x RGB %x", $time, v.Pixel.u, v.Pixel.v, pack(v.Pixel.rgb));
        endrule

        mkAutoFSM(delay(1000));

    endmodule

endpackage