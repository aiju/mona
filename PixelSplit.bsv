package PixelSplit;

    import FIFOF :: *;
    import Semi_FIFOF :: *;
    import Vector :: *;
    import FineRaster :: *;
    import CoarseRaster :: *;
    import UVInterp :: *;
    import SpecialFIFOs :: *;

    interface PixelSplit;
        interface FIFOF_I #(FineRasterOut) in;
        interface FIFOF_O #(UVInterpIn) out;
    endinterface

    (* synthesize *)
    module mkPixelSplit(PixelSplit);
        FIFOF #(UVInterpIn) f_out <- mkPipelineFIFOF;

        Reg #(Bool) c_active[2] <- mkCReg (2, False);
        Reg #(FineRasterOut) c_current[2] <- mkCRegU (2);
        Reg #(Bool) active = c_active[0];
        Reg #(FineRasterOut) current = c_current[0];

        rule rl_process_flush (active &&& current matches tagged Flush);
            f_out.enq(tagged Flush);
            active <= False;
        endrule

        rule rl_process_empty_tile (active &&& current matches tagged Tile .tile &&& tile.pixels == 0);
            active <= False;
        endrule

        function Int #(27) edge_vec(Bit #(4) yx, Vector #(3, EdgeFn) edge_fns, Integer i)
            = edge_fns[i].a
                + edge_fns[i].x * zeroExtend(unpack(yx[1:0]))
                + edge_fns[i].y * zeroExtend(unpack(yx[3:2]));

        rule rl_process_pixel (active &&& current matches tagged Tile .tile &&& tile.pixels != 0);
            Bit #(4) yx = truncate(pack(countZerosLSB(tile.pixels)));
            f_out.enq(tagged Pixel {
                uv: tile.uv,
                x: zeroExtend(tile.tx) * 4 + zeroExtend(unpack(yx[1:0])),
                y: zeroExtend(tile.ty) * 4 + zeroExtend(unpack(yx[3:2])),
                edge_vec: genWith(edge_vec(yx, tile.edge_fns))
            });
            let new_pixels = tile.pixels & (tile.pixels - 1);
            active <= new_pixels != 0;
            current <= tagged Tile {
                pixels: new_pixels,
                uv: tile.uv,
                edge_fns: tile.edge_fns,
                tx: tile.tx,
                ty: tile.ty
            };
        endrule

        interface in = interface FIFOF_I;
            method Action enq(value) if(!c_active[1]);
                c_active[1] <= True;
                c_current[1] <= value;
            endmethod
            method Bool notFull;
                return !c_active[1];
            endmethod
        endinterface;
        interface out = to_FIFOF_O(f_out);
    endmodule

endpackage