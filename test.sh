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

diff --color -w <(cat $trace_file | ../memhier/memhier_ref | head -n -26) <(cat $trace_file | RUSTFLAGS="-Awarnings" cargo run)

if [ $? -eq 0 ] 
then
    echo "Test passed!"
fi
