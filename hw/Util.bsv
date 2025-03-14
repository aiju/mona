package Util;

    import FIFOF :: *;

    function ActionValue #(t) pop(FIFOF #(t) fifo);
        actionvalue
            fifo.deq;
            return fifo.first;
        endactionvalue
    endfunction

endpackage