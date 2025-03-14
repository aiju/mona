package CoarseRaster_tb;

    import CoarseRaster :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import StmtFSM :: *;
    import Real :: *;

    module mkCoarseRaster_tb(Empty);
        let dut <- mkCoarseRaster;

        function CoarseRasterIn calc_in(Vector #(3, Vector #(2, Real)) coords);
            CoarseRasterIn p = ?;
            p.min_x = 127; p.min_y = 127;
            p.max_x = 0; p.max_y = 0;
            for(Integer i = 0; i < 3; i = i + 1) begin
                Real ax = -(coords[(i+2)%3][1] - coords[(i+1)%3][1]);
                Real ay = coords[(i+2)%3][0] - coords[(i+1)%3][0];
                Real c = -(ax * coords[(i+1)%3][0] + ay * coords[(i+1)%3][1]);
                p.edge_fns[i].x = fromInteger(floor(ax));
                p.edge_fns[i].y = fromInteger(floor(ay));
                p.edge_fns[i].a = fromInteger(floor(c));
                UInt #(7) tx = fromInteger(floor(coords[i][0] / 4));
                UInt #(7) ty = fromInteger(floor(coords[i][1] / 4));
                if(tx < p.min_x) p.min_x = tx;
                if(tx > p.max_x) p.max_x = tx;
                if(ty < p.min_y) p.min_y = ty;
                if(ty > p.max_y) p.max_y = ty;
            end
            for(Integer i = 0; i < 3; i = i + 1)
                p.edge_fns[i].a = p.edge_fns[i].a + p.edge_fns[i].x * unpack(pack(extend(p.min_x))) * 4 + p.edge_fns[i].y * unpack(pack(extend(p.min_y))) * 4;
            return p;
        endfunction

        mkAutoFSM(seq
            action
                Vector #(3, Vector #(2, Real)) coords = newVector;
                coords[0][0] = 100;
                coords[0][1] = 100;
                coords[1][0] = 200;
                coords[1][1] = 100;
                coords[2][0] = 200;
                coords[2][1] = 200;
                dut.in.enq(calc_in(coords));
            endaction
            delay(1000);
        endseq);

        rule rl_out;
            $display(fshow(dut.out.first));
            dut.out.deq;
        endrule
    endmodule

endpackage