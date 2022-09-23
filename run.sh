#!/bin/bash

case $1 in
    1)
        trace_file=../memhier/trace.dat
        ;;
    2)
        trace_file=../memhier/ext_trace.dat
        ;;
    3)
        trace_file=../memhier/wr_trace.dat
        ;;
    4)
        trace_file=../memhier/long_trace.dat
        ;;
    *)
        echo "Unknown test case. Exiting..."
        exit
        ;;
esac

cat $trace_file | RUSTFLAGS="-Awarnings" cargo run
