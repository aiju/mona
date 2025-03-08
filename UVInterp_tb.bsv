package UVInterp_tb;

    import UVInterp :: *;
    import StmtFSM :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import Defs :: *;

    module mkUVInterp_tb();

        UVInterp dut <- mkUVInterp;


        Reg #(Int #(27)) x <- mkReg (0);

        rule rl_input;
            Vector #(3, Int #(27)) edge_vec = newVector;
            edge_vec[0] = x;
            edge_vec[1] = (1<<10) - x;
            edge_vec[2] = 0;
            x <= x + 1;
            PerVertexData pv = unpack(0);
            pv[0].uv[0] = 2<<10;
            pv[1].uv[0] = 0;
            pv[2].uv[0] = 0;
            pv[0].uv[1] = 0;
            pv[1].uv[1] = 1<<10;
            pv[2].uv[1] = 1<<10;
            Vector #(3, Vector #(3, Bit #(8))) rgb = replicate(replicate(0));
            pv[0].rgb[0] = 255;
            pv[1].rgb[1] = 255;
            pv[2].rgb[2] = 255;
            dut.in.enq(tagged Pixel {
                edge_vec: edge_vec,
                per_vertex_data: pv,
                x: 0,
                y: 0
            });
        endrule

        rule rl_output;
            let v <- pop_o(dut.out); $display("%t U  %x V %x RGB %x", $time, v.Pixel.per_vertex.uv[0], v.Pixel.per_vertex.uv[1], pack(v.Pixel.per_vertex.rgb));
        endrule

        mkAutoFSM(delay(1000));

    endmodule

endpackage