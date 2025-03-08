package Defs;

    import Vector :: *;

    typedef 18 WdTexCoord;
    typedef Bit #(WdTexCoord) TexCoord;

    typedef struct {
        Vector #(2, TexCoord) uv;
        Vector #(3, Bit #(8)) rgb;
    } PerVertex
    deriving (Bits, FShow);

    typedef Vector #(3, PerVertex) PerVertexData;

endpackage